use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const NLASVC_INTERNET_PATH: &str =
  r"SYSTEM\CurrentControlSet\Services\NlaSvc\Parameters\Internet";
const ENABLE_ACTIVE_PROBING: &str = "EnableActiveProbing";

pub struct DisableNcsiTweak {
  meta: TweakMeta,
}

impl DisableNcsiTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "disable_ncsi".to_string(),
        category: TweakCategory::Network,
        name_key: "tweaks.disableNcsi.name".to_string(),
        description_key: "tweaks.disableNcsi.description".to_string(),
        details_key: "tweaks.disableNcsi.details".to_string(),
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

impl Tweak for DisableNcsiTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let value = registry::read_reg_u32(
      HKEY_LOCAL_MACHINE,
      NLASVC_INTERNET_PATH,
      ENABLE_ACTIVE_PROBING,
    );
    let is_applied = value.map(|v| v == 0).unwrap_or(false);

    Ok(TweakState {
      id: self.meta.id.clone(),
      current_value: Some(if is_applied { "1" } else { "0" }.to_string()),
      is_applied,
    })
  }

  fn apply(&self, _value: Option<&str>) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      NLASVC_INTERNET_PATH,
      ENABLE_ACTIVE_PROBING,
      0,
    )
    .map_err(|e| e.to_string())
  }

  fn revert(&self) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      NLASVC_INTERNET_PATH,
      ENABLE_ACTIVE_PROBING,
      1,
    )
    .map_err(|e| e.to_string())
  }
}
