use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const DEVICE_GUARD_PATH: &str = r"SYSTEM\CurrentControlSet\Control\DeviceGuard\Scenarios\HypervisorEnforcedCodeIntegrity";
const ENABLED: &str = "Enabled";

pub struct CoreIsolationTweak {
  meta: TweakMeta,
}

impl CoreIsolationTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "core_isolation".to_string(),
        category: TweakCategory::Security,
        name_key: "tweaks.coreIsolation.name".to_string(),
        description_key: "tweaks.coreIsolation.description".to_string(),
        details_key: "tweaks.coreIsolation.details".to_string(),
        ui_type: TweakUiType::Toggle,
        options: vec![],
        requires_reboot: true,
        risk_level: RiskLevel::High,
        min_windows_build: None,
      },
    }
  }
}

impl Tweak for CoreIsolationTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let value =
      registry::read_reg_u32(HKEY_LOCAL_MACHINE, DEVICE_GUARD_PATH, ENABLED);
    let is_applied = value.map(|v| v == 0).unwrap_or(true);
    Ok(TweakState {
      id: self.meta.id.clone(),
      current_value: Some(if is_applied { "1" } else { "0" }.to_string()),
      is_applied,
    })
  }

  fn apply(&self, _value: Option<&str>) -> Result<(), String> {
    registry::write_reg_u32(HKEY_LOCAL_MACHINE, DEVICE_GUARD_PATH, ENABLED, 0)
      .map_err(|e| e.to_string())
  }

  fn revert(&self) -> Result<(), String> {
    registry::write_reg_u32(HKEY_LOCAL_MACHINE, DEVICE_GUARD_PATH, ENABLED, 1)
      .map_err(|e| e.to_string())
  }
}
