use std::env;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::startup::types::StartupScope;

pub const HKCU_DISABLED_REGISTRY_PATH: &str =
    r"Software\Winsentials\StartupManager\DisabledRegistry";
pub const HKLM_DISABLED_REGISTRY_PATH: &str =
    r"Software\Winsentials\StartupManager\DisabledRegistry";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisabledRegistryRecord {
    pub id: String,
    pub original_hive: String,
    pub original_path: String,
    pub value_name: String,
    pub command: String,
    pub run_once: bool,
    pub scope: String,
    pub disabled_at: String,
    pub source_kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisabledStartupFileMetadata {
    pub id: String,
    pub original_path: String,
    pub disabled_file_path: String,
    pub scope: String,
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

pub fn decode_scope(scope: &str) -> StartupScope {
    if scope.eq_ignore_ascii_case("all_users") {
        StartupScope::AllUsers
    } else {
        StartupScope::CurrentUser
    }
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
        .unwrap_or_else(|| {
            log::warn!(
                "startup_sidecar_path fallback used because file_name() returned None for {}",
                disabled_file_path.display()
            );
            "startup-item"
        });

    disabled_file_path.with_file_name(format!("{file_name}.winsentials.json"))
}
