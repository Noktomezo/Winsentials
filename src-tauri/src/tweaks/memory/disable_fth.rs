use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const FTH_PATH: &str = r"SOFTWARE\Microsoft\FTH";
const ENABLED: &str = "Enabled";

pub struct DisableFthTweak {
  meta: TweakMeta,
}

impl DisableFthTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "disable_fth".to_string(),
        category: TweakCategory::Memory,
        name_key: "tweaks.disableFth.name".to_string(),
        description_key: "tweaks.disableFth.description".to_string(),
        details_key: "tweaks.disableFth.details".to_string(),
        risk_details_key: None,
        ui_type: TweakUiType::Toggle,
        options: vec![],
        requires_reboot: false,
        requires_logout: false,
        risk_level: RiskLevel::Low,
        min_windows_build: None,
      },
    }
  }
}

impl Tweak for DisableFthTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let value = registry::read_reg_u32(HKEY_LOCAL_MACHINE, FTH_PATH, ENABLED);
    let is_applied = value.map(|v| v == 0).unwrap_or(false);
    Ok(TweakState {
      id: self.meta.id.clone(),
      current_value: Some(if is_applied { "1" } else { "0" }.to_string()),
      is_applied,
    })
  }

  fn apply(&self, _value: Option<&str>) -> Result<(), String> {
    registry::write_reg_u32(HKEY_LOCAL_MACHINE, FTH_PATH, ENABLED, 0)
      .map_err(|e| e.to_string())
  }

  fn revert(&self) -> Result<(), String> {
    registry::delete_reg_value(HKEY_LOCAL_MACHINE, FTH_PATH, ENABLED).ok();
    Ok(())
  }
}
