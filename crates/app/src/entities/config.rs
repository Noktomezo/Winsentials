use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::entities::tweaks::input::SnapKeyPreset;
use crate::features::discord_rpc::DiscordRpcActivity;

const fn default_true() -> bool {
    true
}

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub minimize_to_tray: bool,
    #[serde(default)]
    pub autostart: bool,
    #[serde(default)]
    pub autostart_to_tray: bool,
    #[serde(default)]
    pub discord_rpc: DiscordRpcActivity,
    #[serde(default)]
    pub snapkey: SnapKeyPreset,
    #[serde(default = "default_true")]
    pub check_updates: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            minimize_to_tray: false,
            autostart: false,
            autostart_to_tray: false,
            discord_rpc: DiscordRpcActivity::default(),
            snapkey: SnapKeyPreset::default(),
            check_updates: true,
        }
    }
}

#[must_use]
pub fn get_config_path() -> PathBuf {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            let config_file = parent.join("config.toml");
            return config_file;
        }
    }
    PathBuf::from("config.toml")
}

#[must_use]
pub fn load_config() -> AppConfig {
    let path = get_config_path();
    if path.is_file() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(config) = toml::from_str::<AppConfig>(&content) {
                return config;
            }
        }
    }
    AppConfig::default()
}

pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let path = get_config_path();
    let content =
        toml::to_string_pretty(config).map_err(|e| format!("Failed to serialize config: {e}"))?;
    std::fs::write(&path, content)
        .map_err(|e| format!("Failed to write config to {}: {e}", path.display()))?;
    Ok(())
}
