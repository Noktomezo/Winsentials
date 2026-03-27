pub mod appearance;
pub mod behaviour;
pub mod network;
pub mod performance;
pub mod privacy;
pub mod security;
pub mod system;

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

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
pub struct TweakConflict {
    pub description: String,
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
    pub conflicts: Option<Vec<TweakConflict>>,
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

fn build_tweak_registry() -> Vec<Arc<dyn Tweak>> {
    let mut tweaks: Vec<Arc<dyn Tweak>> = Vec::new();
    tweaks.extend(appearance::tweaks().into_iter().map(Arc::from));
    tweaks.extend(behaviour::tweaks().into_iter().map(Arc::from));
    tweaks.extend(network::tweaks().into_iter().map(Arc::from));
    tweaks.extend(performance::tweaks().into_iter().map(Arc::from));
    tweaks.extend(privacy::tweaks().into_iter().map(Arc::from));
    tweaks.extend(security::tweaks().into_iter().map(Arc::from));
    tweaks.extend(system::tweaks().into_iter().map(Arc::from));
    tweaks
}

fn tweak_registry() -> &'static Vec<Arc<dyn Tweak>> {
    static TWEAK_REGISTRY: OnceLock<Vec<Arc<dyn Tweak>>> = OnceLock::new();
    TWEAK_REGISTRY.get_or_init(build_tweak_registry)
}

fn category_registry() -> &'static HashMap<String, Vec<Arc<dyn Tweak>>> {
    static CATEGORY_REGISTRY: OnceLock<HashMap<String, Vec<Arc<dyn Tweak>>>> = OnceLock::new();
    CATEGORY_REGISTRY.get_or_init(|| {
        let mut categories: HashMap<String, Vec<Arc<dyn Tweak>>> = HashMap::new();
        for tweak in tweak_registry() {
            categories
                .entry(tweak.meta().category.clone())
                .or_default()
                .push(Arc::clone(tweak));
        }
        categories
    })
}

fn id_registry() -> &'static HashMap<String, Arc<dyn Tweak>> {
    static ID_REGISTRY: OnceLock<HashMap<String, Arc<dyn Tweak>>> = OnceLock::new();
    ID_REGISTRY.get_or_init(|| {
        tweak_registry()
            .iter()
            .map(|tweak| (tweak.id().to_string(), Arc::clone(tweak)))
            .collect()
    })
}

pub fn tweaks_for_category(category: &str) -> Vec<Arc<dyn Tweak>> {
    category_registry()
        .get(category)
        .cloned()
        .unwrap_or_default()
}

pub fn all_tweaks() -> &'static Vec<Arc<dyn Tweak>> {
    tweak_registry()
}

pub fn tweak_by_id(id: &str) -> Result<Arc<dyn Tweak>, AppError> {
    id_registry()
        .get(id)
        .cloned()
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
