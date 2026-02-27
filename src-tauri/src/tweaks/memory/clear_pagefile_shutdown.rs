use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const MEMORY_MGMT_PATH: &str =
  r"SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management";
const CLEAR_PAGEFILE: &str = "ClearPageFileAtShutdown";

pub struct ClearPageFileShutdownTweak {
  meta: TweakMeta,
}

impl ClearPageFileShutdownTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "clear_pagefile_shutdown".to_string(),
        category: TweakCategory::Memory,
        name_key: "tweaks.clearPageFileShutdown.name".to_string(),
        description_key: "tweaks.clearPageFileShutdown.description".to_string(),
        details_key: "tweaks.clearPageFileShutdown.details".to_string(),
        ui_type: TweakUiType::Toggle,
        options: vec![],
        requires_reboot: true,
        requires_logout: false,
        risk_level: RiskLevel::Medium,
        min_windows_build: None,
      },
    }
  }
}

impl Tweak for ClearPageFileShutdownTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let value = registry::read_reg_u32(
      HKEY_LOCAL_MACHINE,
      MEMORY_MGMT_PATH,
      CLEAR_PAGEFILE,
    );
    let is_applied = value.map(|v| v == 1).unwrap_or(false);
    Ok(TweakState {
      id: self.meta.id.clone(),
      current_value: Some(if is_applied { "1" } else { "0" }.to_string()),
      is_applied,
    })
  }

  fn apply(&self, _value: Option<&str>) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      MEMORY_MGMT_PATH,
      CLEAR_PAGEFILE,
      1,
    )
    .map_err(|e| e.to_string())
  }

  fn revert(&self) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      MEMORY_MGMT_PATH,
      CLEAR_PAGEFILE,
      0,
    )
    .map_err(|e| e.to_string())
  }
}
