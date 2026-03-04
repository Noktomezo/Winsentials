use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const MOUSE_PATH: &str = r"Control Panel\Mouse";
const MOUSE_SPEED: &str = "MouseSpeed";
const MOUSE_THRESHOLD1: &str = "MouseThreshold1";
const MOUSE_THRESHOLD2: &str = "MouseThreshold2";
const MOUSE_SENSITIVITY: &str = "MouseSensitivity";
const SMOOTH_MOUSE_Y_CURVE: &str = "SmoothMouseYCurve";
const SMOOTH_MOUSE_X_CURVE: &str = "SmoothMouseXCurve";

const SMOOTH_MOUSE_Y_CURVE_DISABLED: &[u8] = &[
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x38, 0x00, 0x00,
  0x00, 0x00, 0x00, 0x00, 0x00, 0x70, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
  0xA8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xE0, 0x00, 0x00, 0x00, 0x00,
  0x00,
];

const SMOOTH_MOUSE_X_CURVE_DISABLED: &[u8] = &[
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC0, 0xCC, 0x0C, 0x00, 0x00,
  0x00, 0x00, 0x00, 0x80, 0x99, 0x19, 0x00, 0x00, 0x00, 0x00, 0x00, 0x40, 0x66,
  0x26, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x33, 0x33, 0x00, 0x00, 0x00, 0x00,
  0x00,
];

const SMOOTH_MOUSE_Y_CURVE_DEFAULT: &[u8] = &[
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFD, 0x11, 0x01, 0x00, 0x00,
  0x00, 0x00, 0x00, 0x00, 0x24, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xFC,
  0x12, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xC0, 0xBB, 0x01, 0x00, 0x00, 0x00,
  0x00,
];

const SMOOTH_MOUSE_X_CURVE_DEFAULT: &[u8] = &[
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x15, 0x6E, 0x00, 0x00, 0x00,
  0x00, 0x00, 0x00, 0x00, 0x40, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x29, 0xDC,
  0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x28, 0x00, 0x00, 0x00, 0x00,
  0x00,
];

pub struct MouseAccelerationFixTweak {
  meta: TweakMeta,
}

impl MouseAccelerationFixTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "mouse_acceleration_fix".to_string(),
        category: TweakCategory::Input,
        name_key: "tweaks.mouseAccelerationFix.name".to_string(),
        description_key: "tweaks.mouseAccelerationFix.description".to_string(),
        details_key: "tweaks.mouseAccelerationFix.details".to_string(),
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

impl Tweak for MouseAccelerationFixTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let y_curve = registry::read_reg_binary(
      HKEY_CURRENT_USER,
      MOUSE_PATH,
      SMOOTH_MOUSE_Y_CURVE,
    );
    let is_applied = y_curve
      .map(|v| v == SMOOTH_MOUSE_Y_CURVE_DISABLED)
      .unwrap_or(false);
    Ok(TweakState {
      id: self.meta.id.clone(),
      current_value: Some(if is_applied { "1" } else { "0" }.to_string()),
      is_applied,
    })
  }

  fn apply(&self, _value: Option<&str>) -> Result<(), String> {
    registry::write_reg_string(HKEY_CURRENT_USER, MOUSE_PATH, MOUSE_SPEED, "0")
      .map_err(|e| e.to_string())?;
    registry::write_reg_string(
      HKEY_CURRENT_USER,
      MOUSE_PATH,
      MOUSE_THRESHOLD1,
      "0",
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_string(
      HKEY_CURRENT_USER,
      MOUSE_PATH,
      MOUSE_THRESHOLD2,
      "0",
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_string(
      HKEY_CURRENT_USER,
      MOUSE_PATH,
      MOUSE_SENSITIVITY,
      "10",
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_binary(
      HKEY_CURRENT_USER,
      MOUSE_PATH,
      SMOOTH_MOUSE_Y_CURVE,
      SMOOTH_MOUSE_Y_CURVE_DISABLED,
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_binary(
      HKEY_CURRENT_USER,
      MOUSE_PATH,
      SMOOTH_MOUSE_X_CURVE,
      SMOOTH_MOUSE_X_CURVE_DISABLED,
    )
    .map_err(|e| e.to_string())
  }

  fn revert(&self) -> Result<(), String> {
    registry::write_reg_string(HKEY_CURRENT_USER, MOUSE_PATH, MOUSE_SPEED, "1")
      .map_err(|e| e.to_string())?;
    registry::write_reg_string(
      HKEY_CURRENT_USER,
      MOUSE_PATH,
      MOUSE_THRESHOLD1,
      "6",
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_string(
      HKEY_CURRENT_USER,
      MOUSE_PATH,
      MOUSE_THRESHOLD2,
      "0",
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_string(
      HKEY_CURRENT_USER,
      MOUSE_PATH,
      MOUSE_SENSITIVITY,
      "10",
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_binary(
      HKEY_CURRENT_USER,
      MOUSE_PATH,
      SMOOTH_MOUSE_Y_CURVE,
      SMOOTH_MOUSE_Y_CURVE_DEFAULT,
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_binary(
      HKEY_CURRENT_USER,
      MOUSE_PATH,
      SMOOTH_MOUSE_X_CURVE,
      SMOOTH_MOUSE_X_CURVE_DEFAULT,
    )
    .map_err(|e| e.to_string())
  }
}
