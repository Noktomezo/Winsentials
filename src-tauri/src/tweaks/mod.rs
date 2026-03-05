pub mod appearance;
pub mod hardware;
pub mod input;
pub mod memory;
pub mod network;
pub mod privacy;
pub mod registry;
pub mod security;
pub mod system;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum TweakCategory {
  #[default]
  System,
  Appearance,
  Privacy,
  Network,
  Input,
  Security,
  Hardware,
  Memory,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum TweakUiType {
  #[default]
  Toggle,
  Radio,
  Dropdown,
  Multiselect,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
  #[default]
  Low,
  Medium,
  High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweakOption {
  pub value: String,
  pub label_key: String,
  #[serde(default)]
  pub is_default: bool,
  #[serde(default)]
  pub is_recommended: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweakMeta {
  pub id: String,
  pub category: TweakCategory,
  pub name_key: String,
  pub description_key: String,
  pub details_key: String,
  pub risk_details_key: Option<String>,
  pub ui_type: TweakUiType,
  #[serde(default)]
  pub options: Vec<TweakOption>,
  #[serde(default)]
  pub requires_reboot: bool,
  #[serde(default)]
  pub requires_logout: bool,
  #[serde(default)]
  pub risk_level: RiskLevel,
  #[serde(default)]
  pub min_windows_build: Option<u32>,
}

impl Default for TweakMeta {
  fn default() -> Self {
    Self {
      id: String::new(),
      category: TweakCategory::System,
      name_key: String::new(),
      description_key: String::new(),
      details_key: String::new(),
      risk_details_key: None,
      ui_type: TweakUiType::Toggle,
      options: vec![],
      requires_reboot: false,
      requires_logout: false,
      risk_level: RiskLevel::Low,
      min_windows_build: None,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweakState {
  pub id: String,
  pub current_value: Option<String>,
  pub is_applied: bool,
}

pub trait Tweak: Send + Sync {
  fn meta(&self) -> &TweakMeta;
  fn check(&self) -> Result<TweakState, String>;
  fn apply(&self, value: Option<&str>) -> Result<(), String>;
  fn revert(&self) -> Result<(), String>;
}

pub fn get_all_tweaks() -> Vec<Box<dyn Tweak>> {
  let mut tweaks: Vec<Box<dyn Tweak>> = vec![];
  tweaks.extend(system::get_tweaks());
  tweaks.extend(appearance::get_tweaks());
  tweaks.extend(privacy::get_tweaks());
  tweaks.extend(network::get_tweaks());
  tweaks.extend(input::get_tweaks());
  tweaks.extend(security::get_tweaks());
  tweaks.extend(hardware::get_tweaks());
  tweaks.extend(memory::get_tweaks());
  tweaks
}

pub fn get_tweak_by_id(id: &str) -> Option<Box<dyn Tweak>> {
  get_all_tweaks().into_iter().find(|t| t.meta().id == id)
}
