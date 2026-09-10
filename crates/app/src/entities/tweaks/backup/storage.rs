use std::collections::HashMap;
use std::path::PathBuf;

use super::time::format_local_time_now;
use super::types::TweakBackup;
use crate::entities::tweaks::ALL_TWEAKS;

#[must_use]
pub fn get_backups_path() -> PathBuf {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            let primary = parent.join("backups.json");
            if primary.exists() {
                return primary;
            }
            // Test if parent is writable
            let test_file = parent.join(".test_write");
            if std::fs::write(&test_file, b"").is_ok() {
                let _ = std::fs::remove_file(&test_file);
                return primary;
            }
        }
    }
    if let Some(local_appdata) = std::env::var_os("LOCALAPPDATA") {
        let dir = PathBuf::from(local_appdata).join("Winsentials");
        let _ = std::fs::create_dir_all(&dir);
        return dir.join("backups.json");
    }
    PathBuf::from("backups.json")
}

#[must_use]
pub fn load_backups() -> Vec<TweakBackup> {
    let path = get_backups_path();
    let mut backups: Vec<TweakBackup> = if path.is_file() {
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|data| serde_json::from_str(&data).ok())
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    let mut migrated = false;
    for backup in &mut backups {
        for tweak in ALL_TWEAKS {
            if !backup.tweak_states.contains_key(tweak.id) {
                backup.tweak_states.insert(tweak.id.to_string(), false);
                migrated = true;
            }
        }
    }

    if migrated {
        let _ = save_backups(&backups);
    }

    backups.sort_by_key(|a| std::cmp::Reverse(a.timestamp_epoch_secs));
    backups
}

pub fn save_backups(backups: &[TweakBackup]) -> Result<(), String> {
    let path = get_backups_path();
    let json = serde_json::to_string_pretty(backups)
        .map_err(|e| format!("Failed to serialize backups: {e}"))?;

    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    std::fs::write(&path, json)
        .map_err(|e| format!("Failed to write backups to {}: {e}", path.display()))
}

#[must_use]
pub fn create_backup_from_states(
    name: &str,
    current_states: &HashMap<&'static str, bool>,
) -> TweakBackup {
    let id = uuid::Uuid::new_v4().to_string();
    let (created_at, timestamp_epoch_secs) = format_local_time_now();

    let mut tweak_states = HashMap::with_capacity(ALL_TWEAKS.len());
    for tweak in ALL_TWEAKS {
        let applied = current_states
            .get(tweak.id)
            .copied()
            .unwrap_or_else(tweak.is_applied);
        tweak_states.insert(tweak.id.to_string(), applied);
    }

    TweakBackup::new(id, name, created_at, timestamp_epoch_secs, tweak_states)
}

pub fn delete_backup(id: &str, backups: &mut Vec<TweakBackup>) -> bool {
    let initial_len = backups.len();
    backups.retain(|b| b.id != id);
    let changed = backups.len() != initial_len;
    if changed {
        let _ = save_backups(backups);
    }
    changed
}

pub fn rename_backup(id: &str, new_name: &str, backups: &mut [TweakBackup]) -> bool {
    if let Some(backup) = backups.iter_mut().find(|b| b.id == id) {
        backup.name = new_name.trim().to_string();
        let _ = save_backups(backups);
        true
    } else {
        false
    }
}
