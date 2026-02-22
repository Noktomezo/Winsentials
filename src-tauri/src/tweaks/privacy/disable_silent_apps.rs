use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const CONTENT_DELIVERY_PATH: &str =
  r"Software\Microsoft\Windows\CurrentVersion\ContentDeliveryManager";
const SILENT_INSTALLED_APPS: &str = "SilentInstalledAppsEnabled";

pub struct DisableSilentAppsTweak {
  meta: TweakMeta,
}

impl DisableSilentAppsTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "disable_silent_apps".to_string(),
        category: TweakCategory::Privacy,
        name_key: "tweaks.disableSilentApps.name".to_string(),
        description_key: "tweaks.disableSilentApps.description".to_string(),
        details_key: "tweaks.disableSilentApps.details".to_string(),
        ui_type: TweakUiType::Toggle,
        options: vec![],
        requires_reboot: false,
        risk_level: RiskLevel::Low,
        min_windows_build: None,
      },
    }
  }
}

impl Tweak for DisableSilentAppsTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let value = registry::read_reg_u32(
      HKEY_CURRENT_USER,
      CONTENT_DELIVERY_PATH,
      SILENT_INSTALLED_APPS,
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
      SILENT_INSTALLED_APPS,
      0,
    )
    .map_err(|e| e.to_string())
  }

  fn revert(&self) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_CURRENT_USER,
      CONTENT_DELIVERY_PATH,
      SILENT_INSTALLED_APPS,
      1,
    )
    .map_err(|e| e.to_string())
  }
}
