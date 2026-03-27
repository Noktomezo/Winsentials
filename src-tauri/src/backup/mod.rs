use std::collections::HashMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::commands::tweaks::tweak_apply;
use crate::error::AppError;
use crate::tweaks::all_tweaks;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupMeta {
    pub filename: String,
    pub label: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreReport {
    pub applied: u32,
    pub failed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupEntry {
    pub filename: String,
    pub label: String,
    pub created_at: String,
    pub tweaks: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BackupSnapshot {
    created_at: String,
    label: String,
    tweaks: HashMap<String, String>,
}

fn backups_dir() -> Result<PathBuf, AppError> {
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .ok_or_else(|| AppError::message("failed to resolve user home directory"))?;
    let dir = PathBuf::from(home).join(".winsentials").join("backups");
    fs::create_dir_all(&dir)
        .map_err(|error| AppError::message(format!("failed to create backups dir: {error}")))?;
    Ok(dir)
}

fn utc_now_iso8601() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let millis = now.subsec_millis();
    let sec = secs % 60;
    let mins = secs / 60;
    let min = mins % 60;
    let hours = mins / 60;
    let hour = hours % 24;
    let days = hours / 24;
    let (year, month, day) = days_to_date(days);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{min:02}:{sec:02}.{millis:03}Z")
}

fn validate_backup_filename(filename: &str) -> Result<&str, AppError> {
    let path = Path::new(filename);
    let is_plain_basename = path
        .components()
        .all(|component| matches!(component, Component::Normal(_)))
        && path.file_name().and_then(|value| value.to_str()) == Some(filename);

    if !is_plain_basename || filename.is_empty() {
        return Err(AppError::message("invalid backup filename"));
    }

    Ok(filename)
}

fn collect_tweak_snapshot() -> HashMap<String, String> {
    let mut tweaks_map: HashMap<String, String> = HashMap::new();
    for tweak in all_tweaks() {
        if let Ok(status) = tweak.get_status() {
            tweaks_map.insert(tweak.id().to_string(), status.current_value);
        }
    }
    tweaks_map
}

fn write_backup_snapshot(
    filename: &str,
    label: String,
    created_at: Option<String>,
) -> Result<BackupEntry, AppError> {
    let dir = backups_dir()?;
    let filename = validate_backup_filename(filename)?;
    let created_at = created_at.unwrap_or_else(utc_now_iso8601);
    let tweaks_map = collect_tweak_snapshot();

    let snapshot = BackupSnapshot {
        created_at: created_at.clone(),
        label: label.clone(),
        tweaks: tweaks_map.clone(),
    };

    let json = serde_json::to_string_pretty(&snapshot)?;
    fs::write(dir.join(filename), json)?;

    Ok(BackupEntry {
        filename: filename.to_string(),
        label,
        created_at,
        tweaks: tweaks_map,
    })
}

/// Howard Hinnant's civil_from_days algorithm.
fn days_to_date(days: u64) -> (u64, u64, u64) {
    let z = days as i64 + 719_468_i64;
    let era = (if z >= 0 { z } else { z - 146_096 }) / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y } as u64;
    (y, m, d)
}

#[tauri::command]
pub fn backup_create(label: Option<String>) -> Result<BackupEntry, AppError> {
    let created_at = utc_now_iso8601();
    // Replace `:` with `-` to make a valid filename on Windows.
    let filename = format!("{}.json", created_at.replace(':', "-"));
    let label = label.unwrap_or_else(|| "Backup".to_string());

    write_backup_snapshot(&filename, label, Some(created_at))
}

#[tauri::command]
pub fn backup_list() -> Result<Vec<BackupEntry>, AppError> {
    let dir = backups_dir()?;

    if !dir.exists() {
        return Ok(vec![]);
    }

    let mut entries: Vec<BackupEntry> = Vec::new();

    for entry in fs::read_dir(&dir)?.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Some(filename) = path.file_name().and_then(|n| n.to_str()).map(str::to_owned) else {
            continue;
        };
        if let Ok(content) = fs::read_to_string(&path)
            && let Ok(snapshot) = serde_json::from_str::<BackupSnapshot>(&content)
        {
            entries.push(BackupEntry {
                filename,
                label: snapshot.label,
                created_at: snapshot.created_at,
                tweaks: snapshot.tweaks,
            });
        }
    }

    entries.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(entries)
}

#[tauri::command]
pub fn backup_restore(filename: String) -> Result<RestoreReport, AppError> {
    let path = backups_dir()?.join(validate_backup_filename(&filename)?);
    let content = fs::read_to_string(&path)?;
    let snapshot: BackupSnapshot = serde_json::from_str(&content)?;

    let mut applied: u32 = 0;
    let mut failed: Vec<String> = Vec::new();

    for (id, value) in &snapshot.tweaks {
        match tweak_apply(id.clone(), value.clone()) {
            Ok(_) => applied += 1,
            Err(_) => failed.push(id.clone()),
        }
    }

    Ok(RestoreReport { applied, failed })
}

#[tauri::command]
pub fn backup_rename(filename: String, new_label: String) -> Result<(), AppError> {
    let path = backups_dir()?.join(validate_backup_filename(&filename)?);
    let content = fs::read_to_string(&path)?;
    let mut snapshot: BackupSnapshot = serde_json::from_str(&content)?;
    snapshot.label = new_label;
    let json = serde_json::to_string_pretty(&snapshot)?;
    fs::write(&path, json)?;
    Ok(())
}

#[tauri::command]
pub fn backup_delete(filename: String) -> Result<(), AppError> {
    let path = backups_dir()?.join(validate_backup_filename(&filename)?);
    fs::remove_file(&path)?;
    Ok(())
}

/// Called on app startup. Creates an initial backup if the backups folder
/// does not exist or is empty.
pub fn ensure_initial_backup() {
    let Ok(dir) = backups_dir() else { return };

    if !dir.join("initial.json").exists() {
        let _ = write_backup_snapshot("initial.json", "Initial".to_string(), None);
    }
}
