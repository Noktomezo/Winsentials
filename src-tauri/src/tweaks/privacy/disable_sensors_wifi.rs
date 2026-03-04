use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const LOCATION_SENSORS_PATH: &str =
  r"SOFTWARE\Policies\Microsoft\Windows\LocationAndSensors";
const DISABLE_SENSORS: &str = "DisableSensors";
const WIFISENSE_PATH: &str =
  r"SOFTWARE\Microsoft\WcmSvc\wifinetworkmanager\features";
const WIFI_SENSE_ENABLED: &str = "WiFiSenseEnabled";

pub struct DisableSensorsWifiTweak {
  meta: TweakMeta,
}

impl DisableSensorsWifiTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "disable_sensors_wifi".to_string(),
        category: TweakCategory::Privacy,
        name_key: "tweaks.disableSensorsWifi.name".to_string(),
        description_key: "tweaks.disableSensorsWifi.description".to_string(),
        details_key: "tweaks.disableSensorsWifi.details".to_string(),
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

impl Tweak for DisableSensorsWifiTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let sensors = registry::read_reg_u32(
      HKEY_LOCAL_MACHINE,
      LOCATION_SENSORS_PATH,
      DISABLE_SENSORS,
    );
    let wifi = registry::read_reg_u32(
      HKEY_LOCAL_MACHINE,
      WIFISENSE_PATH,
      WIFI_SENSE_ENABLED,
    );
    let is_applied = sensors.map(|v| v == 1).unwrap_or(false)
      && wifi.map(|v| v == 0).unwrap_or(false);

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
      DISABLE_SENSORS,
      1,
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      WIFISENSE_PATH,
      WIFI_SENSE_ENABLED,
      0,
    )
    .map_err(|e| e.to_string())?;
    Ok(())
  }

  fn revert(&self) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      LOCATION_SENSORS_PATH,
      DISABLE_SENSORS,
      0,
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      WIFISENSE_PATH,
      WIFI_SENSE_ENABLED,
      1,
    )
    .map_err(|e| e.to_string())?;
    Ok(())
  }
}
