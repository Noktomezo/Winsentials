use std::fs;
use std::path::{Path, PathBuf};

use windows::Win32::System::Com::CoTaskMemFree;
use windows::Win32::UI::Shell::{
    FOLDERID_CommonStartup, FOLDERID_Startup, KNOWN_FOLDER_FLAG, SHGetKnownFolderPath,
};

use crate::error::AppError;
use crate::startup::disabled_store::{
    DisabledStartupFileMetadata, hex_encode, startup_disabled_dir, startup_sidecar_path,
};
use crate::startup::presentation::{
    startup_folder_presentation, startup_folder_presentation_light,
};
use crate::startup::types::{
    StartupEntry, StartupEntryDetails, StartupScope, StartupSource, StartupStatus,
};

pub fn list_entries() -> Result<Vec<StartupEntry>, AppError> {
    let mut entries = Vec::new();
    entries.extend(list_active_entries(StartupScope::CurrentUser)?);
    entries.extend(list_active_entries(StartupScope::AllUsers)?);
    entries.extend(list_disabled_entries(StartupScope::CurrentUser)?);
    entries.extend(list_disabled_entries(StartupScope::AllUsers)?);
    Ok(entries)
}

pub fn entry(id: &str) -> Result<StartupEntry, AppError> {
    match find_active_path(id) {
        Ok((scope, active_path)) => {
            return build_entry_from_path(
                active_path,
                scope,
                StartupStatus::Enabled,
                false,
                Some(id.to_string()),
                true,
            );
        }
        Err(error) if is_startup_folder_entry_not_found(&error, id) => {}
        Err(error) => return Err(error),
    }

    let metadata = read_disabled_metadata(id)?;
    let (scope, _, disabled_file_path) = validate_metadata_paths(&metadata)?;
    build_entry_from_path(
        disabled_file_path,
        scope,
        StartupStatus::Disabled,
        false,
        Some(metadata.id),
        true,
    )
}

pub fn disable_entry(id: &str) -> Result<StartupEntry, AppError> {
    let (scope, original_path) = find_active_path(id)?;
    let disabled_dir = startup_disabled_dir(scope)?;
    fs::create_dir_all(&disabled_dir)?;

    let file_name = original_path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| AppError::message("startup file has no valid name"))?;
    let disabled_file_path = unique_disabled_path(&disabled_dir, file_name, id);

    let metadata = DisabledStartupFileMetadata {
        id: id.to_string(),
        original_path: original_path.to_string_lossy().into_owned(),
        disabled_file_path: disabled_file_path.to_string_lossy().into_owned(),
        scope,
        disabled_at: "now".to_string(),
        source_kind: "startup_folder".to_string(),
    };
    let sidecar_path = startup_sidecar_path(&disabled_file_path);
    let metadata_bytes = serde_json::to_vec_pretty(&metadata)?;

    fs::rename(&original_path, &disabled_file_path)?;

    if let Err(error) = fs::write(&sidecar_path, &metadata_bytes) {
        let _ = fs::remove_file(&sidecar_path);
        return match fs::rename(&disabled_file_path, &original_path) {
            Ok(()) => Err(AppError::from(error)),
            Err(restore_error) => Err(AppError::message(format!(
                "failed to write startup metadata and restore startup entry: {error}; restore error: {restore_error}"
            ))),
        };
    }

    build_entry_from_path(
        disabled_file_path,
        scope,
        StartupStatus::Disabled,
        false,
        Some(id.to_string()),
        true,
    )
}

pub fn enable_entry(id: &str) -> Result<StartupEntry, AppError> {
    let metadata = read_disabled_metadata(id)?;
    let (scope, original_path, disabled_file_path) = validate_metadata_paths(&metadata)?;
    if let Some(parent) = original_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::rename(&disabled_file_path, &original_path)?;
    delete_sidecar(&disabled_file_path)?;

    build_entry_from_path(
        original_path,
        scope,
        StartupStatus::Enabled,
        false,
        Some(metadata.id),
        true,
    )
}

pub fn delete_entry(id: &str) -> Result<(), AppError> {
    if let Ok((_, active_path)) = find_active_path(id) {
        fs::remove_file(active_path)?;
        return Ok(());
    }

    let metadata = read_disabled_metadata(id)?;
    let (_, _, disabled_file_path) = validate_metadata_paths(&metadata)?;
    if disabled_file_path.exists() {
        fs::remove_file(&disabled_file_path)?;
    }
    delete_sidecar(&disabled_file_path)
}

pub fn entry_details(id: &str) -> Result<StartupEntryDetails, AppError> {
    if let Ok((scope, active_path)) = find_active_path(id) {
        let entry = build_entry_from_path(
            active_path.clone(),
            scope,
            StartupStatus::Enabled,
            false,
            Some(id.to_string()),
            true,
        )?;
        return Ok(build_details(entry, scope, active_path));
    }

    let metadata = read_disabled_metadata(id)?;
    let (scope, _, disabled_file_path) = validate_metadata_paths(&metadata)?;
    let entry = build_entry_from_path(
        disabled_file_path.clone(),
        scope,
        StartupStatus::Disabled,
        false,
        Some(metadata.id),
        true,
    )?;
    Ok(build_details(entry, scope, disabled_file_path))
}

fn list_active_entries(scope: StartupScope) -> Result<Vec<StartupEntry>, AppError> {
    let startup_dir = startup_dir(scope)?;
    let mut entries = Vec::new();

    if !startup_dir.exists() {
        return Ok(entries);
    }

    for item in fs::read_dir(startup_dir)? {
        let path = item?.path();
        if !path.is_file() {
            continue;
        }
        if is_sidecar(&path) {
            continue;
        }
        if should_skip_startup_path(&path) {
            continue;
        }
        entries.push(build_entry_from_path(
            path,
            scope,
            StartupStatus::Enabled,
            false,
            None,
            false,
        )?);
    }

    Ok(entries)
}

fn list_disabled_entries(scope: StartupScope) -> Result<Vec<StartupEntry>, AppError> {
    let disabled_dir = startup_disabled_dir(scope)?;
    let mut entries = Vec::new();

    if !disabled_dir.exists() {
        return Ok(entries);
    }

    for item in fs::read_dir(&disabled_dir)? {
        let path = item?.path();
        if !is_sidecar(&path) {
            continue;
        }

        let metadata: DisabledStartupFileMetadata = serde_json::from_slice(&fs::read(&path)?)?;
        let Ok((metadata_scope, _, disabled_file_path)) = validate_metadata_paths(&metadata) else {
            continue;
        };
        if metadata_scope != scope {
            continue;
        }
        if !disabled_file_path.exists() {
            continue;
        }
        entries.push(build_entry_from_path(
            disabled_file_path,
            scope,
            StartupStatus::Disabled,
            false,
            Some(metadata.id),
            false,
        )?);
    }

    Ok(entries)
}

fn find_active_path(id: &str) -> Result<(StartupScope, PathBuf), AppError> {
    for scope in [StartupScope::CurrentUser, StartupScope::AllUsers] {
        let startup_dir = startup_dir(scope)?;
        if !startup_dir.exists() {
            continue;
        }

        for item in fs::read_dir(&startup_dir)? {
            let path = item?.path();
            if !path.is_file() || is_sidecar(&path) {
                continue;
            }
            if should_skip_startup_path(&path) {
                continue;
            }

            let entry_id = startup_folder_id(scope, &path);
            if entry_id == id {
                return Ok((scope, path));
            }
        }
    }

    Err(AppError::message(format!(
        "startup folder entry not found: {id}"
    )))
}

fn is_startup_folder_entry_not_found(error: &AppError, id: &str) -> bool {
    matches!(
        error,
        AppError::Message(message) if message == &format!("startup folder entry not found: {id}")
    )
}

fn read_disabled_metadata(id: &str) -> Result<DisabledStartupFileMetadata, AppError> {
    for scope in [StartupScope::CurrentUser, StartupScope::AllUsers] {
        let disabled_dir = startup_disabled_dir(scope)?;
        if !disabled_dir.exists() {
            continue;
        }

        for item in fs::read_dir(&disabled_dir)? {
            let path = item?.path();
            if !is_sidecar(&path) {
                continue;
            }

            let metadata: DisabledStartupFileMetadata = serde_json::from_slice(&fs::read(&path)?)?;
            if metadata.id == id {
                return Ok(metadata);
            }
        }
    }

    Err(AppError::message(format!(
        "disabled startup folder entry not found: {id}"
    )))
}

fn build_entry_from_path(
    path: PathBuf,
    scope: StartupScope,
    status: StartupStatus,
    run_once: bool,
    id_override: Option<String>,
    enrich: bool,
) -> Result<StartupEntry, AppError> {
    let name = path
        .file_stem()
        .and_then(|value| value.to_str())
        .or_else(|| path.file_name().and_then(|value| value.to_str()))
        .unwrap_or("Startup entry")
        .to_string();
    let location_label = startup_dir(scope)?.to_string_lossy().into_owned();
    let path_string = path.to_string_lossy().into_owned();
    let presentation = if enrich {
        startup_folder_presentation(&path)
    } else {
        startup_folder_presentation_light(&path)
    };

    Ok(StartupEntry {
        id: id_override.unwrap_or_else(|| startup_folder_id(scope, &path)),
        name,
        display_name: presentation.display_name,
        source: StartupSource::StartupFolder,
        scope,
        status,
        command: presentation
            .target_path
            .as_ref()
            .map(|target| match presentation.arguments.as_deref() {
                Some(arguments) if !arguments.is_empty() => format!("{target} {arguments}"),
                _ => target.clone(),
            })
            .or(Some(path_string.clone())),
        target_path: presentation.target_path,
        arguments: presentation.arguments,
        working_directory: presentation.working_directory,
        location_label,
        source_display: "Startup Folder".to_string(),
        run_once,
        publisher: presentation.publisher,
        icon_data_url: presentation.icon_data_url,
        registry_path: None,
        task_path: None,
        last_error: None,
    })
}

fn build_details(
    entry: StartupEntry,
    scope: StartupScope,
    file_path: PathBuf,
) -> StartupEntryDetails {
    StartupEntryDetails {
        entry,
        registry_hive: None,
        registry_path: None,
        registry_value_name: None,
        startup_folder_path: startup_dir(scope)
            .ok()
            .map(|value| value.to_string_lossy().into_owned()),
        startup_file_path: Some(file_path.to_string_lossy().into_owned()),
        task_path: None,
        task_author: None,
        task_description: None,
        task_triggers: vec![],
        task_actions: vec![],
        raw_xml_preview: None,
    }
}

fn startup_dir(scope: StartupScope) -> Result<PathBuf, AppError> {
    known_startup_dir(scope)
}

fn known_startup_dir(scope: StartupScope) -> Result<PathBuf, AppError> {
    let folder_id = match scope {
        StartupScope::CurrentUser => &FOLDERID_Startup,
        StartupScope::AllUsers => &FOLDERID_CommonStartup,
    };
    let folder_label = match scope {
        StartupScope::CurrentUser => "Startup",
        StartupScope::AllUsers => "Common Startup",
    };

    unsafe {
        let path_ptr = SHGetKnownFolderPath(folder_id, KNOWN_FOLDER_FLAG(0), None)
            .map_err(AppError::from)
            .map_err(|_| {
                AppError::message(format!("failed to resolve {folder_label} known folder"))
            })?;

        let path_string = path_ptr.to_string().map_err(|error| {
            AppError::message(format!(
                "failed to decode {folder_label} known folder path: {error}"
            ))
        });

        if !path_ptr.is_null() {
            CoTaskMemFree(Some(path_ptr.0 as _));
        }

        path_string.map(PathBuf::from)
    }
}

fn startup_folder_id(scope: StartupScope, path: &Path) -> String {
    format!(
        "folder:{}:{}",
        scope_label(scope),
        path.to_string_lossy().to_ascii_lowercase()
    )
}

fn validate_metadata_paths(
    metadata: &DisabledStartupFileMetadata,
) -> Result<(StartupScope, PathBuf, PathBuf), AppError> {
    let scope = metadata.scope;
    let original_path = PathBuf::from(&metadata.original_path);
    let disabled_file_path = PathBuf::from(&metadata.disabled_file_path);
    let expected_startup_dir = startup_dir(scope)?;
    let expected_disabled_dir = startup_disabled_dir(scope)?;

    if original_path.parent() != Some(expected_startup_dir.as_path()) {
        return Err(AppError::message(format!(
            "disabled startup metadata points outside startup folder: {}",
            metadata.original_path
        )));
    }

    if disabled_file_path.parent() != Some(expected_disabled_dir.as_path()) {
        return Err(AppError::message(format!(
            "disabled startup metadata points outside disabled startup folder: {}",
            metadata.disabled_file_path
        )));
    }

    let expected_id = startup_folder_id(scope, &original_path);
    if metadata.id != expected_id {
        return Err(AppError::message(format!(
            "disabled startup metadata id does not match managed path: {}",
            metadata.id
        )));
    }

    Ok((scope, original_path, disabled_file_path))
}

fn scope_label(scope: StartupScope) -> &'static str {
    match scope {
        StartupScope::CurrentUser => "current_user",
        StartupScope::AllUsers => "all_users",
    }
}

fn unique_disabled_path(disabled_dir: &Path, file_name: &str, id: &str) -> PathBuf {
    let candidate = disabled_dir.join(file_name);
    if !candidate.exists() && !startup_sidecar_path(&candidate).exists() {
        return candidate;
    }

    let path = Path::new(file_name);
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or(file_name);
    let ext = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("");
    let suffix = &hex_encode(id)[..8];

    if ext.is_empty() {
        disabled_dir.join(format!("{stem}.{suffix}"))
    } else {
        disabled_dir.join(format!("{stem}.{suffix}.{ext}"))
    }
}

fn delete_sidecar(disabled_file_path: &Path) -> Result<(), AppError> {
    let sidecar_path = startup_sidecar_path(disabled_file_path);
    if sidecar_path.exists() {
        fs::remove_file(sidecar_path)?;
    }
    Ok(())
}

fn is_sidecar(path: &Path) -> bool {
    path.file_name()
        .and_then(|value| value.to_str())
        .map(|value| value.ends_with(".winsentials.json"))
        .unwrap_or(false)
}

fn should_skip_startup_path(path: &Path) -> bool {
    path.file_name()
        .and_then(|value| value.to_str())
        .map(|value| value.eq_ignore_ascii_case("desktop.ini"))
        .unwrap_or(false)
}
