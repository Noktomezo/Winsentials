use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const CLOUD_CONTENT_PATH: &str =
  r"SOFTWARE\Policies\Microsoft\Windows\CloudContent";
const DISABLE_CONSUMER_FEATURES: &str = "DisableWindowsConsumerFeatures";

pub struct DisableConsumerFeaturesTweak {
  meta: TweakMeta,
}

impl DisableConsumerFeaturesTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "disable_consumer_features".to_string(),
        category: TweakCategory::Privacy,
        name_key: "tweaks.disableConsumerFeatures.name".to_string(),
        description_key: "tweaks.disableConsumerFeatures.description"
          .to_string(),
        details_key: "tweaks.disableConsumerFeatures.details".to_string(),
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

impl Tweak for DisableConsumerFeaturesTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let value = registry::read_reg_u32(
      HKEY_LOCAL_MACHINE,
      CLOUD_CONTENT_PATH,
      DISABLE_CONSUMER_FEATURES,
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
      HKEY_LOCAL_MACHINE,
      CLOUD_CONTENT_PATH,
      DISABLE_CONSUMER_FEATURES,
      1,
    )
    .map_err(|e| e.to_string())
  }

  fn revert(&self) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      CLOUD_CONTENT_PATH,
      DISABLE_CONSUMER_FEATURES,
      0,
    )
    .map_err(|e| e.to_string())
  }
}
