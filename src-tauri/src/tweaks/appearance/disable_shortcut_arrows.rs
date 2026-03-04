use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const REG_PATH: &str =
  r"Software\Microsoft\Windows\CurrentVersion\Explorer\Shell Icons";
const REG_KEY: &str = "29";
const DISABLED_VALUE: &str = r"%SystemRoot%\System32\shell32.dll,-50";

pub struct DisableShortcutArrowsTweak {
  meta: TweakMeta,
}

impl DisableShortcutArrowsTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "disableShortcutArrows".to_string(),
        category: TweakCategory::Appearance,
        name_key: "tweaks.disableShortcutArrows.name".to_string(),
        description_key: "tweaks.disableShortcutArrows.description".to_string(),
        details_key: "tweaks.disableShortcutArrows.details".to_string(),
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

impl Tweak for DisableShortcutArrowsTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let value =
      registry::read_reg_string(HKEY_LOCAL_MACHINE, REG_PATH, REG_KEY);

    let is_applied = value.as_deref() == Some(DISABLED_VALUE);

    Ok(TweakState {
      id: self.meta.id.clone(),
      current_value: Some(if is_applied { "1" } else { "0" }.to_string()),
      is_applied,
    })
  }

  fn apply(&self, _value: Option<&str>) -> Result<(), String> {
    registry::write_reg_string(
      HKEY_LOCAL_MACHINE,
      REG_PATH,
      REG_KEY,
      DISABLED_VALUE,
    )
    .map_err(|e| format!("Failed to disable shortcut arrows: {e}"))?;

    registry::restart_explorer();

    Ok(())
  }

  fn revert(&self) -> Result<(), String> {
    registry::delete_reg_value(HKEY_LOCAL_MACHINE, REG_PATH, REG_KEY).ok();

    registry::restart_explorer();

    Ok(())
  }
}
