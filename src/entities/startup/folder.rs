use std::fs;
use std::path::{Path, PathBuf};

use super::types::{StartupEntry, StartupScope, StartupSource, StartupStatus};
use super::vendor::{extract_clean_exe_path, get_file_publisher};

pub fn scan_folder_startup() -> Vec<StartupEntry> {
    let mut entries = Vec::new();

    // 1. Current User Startup Folder
    if let Ok(appdata) = std::env::var("APPDATA") {
        let user_folder =
            PathBuf::from(appdata).join(r"Microsoft\Windows\Start Menu\Programs\Startup");
        scan_dir(&user_folder, StartupScope::CurrentUser, &mut entries);
    }

    // 2. All Users (Common) Startup Folder
    if let Ok(programdata) = std::env::var("PROGRAMDATA") {
        let common_folder =
            PathBuf::from(programdata).join(r"Microsoft\Windows\Start Menu\Programs\Startup");
        scan_dir(&common_folder, StartupScope::AllUsers, &mut entries);
    }

    entries
}

fn scan_dir(dir: &Path, scope: StartupScope, entries: &mut Vec<StartupEntry>) {
    if !dir.exists() {
        return;
    }

    let Ok(read_dir) = fs::read_dir(dir) else {
        return;
    };

    for entry in read_dir.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let Some(file_name) = path.file_name().and_then(|s| s.to_str()) else {
            continue;
        };

        if file_name.eq_ignore_ascii_case("desktop.ini") {
            continue;
        }

        let is_disabled = file_name.ends_with(".disabled");
        let clean_name = if is_disabled {
            file_name.trim_end_matches(".disabled")
        } else {
            file_name
        };

        let display_name = Path::new(clean_name)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(clean_name)
            .to_string();

        let path_str = path.to_string_lossy().to_string();
        let status = if is_disabled {
            StartupStatus::Disabled
        } else {
            StartupStatus::Enabled
        };

        let location_label = match scope {
            StartupScope::CurrentUser => "shell:startup".to_string(),
            StartupScope::AllUsers => "shell:common startup".to_string(),
        };

        // Try extracting target exe or publisher
        let target_exe = extract_clean_exe_path(&path_str);
        let publisher = target_exe.as_deref().and_then(get_file_publisher);

        entries.push(StartupEntry {
            id: format!(
                "folder_{}_{clean_name}",
                match scope {
                    StartupScope::CurrentUser => "user",
                    StartupScope::AllUsers => "common",
                }
            ),
            name: clean_name.to_string(),
            display_name,
            publisher,
            source: StartupSource::StartupFolder,
            scope,
            status,
            command: Some(path_str.clone()),
            target_path: Some(path_str.clone()),
            location_label,
            raw_id: path_str,
        });
    }
}

pub fn toggle_folder_entry(entry: &StartupEntry) -> bool {
    let current_path = PathBuf::from(&entry.raw_id);
    if !current_path.exists() {
        return false;
    }

    match entry.status {
        StartupStatus::Enabled => {
            let file_name = current_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            let disabled_name = format!("{file_name}.disabled");
            let new_path = current_path.with_file_name(disabled_name);
            fs::rename(&current_path, new_path).is_ok()
        }
        StartupStatus::Disabled => {
            let file_name = current_path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            let enabled_name = file_name.trim_end_matches(".disabled");
            let new_path = current_path.with_file_name(enabled_name);
            fs::rename(&current_path, new_path).is_ok()
        }
    }
}

pub fn delete_folder_entry(entry: &StartupEntry) -> bool {
    let path = PathBuf::from(&entry.raw_id);
    if path.exists() {
        fs::remove_file(path).is_ok()
    } else {
        false
    }
}
