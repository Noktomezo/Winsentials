use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const CONTENT_DELIVERY_PATH: &str =
  r"Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager";
const SYSTEM_PANE_SUGGESTIONS: &str = "SystemPaneSuggestionsEnabled";
const SOFT_LANDING: &str = "SoftLandingEnabled";
const SUBSCRIBED_CONTENT: &str = "SubscribedContentEnabled";

pub struct DisableSuggestionsTweak {
  meta: TweakMeta,
}

impl DisableSuggestionsTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "disable_suggestions".to_string(),
        category: TweakCategory::Privacy,
        name_key: "tweaks.disableSuggestions.name".to_string(),
        description_key: "tweaks.disableSuggestions.description".to_string(),
        details_key: "tweaks.disableSuggestions.details".to_string(),
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

impl Tweak for DisableSuggestionsTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let value = registry::read_reg_u32(
      HKEY_CURRENT_USER,
      CONTENT_DELIVERY_PATH,
      SYSTEM_PANE_SUGGESTIONS,
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
      HKEY_CURRENT_USER,
      CONTENT_DELIVERY_PATH,
      SYSTEM_PANE_SUGGESTIONS,
      0,
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_u32(
      HKEY_CURRENT_USER,
      CONTENT_DELIVERY_PATH,
      SOFT_LANDING,
      0,
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_u32(
      HKEY_CURRENT_USER,
      CONTENT_DELIVERY_PATH,
      SUBSCRIBED_CONTENT,
      0,
    )
    .map_err(|e| e.to_string())
  }

  fn revert(&self) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_CURRENT_USER,
      CONTENT_DELIVERY_PATH,
      SYSTEM_PANE_SUGGESTIONS,
      1,
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_u32(
      HKEY_CURRENT_USER,
      CONTENT_DELIVERY_PATH,
      SOFT_LANDING,
      1,
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_u32(
      HKEY_CURRENT_USER,
      CONTENT_DELIVERY_PATH,
      SUBSCRIBED_CONTENT,
      1,
    )
    .map_err(|e| e.to_string())
  }
}
