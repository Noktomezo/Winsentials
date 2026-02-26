use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const REG_PATH: &str =
  r"Software\Classes\CLSID\{f874310e-b6b7-47dc-bc84-b9e6b38f5903}";
const REG_VALUE: &str = "System.IsPinnedToNameSpaceTree";
const WIN11_BUILD: u32 = 22000;

pub struct HideExplorerHomeTweak {
  meta: TweakMeta,
}

impl HideExplorerHomeTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "hideExplorerHome".to_string(),
        category: TweakCategory::Appearance,
        name_key: "tweaks.hideExplorerHome.name".to_string(),
        description_key: "tweaks.hideExplorerHome.description".to_string(),
        details_key: "tweaks.hideExplorerHome.details".to_string(),
        ui_type: TweakUiType::Toggle,
        options: vec![],
        requires_reboot: false,
        risk_level: RiskLevel::Low,
        min_windows_build: Some(WIN11_BUILD),
      },
    }
  }
}

impl Tweak for HideExplorerHomeTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let value = registry::read_reg_u32(HKEY_CURRENT_USER, REG_PATH, REG_VALUE);

    let is_applied = value.map(|v| v == 0).unwrap_or(false);

    Ok(TweakState {
      id: self.meta.id.clone(),
      current_value: Some(if is_applied { "1" } else { "0" }.to_string()),
      is_applied,
    })
  }

  fn apply(&self, _value: Option<&str>) -> Result<(), String> {
    registry::write_reg_u32(HKEY_CURRENT_USER, REG_PATH, REG_VALUE, 0)
      .map_err(|e| format!("Failed to hide Home button: {e}"))?;

    registry::restart_explorer();

    Ok(())
  }

  fn revert(&self) -> Result<(), String> {
    registry::write_reg_u32(HKEY_CURRENT_USER, REG_PATH, REG_VALUE, 1)
      .map_err(|e| format!("Failed to show Home button: {e}"))?;

    registry::restart_explorer();

    Ok(())
  }
}
