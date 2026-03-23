use std::env;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::startup::types::StartupScope;

pub const HKCU_DISABLED_REGISTRY_PATH: &str =
    r"Software\Winsentials\StartupManager\DisabledRegistry";
pub const HKLM_DISABLED_REGISTRY_PATH: &str =
    r"Software\Winsentials\StartupManager\DisabledRegistryMachine";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisabledRegistryRecord {
    pub id: String,
    pub original_hive: String,
    pub original_path: String,
    pub value_name: String,
    pub command: String,
    pub run_once: bool,
    pub scope: StartupScope,
    pub disabled_at: String,
    pub source_kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisabledStartupFileMetadata {
    pub id: String,
    pub original_path: String,
    pub disabled_file_path: String,
    pub scope: StartupScope,
    pub disabled_at: String,
    pub source_kind: String,
}

pub fn hex_encode(value: &str) -> String {
    value
        .as_bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub fn startup_disabled_dir(scope: StartupScope) -> Result<PathBuf, AppError> {
    let base = match scope {
        StartupScope::CurrentUser => env::var("APPDATA")
            .map(PathBuf::from)
            .map_err(|_| AppError::message("APPDATA environment variable is not available"))?,
        StartupScope::AllUsers => env::var("ProgramData")
            .map(PathBuf::from)
            .map_err(|_| AppError::message("ProgramData environment variable is not available"))?,
    };

    Ok(base
        .join("Winsentials")
        .join("StartupManager")
        .join("DisabledStartup")
        .join(match scope {
            StartupScope::CurrentUser => "CurrentUser",
            StartupScope::AllUsers => "AllUsers",
        }))
}

pub fn startup_sidecar_path(disabled_file_path: &Path) -> PathBuf {
    let file_name = disabled_file_path
        .file_name()
        .and_then(|value| value.to_str())
        .map(str::to_string)
        .unwrap_or_else(|| {
            let fallback_id = startup_sidecar_fallback_id(disabled_file_path);
            let fallback_name = format!("startup-item-{fallback_id}");
            log::warn!(
                "startup_sidecar_path fallback used because file_name() returned None for {}; generated id: {}",
                disabled_file_path.display(),
                fallback_id
            );
            fallback_name
        });

    disabled_file_path.with_file_name(format!("{file_name}.winsentials.json"))
}

fn startup_sidecar_fallback_id(disabled_file_path: &Path) -> String {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    disabled_file_path.to_string_lossy().hash(&mut hasher);
    format!("{:08x}", hasher.finish())
}
