use windows_registry::{CURRENT_USER, Key, LOCAL_MACHINE};

use super::types::{StartupEntry, StartupScope, StartupSource, StartupStatus};

const REG_RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const REG_RUN_ONCE: &str = r"Software\Microsoft\Windows\CurrentVersion\RunOnce";
const REG_DISABLED_BACKUP: &str = r"Software\Winsentials\DisabledStartup\Registry";

fn scan_key_values(
    root: &Key,
    key_path: &str,
    scope: StartupScope,
    prefix: &str,
    label: &str,
    entries: &mut Vec<StartupEntry>,
) {
    let Ok(key) = root.open(key_path) else {
        return;
    };
    let Ok(values) = key.values() else {
        return;
    };

    for (name, _) in values {
        if let Ok(cmd) = key.get_string(&name) {
            let id = format!("{prefix}_{name}");
            let display_name = extract_display_name(&name, &cmd);
            let target_path = extract_target_path(&cmd);
            entries.push(StartupEntry {
                id,
                name: name.clone(),
                display_name,
                source: StartupSource::Registry,
                scope,
                status: StartupStatus::Enabled,
                command: Some(cmd),
                target_path,
                location_label: label.to_string(),
                raw_id: format!("{prefix}|{key_path}|{name}"),
            });
        }
    }
}

pub fn scan_registry_startup() -> Vec<StartupEntry> {
    let mut entries = Vec::new();

    scan_key_values(
        CURRENT_USER,
        REG_RUN,
        StartupScope::CurrentUser,
        "HKCU",
        "HKCU\\...\\Run",
        &mut entries,
    );
    scan_key_values(
        LOCAL_MACHINE,
        REG_RUN,
        StartupScope::AllUsers,
        "HKLM",
        "HKLM\\...\\Run",
        &mut entries,
    );
    scan_key_values(
        CURRENT_USER,
        REG_RUN_ONCE,
        StartupScope::CurrentUser,
        "HKCU_ONCE",
        "HKCU\\...\\RunOnce",
        &mut entries,
    );

    // Disabled registry entries from backup store
    if let Ok(key) = CURRENT_USER.open(REG_DISABLED_BACKUP) {
        if let Ok(values) = key.values() {
            for (name, _) in values {
                if let Ok(stored_val) = key.get_string(&name) {
                    let parts: Vec<&str> = stored_val.splitn(4, '|').collect();
                    if parts.len() == 4 {
                        let hive = parts[0];
                        let orig_name = parts[2];
                        let cmd = parts[3];
                        let scope = if hive == "HKLM" {
                            StartupScope::AllUsers
                        } else {
                            StartupScope::CurrentUser
                        };
                        let id = format!("reg_disabled_{name}");
                        let display_name = extract_display_name(orig_name, cmd);
                        let target_path = extract_target_path(cmd);
                        entries.push(StartupEntry {
                            id,
                            name: orig_name.to_string(),
                            display_name,
                            source: StartupSource::Registry,
                            scope,
                            status: StartupStatus::Disabled,
                            command: Some(cmd.to_string()),
                            target_path,
                            location_label: format!("{hive}\\...\\Disabled"),
                            raw_id: format!("DISABLED|{name}|{stored_val}"),
                        });
                    }
                }
            }
        }
    }

    entries
}

pub fn toggle_registry_entry(entry: &StartupEntry) -> bool {
    match entry.status {
        StartupStatus::Enabled => {
            let parts: Vec<&str> = entry.raw_id.splitn(3, '|').collect();
            if parts.len() == 3 {
                let hive = parts[0];
                let key_path = parts[1];
                let val_name = parts[2];
                let cmd = entry.command.as_deref().unwrap_or("");

                if let Ok(backup_key) = CURRENT_USER.create(REG_DISABLED_BACKUP) {
                    let backup_name = format!("{hive}_{val_name}");
                    let backup_val = format!("{hive}|{key_path}|{val_name}|{cmd}");
                    let _ = backup_key.set_string(&backup_name, &backup_val);
                }

                if hive.starts_with("HKCU") {
                    if let Ok(key) = CURRENT_USER.open(key_path) {
                        let _ = key.remove_value(val_name);
                    }
                } else if hive == "HKLM" {
                    if let Ok(key) = LOCAL_MACHINE.open(key_path) {
                        let _ = key.remove_value(val_name);
                    }
                }
                return true;
            }
            false
        }
        StartupStatus::Disabled => {
            let parts: Vec<&str> = entry.raw_id.splitn(3, '|').collect();
            if parts.len() == 3 && parts[0] == "DISABLED" {
                let backup_name = parts[1];
                let stored_val = parts[2];
                let val_parts: Vec<&str> = stored_val.splitn(4, '|').collect();
                if val_parts.len() == 4 {
                    let hive = val_parts[0];
                    let key_path = val_parts[1];
                    let orig_name = val_parts[2];
                    let cmd = val_parts[3];

                    if hive.starts_with("HKCU") {
                        if let Ok(key) = CURRENT_USER.create(key_path) {
                            let _ = key.set_string(orig_name, cmd);
                        }
                    } else if hive == "HKLM" {
                        if let Ok(key) = LOCAL_MACHINE.create(key_path) {
                            let _ = key.set_string(orig_name, cmd);
                        }
                    }

                    if let Ok(backup_key) = CURRENT_USER.open(REG_DISABLED_BACKUP) {
                        let _ = backup_key.remove_value(backup_name);
                    }
                    return true;
                }
            }
            false
        }
    }
}

pub fn delete_registry_entry(entry: &StartupEntry) -> bool {
    let parts: Vec<&str> = entry.raw_id.splitn(3, '|').collect();
    if parts.len() == 3 {
        if parts[0] == "DISABLED" {
            let backup_name = parts[1];
            if let Ok(backup_key) = CURRENT_USER.open(REG_DISABLED_BACKUP) {
                return backup_key.remove_value(backup_name).is_ok();
            }
        } else {
            let hive = parts[0];
            let key_path = parts[1];
            let val_name = parts[2];
            if hive.starts_with("HKCU") {
                if let Ok(key) = CURRENT_USER.open(key_path) {
                    return key.remove_value(val_name).is_ok();
                }
            } else if hive == "HKLM" {
                if let Ok(key) = LOCAL_MACHINE.open(key_path) {
                    return key.remove_value(val_name).is_ok();
                }
            }
        }
    }
    false
}

pub fn extract_target_path(cmd: &str) -> Option<String> {
    let trimmed = cmd.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(stripped) = trimmed.strip_prefix('"') {
        if let Some(end_idx) = stripped.find('"') {
            return Some(stripped[..end_idx].to_string());
        }
    }

    if let Some(space_idx) = trimmed.find(' ') {
        Some(trimmed[..space_idx].to_string())
    } else {
        Some(trimmed.to_string())
    }
}

pub fn extract_display_name(name: &str, cmd: &str) -> String {
    let trimmed_name = name.trim();
    if !trimmed_name.is_empty() {
        return trimmed_name.to_string();
    }

    if let Some(path) = extract_target_path(cmd) {
        let p = std::path::Path::new(&path);
        if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
            return stem.to_string();
        }
    }

    "Startup Program".to_string()
}
