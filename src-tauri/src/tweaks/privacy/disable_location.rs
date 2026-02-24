use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const LOCATION_SENSORS_PATH: &str =
  r"SOFTWARE\Policies\Microsoft\Windows\LocationAndSensors";
const DISABLE_LOCATION: &str = "DisableLocation";

pub struct DisableLocationTweak {
  meta: TweakMeta,
}

impl DisableLocationTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "disable_location".to_string(),
        category: TweakCategory::Privacy,
        name_key: "tweaks.disableLocation.name".to_string(),
        description_key: "tweaks.disableLocation.description".to_string(),
        details_key: "tweaks.disableLocation.details".to_string(),
        ui_type: TweakUiType::Toggle,
        options: vec![],
        requires_reboot: false,
        risk_level: RiskLevel::Low,
        min_windows_build: None,
      },
    }
  }
}

impl Tweak for DisableLocationTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let location = registry::read_reg_u32(
      HKEY_LOCAL_MACHINE,
      LOCATION_SENSORS_PATH,
      DISABLE_LOCATION,
    );
    let is_applied = location.map(|v| v == 1).unwrap_or(false);
    Ok(TweakState {
      id: self.meta.id.clone(),
      current_value: Some(if is_applied { "1" } else { "0" }.to_string()),
      is_applied,
    })
  }

  fn apply(&self, _value: Option<&str>) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      LOCATION_SENSORS_PATH,
      DISABLE_LOCATION,
      1,
    )
    .map_err(|e| e.to_string())
  }

  fn revert(&self) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      LOCATION_SENSORS_PATH,
      DISABLE_LOCATION,
      0,
    )
    .map_err(|e| e.to_string())
  }
}
