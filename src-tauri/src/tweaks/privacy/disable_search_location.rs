use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const WINDOWS_SEARCH_PATH: &str =
  r"SOFTWARE\Policies\Microsoft\Windows\Windows Search";
const ALLOW_SEARCH_LOCATION: &str = "AllowSearchToUseLocation";

pub struct DisableSearchLocationTweak {
  meta: TweakMeta,
}

impl DisableSearchLocationTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "disable_search_location".to_string(),
        category: TweakCategory::Privacy,
        name_key: "tweaks.disableSearchLocation.name".to_string(),
        description_key: "tweaks.disableSearchLocation.description".to_string(),
        details_key: "tweaks.disableSearchLocation.details".to_string(),
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

impl Tweak for DisableSearchLocationTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let value = registry::read_reg_u32(
      HKEY_LOCAL_MACHINE,
      WINDOWS_SEARCH_PATH,
      ALLOW_SEARCH_LOCATION,
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
      WINDOWS_SEARCH_PATH,
      ALLOW_SEARCH_LOCATION,
      0,
    )
    .map_err(|e| e.to_string())
  }

  fn revert(&self) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      WINDOWS_SEARCH_PATH,
      ALLOW_SEARCH_LOCATION,
      1,
    )
    .map_err(|e| e.to_string())
  }
}
