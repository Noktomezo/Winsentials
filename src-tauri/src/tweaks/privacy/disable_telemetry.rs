use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const DATA_COLLECTION_PATH: &str =
  r"SOFTWARE\Policies\Microsoft\Windows\DataCollection";
const ALLOW_TELEMETRY: &str = "AllowTelemetry";
const ALLOW_DEVICE_NAME_IN_TELEMETRY: &str = "AllowDeviceNameInTelemetry";

pub struct DisableTelemetryTweak {
  meta: TweakMeta,
}

impl DisableTelemetryTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "disable_telemetry".to_string(),
        category: TweakCategory::Privacy,
        name_key: "tweaks.disableTelemetry.name".to_string(),
        description_key: "tweaks.disableTelemetry.description".to_string(),
        details_key: "tweaks.disableTelemetry.details".to_string(),
        ui_type: TweakUiType::Toggle,
        options: vec![],
        requires_reboot: true,
        risk_level: RiskLevel::Low,
        min_windows_build: None,
      },
    }
  }
}

impl Tweak for DisableTelemetryTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let value = registry::read_reg_u32(
      HKEY_LOCAL_MACHINE,
      DATA_COLLECTION_PATH,
      ALLOW_TELEMETRY,
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
      DATA_COLLECTION_PATH,
      ALLOW_TELEMETRY,
      0,
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      DATA_COLLECTION_PATH,
      ALLOW_DEVICE_NAME_IN_TELEMETRY,
      0,
    )
    .map_err(|e| e.to_string())
  }

  fn revert(&self) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      DATA_COLLECTION_PATH,
      ALLOW_TELEMETRY,
      1,
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      DATA_COLLECTION_PATH,
      ALLOW_DEVICE_NAME_IN_TELEMETRY,
      1,
    )
    .map_err(|e| e.to_string())
  }
}
