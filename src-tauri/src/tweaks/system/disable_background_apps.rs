use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const BG_APPS_PATH: &str =
  r"Software\Microsoft\Windows\CurrentVersion\BackgroundAccessApplications";
const APP_PRIVACY_PATH: &str =
  r"SOFTWARE\Policies\Microsoft\Windows\AppPrivacy";
const GLOBAL_USER_DISABLED: &str = "GlobalUserDisabled";
const LET_APPS_RUN_IN_BACKGROUND: &str = "LetAppsRunInBackground";

pub struct DisableBackgroundAppsTweak {
  meta: TweakMeta,
}

impl DisableBackgroundAppsTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "disable_background_apps".to_string(),
        category: TweakCategory::System,
        name_key: "tweaks.disableBackgroundApps.name".to_string(),
        description_key: "tweaks.disableBackgroundApps.description".to_string(),
        details_key: "tweaks.disableBackgroundApps.details".to_string(),
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

impl Tweak for DisableBackgroundAppsTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let value = registry::read_reg_u32(
      HKEY_CURRENT_USER,
      BG_APPS_PATH,
      GLOBAL_USER_DISABLED,
    );
    let is_applied = value.map(|v| v == 1).unwrap_or(false);
    Ok(TweakState {
      id: self.meta.id.clone(),
      current_value: Some(if is_applied { "1" } else { "0" }.to_string()),
      is_applied,
    })
  }

  fn apply(&self, _value: Option<&str>) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_CURRENT_USER,
      BG_APPS_PATH,
      GLOBAL_USER_DISABLED,
      1,
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      APP_PRIVACY_PATH,
      LET_APPS_RUN_IN_BACKGROUND,
      2,
    )
    .map_err(|e| e.to_string())
  }

  fn revert(&self) -> Result<(), String> {
    registry::delete_reg_value(
      HKEY_CURRENT_USER,
      BG_APPS_PATH,
      GLOBAL_USER_DISABLED,
    )
    .ok();
    registry::delete_reg_value(
      HKEY_LOCAL_MACHINE,
      APP_PRIVACY_PATH,
      LET_APPS_RUN_IN_BACKGROUND,
    )
    .ok();
    Ok(())
  }
}
