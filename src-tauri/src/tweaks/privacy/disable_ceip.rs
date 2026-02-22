use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const SQMCLIENT_PATH: &str = r"SOFTWARE\Policies\Microsoft\SQMClient\Windows";
const DOTNET_PATH: &str =
  r"SOFTWARE\Policies\Microsoft\.NETFramework\v4.0.30319";
const CEIP_ENABLE: &str = "CEIPEnable";

pub struct DisableCEIPTweak {
  meta: TweakMeta,
}

impl DisableCEIPTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "disable_ceip".to_string(),
        category: TweakCategory::Privacy,
        name_key: "tweaks.disableCEIP.name".to_string(),
        description_key: "tweaks.disableCEIP.description".to_string(),
        details_key: "tweaks.disableCEIP.details".to_string(),
        ui_type: TweakUiType::Toggle,
        options: vec![],
        requires_reboot: false,
        risk_level: RiskLevel::Low,
        min_windows_build: None,
      },
    }
  }
}

impl Tweak for DisableCEIPTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let sqm_value =
      registry::read_reg_u32(HKEY_LOCAL_MACHINE, SQMCLIENT_PATH, CEIP_ENABLE);
    let dotnet_value =
      registry::read_reg_u32(HKEY_LOCAL_MACHINE, DOTNET_PATH, CEIP_ENABLE);
    let is_applied = sqm_value.map(|v| v == 0).unwrap_or(false)
      && dotnet_value.map(|v| v == 0).unwrap_or(false);
    Ok(TweakState {
      id: self.meta.id.clone(),
      current_value: Some(if is_applied { "1" } else { "0" }.to_string()),
      is_applied,
    })
  }

  fn apply(&self, _value: Option<&str>) -> Result<(), String> {
    registry::write_reg_u32(HKEY_LOCAL_MACHINE, SQMCLIENT_PATH, CEIP_ENABLE, 0)
      .map_err(|e| e.to_string())?;
    registry::write_reg_u32(HKEY_LOCAL_MACHINE, DOTNET_PATH, CEIP_ENABLE, 0)
      .map_err(|e| e.to_string())
  }

  fn revert(&self) -> Result<(), String> {
    registry::write_reg_u32(HKEY_LOCAL_MACHINE, SQMCLIENT_PATH, CEIP_ENABLE, 1)
      .map_err(|e| e.to_string())?;
    registry::write_reg_u32(HKEY_LOCAL_MACHINE, DOTNET_PATH, CEIP_ENABLE, 1)
      .map_err(|e| e.to_string())
  }
}
