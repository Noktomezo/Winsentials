use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const SESSION_MANAGER_PATH: &str =
  r"SYSTEM\CurrentControlSet\Control\Session Manager";
const HEAP_DECOMMIT_THRESHOLD: &str = "HeapDeCommitFreeBlockThreshold";

pub struct HeapDecommitTweak {
  meta: TweakMeta,
}

impl HeapDecommitTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "heap_decommit".to_string(),
        category: TweakCategory::Memory,
        name_key: "tweaks.heapDecommit.name".to_string(),
        description_key: "tweaks.heapDecommit.description".to_string(),
        details_key: "tweaks.heapDecommit.details".to_string(),
        ui_type: TweakUiType::Toggle,
        options: vec![],
        requires_reboot: false,
        risk_level: RiskLevel::Low,
        min_windows_build: None,
      },
    }
  }
}

impl Tweak for HeapDecommitTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let value = registry::read_reg_u32(
      HKEY_LOCAL_MACHINE,
      SESSION_MANAGER_PATH,
      HEAP_DECOMMIT_THRESHOLD,
    );
    let is_applied = value.map(|v| v == 262144).unwrap_or(false);
    Ok(TweakState {
      id: self.meta.id.clone(),
      current_value: Some(if is_applied { "1" } else { "0" }.to_string()),
      is_applied,
    })
  }

  fn apply(&self, _value: Option<&str>) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      SESSION_MANAGER_PATH,
      HEAP_DECOMMIT_THRESHOLD,
      262144,
    )
    .map_err(|e| e.to_string())
  }

  fn revert(&self) -> Result<(), String> {
    registry::delete_reg_value(
      HKEY_LOCAL_MACHINE,
      SESSION_MANAGER_PATH,
      HEAP_DECOMMIT_THRESHOLD,
    )
    .ok();
    Ok(())
  }
}
