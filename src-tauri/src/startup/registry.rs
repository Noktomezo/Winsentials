use std::io::ErrorKind;

use winreg::RegKey;
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};

use crate::error::AppError;
use crate::startup::disabled_store::{
    DisabledRegistryRecord, HKCU_DISABLED_REGISTRY_PATH, HKLM_DISABLED_REGISTRY_PATH, hex_encode,
};
use crate::startup::presentation::{registry_presentation, registry_presentation_light};
use crate::startup::types::{
    StartupEntry, StartupEntryDetails, StartupScope, StartupSource, StartupStatus,
};

const REGISTRY_LOCATIONS: [RegistryLocation; 6] = [
    RegistryLocation::new(
        "hkcu",
        r"Software\Microsoft\Windows\CurrentVersion\Run",
        StartupScope::CurrentUser,
        false,
    ),
    RegistryLocation::new(
        "hkcu",
        r"Software\Microsoft\Windows\CurrentVersion\RunOnce",
        StartupScope::CurrentUser,
        true,
    ),
    RegistryLocation::new(
        "hklm",
        r"Software\Microsoft\Windows\CurrentVersion\Run",
        StartupScope::AllUsers,
        false,
    ),
    RegistryLocation::new(
        "hklm",
        r"Software\Microsoft\Windows\CurrentVersion\RunOnce",
        StartupScope::AllUsers,
        true,
    ),
    RegistryLocation::new(
        "hklm",
        r"Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Run",
        StartupScope::AllUsers,
        false,
    ),
    RegistryLocation::new(
        "hklm",
        r"Software\WOW6432Node\Microsoft\Windows\CurrentVersion\RunOnce",
        StartupScope::AllUsers,
        true,
    ),
];

#[derive(Clone, Copy)]
struct RegistryLocation {
    hive: &'static str,
    path: &'static str,
    scope: StartupScope,
    run_once: bool,
}

impl RegistryLocation {
    const fn new(
        hive: &'static str,
        path: &'static str,
        scope: StartupScope,
        run_once: bool,
    ) -> Self {
        Self {
            hive,
            path,
            scope,
            run_once,
        }
    }
}

pub fn list_entries() -> Result<Vec<StartupEntry>, AppError> {
    let mut entries = Vec::new();

    for location in REGISTRY_LOCATIONS {
        entries.extend(list_active_entries(location)?);
    }

    entries.extend(list_disabled_entries("hkcu")?);
    entries.extend(list_disabled_entries("hklm")?);

    Ok(entries)
}

pub fn entry(id: &str) -> Result<StartupEntry, AppError> {
    match find_active_entry(id) {
        Ok((location, value_name, command)) => {
            return Ok(build_entry(
                RegistryEntryParams {
                    id: id.to_string(),
                    name: value_name,
                    command: Some(command),
                    scope: location.scope,
                    status: StartupStatus::Enabled,
                    run_once: location.run_once,
                    location_label: format_registry_location_label(location),
                    source_display: "Registry".to_string(),
                    hive: location.hive.to_string(),
                    registry_path: location.path.to_string(),
                    last_error: None,
                },
                true,
            ));
        }
        Err(error) if is_registry_entry_not_found(&error, id) => {}
        Err(error) => return Err(error),
    }

    let record = read_disabled_record(id)?;
    entry_from_record(record, StartupStatus::Disabled, true)
}

pub fn disable_entry(id: &str) -> Result<StartupEntry, AppError> {
    let (location, value_name, command) = find_active_entry(id)?;
    let encoded = hex_encode(id);
    let disabled_store = ensure_disabled_store(location.hive)?;
    let (subkey, _) = disabled_store.create_subkey(&encoded)?;
    let record = DisabledRegistryRecord {
        id: id.to_string(),
        original_hive: location.hive.to_string(),
        original_path: location.path.to_string(),
        value_name: value_name.clone(),
        command: command.clone(),
        run_once: location.run_once,
        scope: location.scope,
        disabled_at: "now".to_string(),
        source_kind: "registry".to_string(),
    };
    write_disabled_record(&subkey, &record)?;
    open_root(location.hive)?
        .create_subkey(location.path)?
        .0
        .delete_value(&value_name)
        .map_err(AppError::from)?;

    entry_from_record(record, StartupStatus::Disabled, true)
}

pub fn enable_entry(id: &str) -> Result<StartupEntry, AppError> {
    let record = read_disabled_record(id)?;
    let root = open_root(&record.original_hive)?;
    root.create_subkey(&record.original_path)?
        .0
        .set_value(&record.value_name, &record.command)
        .map_err(AppError::from)?;
    delete_disabled_record(id, &record.original_hive)?;

    entry_from_record(record, StartupStatus::Enabled, true)
}

pub fn delete_entry(id: &str) -> Result<(), AppError> {
    match find_active_entry(id) {
        Ok((location, value_name, _)) => {
            open_root(location.hive)?
                .create_subkey(location.path)?
                .0
                .delete_value(&value_name)
                .map_err(AppError::from)?;
            return Ok(());
        }
        Err(error) if is_registry_entry_not_found(&error, id) => {}
        Err(error) => return Err(error),
    }

    let record = read_disabled_record(id)?;
    delete_disabled_record(id, &record.original_hive)
}

pub fn entry_details(id: &str) -> Result<StartupEntryDetails, AppError> {
    match find_active_entry(id) {
        Ok((location, value_name, command)) => {
            let entry = build_entry(
                RegistryEntryParams {
                    id: id.to_string(),
                    name: value_name.clone(),
                    command: Some(command),
                    scope: location.scope,
                    status: StartupStatus::Enabled,
                    run_once: location.run_once,
                    location_label: format_registry_location_label(location),
                    source_display: "Registry".to_string(),
                    hive: location.hive.to_string(),
                    registry_path: location.path.to_string(),
                    last_error: None,
                },
                true,
            );

            return Ok(StartupEntryDetails {
                entry,
                registry_hive: Some(location.hive.to_ascii_uppercase()),
                registry_path: Some(location.path.to_string()),
                registry_value_name: Some(value_name),
                startup_folder_path: None,
                startup_file_path: None,
                task_path: None,
                task_author: None,
                task_description: None,
                task_triggers: vec![],
                task_actions: vec![],
                raw_xml_preview: None,
            });
        }
        Err(error) if is_registry_entry_not_found(&error, id) => {}
        Err(error) => return Err(error),
    }

    let record = read_disabled_record(id)?;
    let entry = entry_from_record(record.clone(), StartupStatus::Disabled, true)?;
    Ok(StartupEntryDetails {
        entry,
        registry_hive: Some(record.original_hive.to_ascii_uppercase()),
        registry_path: Some(record.original_path),
        registry_value_name: Some(record.value_name),
        startup_folder_path: None,
        startup_file_path: None,
        task_path: None,
        task_author: None,
        task_description: None,
        task_triggers: vec![],
        task_actions: vec![],
        raw_xml_preview: None,
    })
}

fn list_active_entries(location: RegistryLocation) -> Result<Vec<StartupEntry>, AppError> {
    let root = open_root(location.hive)?;
    let key = match root.open_subkey(location.path) {
        Ok(key) => key,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(vec![]),
        Err(error) => return Err(AppError::from(error)),
    };

    let mut entries = Vec::new();

    for item in key.enum_values() {
        let (value_name, _) = item.map_err(AppError::from)?;
        let command = key
            .get_value::<String, _>(&value_name)
            .map_err(AppError::from)?;
        if is_system_command(&command) {
            continue;
        }
        let id = registry_entry_id(location.hive, location.path, &value_name);
        entries.push(build_entry(
            RegistryEntryParams {
                id,
                name: value_name,
                command: Some(command),
                scope: location.scope,
                status: StartupStatus::Enabled,
                run_once: location.run_once,
                location_label: format_registry_location_label(location),
                source_display: "Registry".to_string(),
                hive: location.hive.to_string(),
                registry_path: location.path.to_string(),
                last_error: None,
            },
            false,
        ));
    }

    Ok(entries)
}

fn list_disabled_entries(hive: &str) -> Result<Vec<StartupEntry>, AppError> {
    let store = match ensure_disabled_store_read(hive) {
        Ok(store) => store,
        Err(AppError::Io(error)) if error.kind() == ErrorKind::NotFound => return Ok(vec![]),
        Err(error) => return Err(error),
    };

    let mut entries = Vec::new();
    for subkey_name in store.enum_keys() {
        let subkey_name = subkey_name.map_err(AppError::from)?;
        let subkey = store.open_subkey(&subkey_name).map_err(AppError::from)?;
        let record = read_disabled_record_from_key(&subkey)?;
        if is_system_command(&record.command) {
            continue;
        }
        entries.push(entry_from_record(record, StartupStatus::Disabled, false)?);
    }

    Ok(entries)
}

fn entry_from_record(
    record: DisabledRegistryRecord,
    status: StartupStatus,
    enrich: bool,
) -> Result<StartupEntry, AppError> {
    Ok(build_entry(
        RegistryEntryParams {
            id: record.id,
            name: record.value_name,
            command: Some(record.command),
            scope: record.scope,
            status,
            run_once: record.run_once,
            location_label: format!(
                "{} {}",
                record.original_hive.to_ascii_uppercase(),
                if record.run_once { "RunOnce" } else { "Run" }
            ),
            source_display: "Registry".to_string(),
            hive: record.original_hive,
            registry_path: record.original_path,
            last_error: None,
        },
        enrich,
    ))
}

fn find_active_entry(id: &str) -> Result<(RegistryLocation, String, String), AppError> {
    for location in REGISTRY_LOCATIONS {
        let root = open_root(location.hive)?;
        let key = match root.open_subkey(location.path) {
            Ok(key) => key,
            Err(error) if error.kind() == ErrorKind::NotFound => continue,
            Err(error) => return Err(AppError::from(error)),
        };

        for item in key.enum_values() {
            let (value_name, _) = item.map_err(AppError::from)?;
            let entry_id = registry_entry_id(location.hive, location.path, &value_name);
            if entry_id != id {
                continue;
            }

            let command = key
                .get_value::<String, _>(&value_name)
                .map_err(AppError::from)?;
            return Ok((location, value_name, command));
        }
    }

    Err(AppError::RegistryEntryNotFound { id: id.to_string() })
}

fn is_registry_entry_not_found(error: &AppError, id: &str) -> bool {
    matches!(
        error,
        AppError::RegistryEntryNotFound { id: not_found_id } if not_found_id == id
    )
}

fn build_entry(params: RegistryEntryParams, enrich: bool) -> StartupEntry {
    let presentation = if enrich {
        registry_presentation(&params.name, params.command.as_deref())
    } else {
        registry_presentation_light(&params.name, params.command.as_deref())
    };

    StartupEntry {
        id: params.id,
        name: params.name,
        display_name: presentation.display_name,
        source: StartupSource::Registry,
        scope: params.scope,
        status: params.status,
        command: params.command,
        target_path: presentation.target_path,
        arguments: presentation.arguments,
        working_directory: presentation.working_directory,
        location_label: params.location_label,
        source_display: params.source_display,
        run_once: params.run_once,
        publisher: presentation.publisher,
        icon_data_url: presentation.icon_data_url,
        registry_path: Some(format!(
            "{}\\{}",
            params.hive.to_ascii_uppercase(),
            params.registry_path
        )),
        task_path: None,
        last_error: params.last_error,
    }
}

struct RegistryEntryParams {
    id: String,
    name: String,
    command: Option<String>,
    scope: StartupScope,
    status: StartupStatus,
    run_once: bool,
    location_label: String,
    source_display: String,
    hive: String,
    registry_path: String,
    last_error: Option<String>,
}

fn write_disabled_record(key: &RegKey, record: &DisabledRegistryRecord) -> Result<(), AppError> {
    key.set_value("id", &record.id).map_err(AppError::from)?;
    key.set_value("original_hive", &record.original_hive)
        .map_err(AppError::from)?;
    key.set_value("original_path", &record.original_path)
        .map_err(AppError::from)?;
    key.set_value("value_name", &record.value_name)
        .map_err(AppError::from)?;
    key.set_value("command", &record.command)
        .map_err(AppError::from)?;
    key.set_value("run_once", &(record.run_once as u32))
        .map_err(AppError::from)?;
    key.set_value("scope", &record.scope)
        .map_err(AppError::from)?;
    key.set_value("disabled_at", &record.disabled_at)
        .map_err(AppError::from)?;
    key.set_value("source_kind", &record.source_kind)
        .map_err(AppError::from)?;
    Ok(())
}

fn read_disabled_record(id: &str) -> Result<DisabledRegistryRecord, AppError> {
    let hive = disabled_record_hive_from_id(id)?;
    let store = match ensure_disabled_store_read(hive) {
        Ok(store) => store,
        Err(AppError::Io(error)) if error.kind() == ErrorKind::NotFound => {
            return Err(AppError::RegistryEntryNotFound { id: id.to_string() });
        }
        Err(error) => return Err(error),
    };
    let subkey = match store.open_subkey(hex_encode(id)) {
        Ok(subkey) => subkey,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return Err(AppError::RegistryEntryNotFound { id: id.to_string() });
        }
        Err(error) => return Err(AppError::from(error)),
    };
    read_disabled_record_from_key(&subkey)
}

fn disabled_record_hive_from_id(id: &str) -> Result<&'static str, AppError> {
    if id.starts_with("reg:hklm:") {
        return Ok("hklm");
    }

    if id.starts_with("reg:hkcu:") {
        return Ok("hkcu");
    }

    Err(AppError::message(format!(
        "unsupported registry startup entry id: {id}"
    )))
}

fn read_disabled_record_from_key(key: &RegKey) -> Result<DisabledRegistryRecord, AppError> {
    Ok(DisabledRegistryRecord {
        id: key.get_value("id").map_err(AppError::from)?,
        original_hive: key.get_value("original_hive").map_err(AppError::from)?,
        original_path: key.get_value("original_path").map_err(AppError::from)?,
        value_name: key.get_value("value_name").map_err(AppError::from)?,
        command: key.get_value("command").map_err(AppError::from)?,
        run_once: key
            .get_value::<u32, _>("run_once")
            .map_err(AppError::from)?
            != 0,
        scope: key.get_value("scope").map_err(AppError::from)?,
        disabled_at: key.get_value("disabled_at").map_err(AppError::from)?,
        source_kind: key.get_value("source_kind").map_err(AppError::from)?,
    })
}

fn delete_disabled_record(id: &str, hive: &str) -> Result<(), AppError> {
    let store_path = disabled_store_path(hive);
    match open_root(hive)?.delete_subkey_all(format!(r"{store_path}\{}", hex_encode(id))) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(AppError::from(error)),
    }
}

fn ensure_disabled_store(hive: &str) -> Result<RegKey, AppError> {
    open_root(hive)?
        .create_subkey(disabled_store_path(hive))
        .map(|(key, _)| key)
        .map_err(AppError::from)
}

fn ensure_disabled_store_read(hive: &str) -> Result<RegKey, AppError> {
    open_root(hive)?
        .open_subkey(disabled_store_path(hive))
        .map_err(AppError::from)
}

fn open_root(hive: &str) -> Result<RegKey, AppError> {
    match hive {
        "hkcu" => Ok(RegKey::predef(HKEY_CURRENT_USER)),
        "hklm" => Ok(RegKey::predef(HKEY_LOCAL_MACHINE)),
        _ => Err(AppError::message(format!(
            "unsupported registry hive: {hive}"
        ))),
    }
}

fn registry_entry_id(hive: &str, path: &str, value_name: &str) -> String {
    format!(
        "reg:{}:{}:{}",
        hive.to_ascii_lowercase(),
        path.to_ascii_lowercase(),
        value_name.to_ascii_lowercase()
    )
}

fn disabled_store_path(hive: &str) -> &'static str {
    match hive {
        "hkcu" => HKCU_DISABLED_REGISTRY_PATH,
        "hklm" => HKLM_DISABLED_REGISTRY_PATH,
        _ => HKCU_DISABLED_REGISTRY_PATH,
    }
}

fn format_registry_location_label(location: RegistryLocation) -> String {
    format!(
        "{} {}",
        location.hive.to_ascii_uppercase(),
        if location.run_once { "RunOnce" } else { "Run" }
    )
}

fn is_system_command(command: &str) -> bool {
    let normalized = command.trim().trim_matches('"').to_ascii_lowercase();
    normalized.contains(r"c:\windows\")
        || normalized.contains(r"%systemroot%")
        || normalized.contains(r"\windows\system32\")
}
