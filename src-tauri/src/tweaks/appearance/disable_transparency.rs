use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const REG_PATH: &str =
  r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";
const REG_VALUE: &str = "EnableTransparency";

pub struct DisableTransparencyTweak {
  meta: TweakMeta,
}

impl DisableTransparencyTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "disableTransparency".to_string(),
        category: TweakCategory::Appearance,
        name_key: "tweaks.disableTransparency.name".to_string(),
        description_key: "tweaks.disableTransparency.description".to_string(),
        details_key: "tweaks.disableTransparency.details".to_string(),
        ui_type: TweakUiType::Toggle,
        options: vec![],
        requires_reboot: false,
        risk_level: RiskLevel::Low,
        min_windows_build: None,
      },
    }
  }
}

impl Tweak for DisableTransparencyTweak {
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
      .map_err(|e| format!("Failed to disable transparency: {e}"))?;

    Ok(())
  }

  fn revert(&self) -> Result<(), String> {
    registry::write_reg_u32(HKEY_CURRENT_USER, REG_PATH, REG_VALUE, 1)
      .map_err(|e| format!("Failed to enable transparency: {e}"))?;

    Ok(())
  }
}
