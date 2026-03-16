pub mod appearance;
pub mod behaviour;
pub mod network;
pub mod performance;
pub mod privacy;
pub mod security;
pub mod system;

use serde::{Deserialize, Serialize};

use crate::error::AppError;
use crate::registry::{Hive, RegKey};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    None,
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweakOption {
    pub label: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TweakControlType {
    Toggle,
    Radio { options: Vec<TweakOption> },
    Dropdown { options: Vec<TweakOption> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RequiresAction {
    None,
    Logout,
    RestartPc,
    RestartService { service_name: String },
    RestartApp { app_name: String },
    RestartDevice { device_name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweakMeta {
    pub id: String,
    pub category: String,
    pub name: String,
    pub short_description: String,
    pub detail_description: String,
    pub control: TweakControlType,
    pub current_value: String,
    pub default_value: String,
    pub recommended_value: String,
    pub risk: RiskLevel,
    pub risk_description: Option<String>,
    pub requires_action: RequiresAction,
    pub min_os_build: Option<u32>,
    pub min_os_ubr: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweakResult {
    pub success: bool,
    pub current_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweakStatus {
    pub current_value: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowsVersion {
    pub build: u32,
    pub ubr: u32,
}

pub trait Tweak: Send + Sync {
    fn id(&self) -> &str;
    fn meta(&self) -> &TweakMeta;
    fn apply(&self, value: &str) -> Result<(), AppError>;
    fn reset(&self) -> Result<(), AppError>;
    fn get_status(&self) -> Result<TweakStatus, AppError>;
    fn extra(&self) -> Result<(), AppError> {
        Ok(())
    }
}

const WINDOWS_VERSION_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SOFTWARE\Microsoft\Windows NT\CurrentVersion",
};

pub fn all_tweaks() -> Vec<Box<dyn Tweak>> {
    let mut tweaks = Vec::new();
    tweaks.extend(appearance::tweaks());
    tweaks.extend(behaviour::tweaks());
    tweaks.extend(network::tweaks());
    tweaks.extend(performance::tweaks());
    tweaks.extend(privacy::tweaks());
    tweaks.extend(security::tweaks());
    tweaks.extend(system::tweaks());
    tweaks
}

pub fn tweaks_for_category(category: &str) -> Vec<Box<dyn Tweak>> {
    all_tweaks()
        .into_iter()
        .filter(|tweak| tweak.meta().category == category)
        .collect()
}

pub fn tweak_by_id(id: &str) -> Result<Box<dyn Tweak>, AppError> {
    all_tweaks()
        .into_iter()
        .find(|tweak| tweak.id() == id)
        .ok_or_else(|| AppError::message(format!("unknown tweak id: {id}")))
}

pub fn get_windows_build_number() -> Result<WindowsVersion, AppError> {
    let build = WINDOWS_VERSION_KEY.get_string("CurrentBuild")?;
    let ubr = WINDOWS_VERSION_KEY.get_dword("UBR")?;

    build
        .parse::<u32>()
        .map(|build| WindowsVersion { build, ubr })
        .map_err(|_| AppError::message(format!("invalid Windows build value: {build}")))
}
