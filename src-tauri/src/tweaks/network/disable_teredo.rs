use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const TCPIP6_PARAMS_PATH: &str =
  r"SYSTEM\CurrentControlSet\Services\Tcpip6\Parameters";
const DISABLED_COMPONENTS: &str = "DisabledComponents";
const TEREDO_DISABLED: u32 = 8;

pub struct DisableTeredoTweak {
  meta: TweakMeta,
}

impl DisableTeredoTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "disable_teredo".to_string(),
        category: TweakCategory::Network,
        name_key: "tweaks.disableTeredo.name".to_string(),
        description_key: "tweaks.disableTeredo.description".to_string(),
        details_key: "tweaks.disableTeredo.details".to_string(),
        ui_type: TweakUiType::Toggle,
        options: vec![],
        requires_reboot: false,
        risk_level: RiskLevel::Low,
        min_windows_build: None,
      },
    }
  }
}

impl Tweak for DisableTeredoTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let value = registry::read_reg_u32(
      HKEY_LOCAL_MACHINE,
      TCPIP6_PARAMS_PATH,
      DISABLED_COMPONENTS,
    );
    let is_applied = value.map(|v| v & TEREDO_DISABLED != 0).unwrap_or(false);

    Ok(TweakState {
      id: self.meta.id.clone(),
      current_value: Some(if is_applied { "1" } else { "0" }.to_string()),
      is_applied,
    })
  }

  fn apply(&self, _value: Option<&str>) -> Result<(), String> {
    let current = registry::read_reg_u32(
      HKEY_LOCAL_MACHINE,
      TCPIP6_PARAMS_PATH,
      DISABLED_COMPONENTS,
    )
    .unwrap_or(0);

    let new_value = current | TEREDO_DISABLED;

    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      TCPIP6_PARAMS_PATH,
      DISABLED_COMPONENTS,
      new_value,
    )
    .map_err(|e| e.to_string())
  }

  fn revert(&self) -> Result<(), String> {
    let current = registry::read_reg_u32(
      HKEY_LOCAL_MACHINE,
      TCPIP6_PARAMS_PATH,
      DISABLED_COMPONENTS,
    )
    .unwrap_or(0);

    let new_value = current & !TEREDO_DISABLED;

    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      TCPIP6_PARAMS_PATH,
      DISABLED_COMPONENTS,
      new_value,
    )
    .map_err(|e| e.to_string())
  }
}
