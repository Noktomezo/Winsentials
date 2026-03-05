use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use sysinfo::System;
use winreg::enums::*;

const CONTROL_PATH: &str = r"SYSTEM\CurrentControlSet\Control";
const SVCHOST_SPLIT_THRESHOLD: &str = "SvcHostSplitThresholdInKB";

pub struct SvcHostSplitThresholdTweak {
  meta: TweakMeta,
}

impl SvcHostSplitThresholdTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "svchost_split_threshold".to_string(),
        category: TweakCategory::Memory,
        name_key: "tweaks.svchostSplitThreshold.name".to_string(),
        description_key: "tweaks.svchostSplitThreshold.description".to_string(),
        details_key: "tweaks.svchostSplitThreshold.details".to_string(),
        risk_details_key: None,
        ui_type: TweakUiType::Toggle,
        options: vec![],
        requires_reboot: true,
        requires_logout: false,
        risk_level: RiskLevel::Low,
        min_windows_build: None,
      },
    }
  }
}

impl Tweak for SvcHostSplitThresholdTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let value = registry::read_reg_u32(
      HKEY_LOCAL_MACHINE,
      CONTROL_PATH,
      SVCHOST_SPLIT_THRESHOLD,
    );
    let is_applied = value.map(|v| v > 3670016).unwrap_or(false);
    Ok(TweakState {
      id: self.meta.id.clone(),
      current_value: Some(if is_applied { "1" } else { "0" }.to_string()),
      is_applied,
    })
  }

  fn apply(&self, _value: Option<&str>) -> Result<(), String> {
    let mut sys = System::new_all();
    sys.refresh_memory();
    let total_memory_kb = sys.total_memory() / 1024 + 1024000;
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      CONTROL_PATH,
      SVCHOST_SPLIT_THRESHOLD,
      total_memory_kb as u32,
    )
    .map_err(|e| e.to_string())
  }

  fn revert(&self) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      CONTROL_PATH,
      SVCHOST_SPLIT_THRESHOLD,
      3670016,
    )
    .map_err(|e| e.to_string())
  }
}
