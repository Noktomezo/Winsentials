use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const PREFETCH_PATH: &str = r"SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management\PrefetchParameters";
const ENABLE_SUPERFETCH: &str = "EnableSuperfetch";

pub struct DisableSuperfetchTweak {
  meta: TweakMeta,
}

impl DisableSuperfetchTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "disable_superfetch".to_string(),
        category: TweakCategory::System,
        name_key: "tweaks.disableSuperfetch.name".to_string(),
        description_key: "tweaks.disableSuperfetch.description".to_string(),
        details_key: "tweaks.disableSuperfetch.details".to_string(),
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

impl Tweak for DisableSuperfetchTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let value = registry::read_reg_u32(
      HKEY_LOCAL_MACHINE,
      PREFETCH_PATH,
      ENABLE_SUPERFETCH,
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
      PREFETCH_PATH,
      ENABLE_SUPERFETCH,
      0,
    )
    .map_err(|e| e.to_string())
  }

  fn revert(&self) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      PREFETCH_PATH,
      ENABLE_SUPERFETCH,
      3,
    )
    .map_err(|e| e.to_string())
  }
}
