use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const USER_PROFILE_PATH: &str = r"Control Panel\International\User Profile";
const HTTP_ACCEPT_LANGUAGE_OPT_OUT: &str = "HttpAcceptLanguageOptOut";

pub struct DisableLanguageOptOutTweak {
  meta: TweakMeta,
}

impl DisableLanguageOptOutTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "disable_language_opt_out".to_string(),
        category: TweakCategory::Privacy,
        name_key: "tweaks.disableLanguageOptOut.name".to_string(),
        description_key: "tweaks.disableLanguageOptOut.description".to_string(),
        details_key: "tweaks.disableLanguageOptOut.details".to_string(),
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

impl Tweak for DisableLanguageOptOutTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let value = registry::read_reg_u32(
      HKEY_CURRENT_USER,
      USER_PROFILE_PATH,
      HTTP_ACCEPT_LANGUAGE_OPT_OUT,
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
      USER_PROFILE_PATH,
      HTTP_ACCEPT_LANGUAGE_OPT_OUT,
      1,
    )
    .map_err(|e| e.to_string())
  }

  fn revert(&self) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_CURRENT_USER,
      USER_PROFILE_PATH,
      HTTP_ACCEPT_LANGUAGE_OPT_OUT,
      0,
    )
    .map_err(|e| e.to_string())
  }
}
