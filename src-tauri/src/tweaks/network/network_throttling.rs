use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const MULTIMEDIA_PROFILE_PATH: &str =
  r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile";
const NETWORK_THROTTLING_INDEX: &str = "NetworkThrottlingIndex";
const SYSTEM_RESPONSIVENESS: &str = "SystemResponsiveness";

pub struct NetworkThrottlingTweak {
  meta: TweakMeta,
}

impl NetworkThrottlingTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "network_throttling".to_string(),
        category: TweakCategory::Network,
        name_key: "tweaks.networkThrottling.name".to_string(),
        description_key: "tweaks.networkThrottling.description".to_string(),
        details_key: "tweaks.networkThrottling.details".to_string(),
        ui_type: TweakUiType::Toggle,
        options: vec![],
        requires_reboot: false,
        risk_level: RiskLevel::Low,
        min_windows_build: None,
      },
    }
  }
}

impl Tweak for NetworkThrottlingTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let value = registry::read_reg_u32(
      HKEY_LOCAL_MACHINE,
      MULTIMEDIA_PROFILE_PATH,
      NETWORK_THROTTLING_INDEX,
    );
    let is_applied = value.map(|v| v == 0xFFFFFFFF || v == 10).unwrap_or(false);
    Ok(TweakState {
      id: self.meta.id.clone(),
      current_value: Some(if is_applied { "1" } else { "0" }.to_string()),
      is_applied,
    })
  }

  fn apply(&self, _value: Option<&str>) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      MULTIMEDIA_PROFILE_PATH,
      NETWORK_THROTTLING_INDEX,
      0xFFFFFFFF,
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      MULTIMEDIA_PROFILE_PATH,
      SYSTEM_RESPONSIVENESS,
      0,
    )
    .map_err(|e| e.to_string())
  }

  fn revert(&self) -> Result<(), String> {
    registry::delete_reg_value(
      HKEY_LOCAL_MACHINE,
      MULTIMEDIA_PROFILE_PATH,
      NETWORK_THROTTLING_INDEX,
    )
    .ok();
    registry::delete_reg_value(
      HKEY_LOCAL_MACHINE,
      MULTIMEDIA_PROFILE_PATH,
      SYSTEM_RESPONSIVENESS,
    )
    .ok();
    Ok(())
  }
}
