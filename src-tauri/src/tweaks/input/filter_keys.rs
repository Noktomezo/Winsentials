use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakOption, TweakState,
  TweakUiType,
};
use winreg::enums::*;

const KEYBOARD_PATH: &str = r"Control Panel\Keyboard";
const ACCESSIBILITY_PATH: &str =
  r"Control Panel\Accessibility\Keyboard Response";

const KEYBOARD_DELAY: &str = "KeyboardDelay";
const KEYBOARD_SPEED: &str = "KeyboardSpeed";
const AUTO_REPEAT_DELAY: &str = "AutoRepeatDelay";
const AUTO_REPEAT_RATE: &str = "AutoRepeatRate";
const DELAY_BEFORE_ACCEPTANCE: &str = "DelayBeforeAcceptance";
const BOUNCE_TIME: &str = "BounceTime";
const FLAGS: &str = "Flags";

pub struct FilterKeysTweak {
  meta: TweakMeta,
}

impl FilterKeysTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "filter_keys".to_string(),
        category: TweakCategory::Input,
        name_key: "tweaks.filterKeys.name".to_string(),
        description_key: "tweaks.filterKeys.description".to_string(),
        details_key: "tweaks.filterKeys.details".to_string(),
        ui_type: TweakUiType::Radio,
        options: vec![
          TweakOption {
            value: "default".to_string(),
            label_key: "tweaks.filterKeys.options.default".to_string(),
            is_default: true,
            is_recommended: false,
          },
          TweakOption {
            value: "fast".to_string(),
            label_key: "tweaks.filterKeys.options.fast".to_string(),
            is_default: false,
            is_recommended: true,
          },
          TweakOption {
            value: "ultrafast".to_string(),
            label_key: "tweaks.filterKeys.options.ultrafast".to_string(),
            is_default: false,
            is_recommended: false,
          },
          TweakOption {
            value: "aggressive".to_string(),
            label_key: "tweaks.filterKeys.options.aggressive".to_string(),
            is_default: false,
            is_recommended: false,
          },
        ],
        requires_reboot: false,
        risk_level: RiskLevel::Low,
        min_windows_build: None,
      },
    }
  }
}

impl Tweak for FilterKeysTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let keyboard_delay = registry::read_reg_string(
      HKEY_CURRENT_USER,
      KEYBOARD_PATH,
      KEYBOARD_DELAY,
    );
    let auto_repeat_delay = registry::read_reg_string(
      HKEY_CURRENT_USER,
      ACCESSIBILITY_PATH,
      AUTO_REPEAT_DELAY,
    );
    let flags =
      registry::read_reg_string(HKEY_CURRENT_USER, ACCESSIBILITY_PATH, FLAGS);

    let current_value = if flags.is_none() && auto_repeat_delay.is_none() {
      if keyboard_delay.as_deref() == Some("0") {
        "fast"
      } else {
        "default"
      }
    } else if flags.is_some() && auto_repeat_delay.is_some() {
      if auto_repeat_delay.as_deref() == Some("150")
        && flags.as_deref() == Some("27")
      {
        "ultrafast"
      } else if auto_repeat_delay.as_deref() == Some("100")
        && flags.as_deref() == Some("27")
      {
        "aggressive"
      } else {
        "default"
      }
    } else {
      "partial"
    };

    let is_applied = current_value != "default";

    Ok(TweakState {
      id: self.meta.id.clone(),
      current_value: Some(current_value.to_string()),
      is_applied,
    })
  }

  fn apply(&self, value: Option<&str>) -> Result<(), String> {
    let preset = value.unwrap_or("fast");

    match preset {
      "default" => self.apply_default(),
      "fast" => self.apply_fast(),
      "ultrafast" => self.apply_ultrafast(),
      "aggressive" => self.apply_aggressive(),
      _ => Err(format!("Invalid filter keys preset: {preset}")),
    }
  }

  fn revert(&self) -> Result<(), String> {
    self.apply_default()
  }
}

impl FilterKeysTweak {
  fn apply_default(&self) -> Result<(), String> {
    registry::write_reg_string(
      HKEY_CURRENT_USER,
      KEYBOARD_PATH,
      KEYBOARD_DELAY,
      "1",
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_string(
      HKEY_CURRENT_USER,
      KEYBOARD_PATH,
      KEYBOARD_SPEED,
      "31",
    )
    .map_err(|e| e.to_string())?;

    let _ = registry::delete_reg_value(
      HKEY_CURRENT_USER,
      ACCESSIBILITY_PATH,
      AUTO_REPEAT_DELAY,
    );
    let _ = registry::delete_reg_value(
      HKEY_CURRENT_USER,
      ACCESSIBILITY_PATH,
      AUTO_REPEAT_RATE,
    );
    let _ = registry::delete_reg_value(
      HKEY_CURRENT_USER,
      ACCESSIBILITY_PATH,
      DELAY_BEFORE_ACCEPTANCE,
    );
    let _ = registry::delete_reg_value(
      HKEY_CURRENT_USER,
      ACCESSIBILITY_PATH,
      BOUNCE_TIME,
    );
    let _ =
      registry::delete_reg_value(HKEY_CURRENT_USER, ACCESSIBILITY_PATH, FLAGS);

    Ok(())
  }

  fn apply_fast(&self) -> Result<(), String> {
    registry::write_reg_string(
      HKEY_CURRENT_USER,
      KEYBOARD_PATH,
      KEYBOARD_DELAY,
      "0",
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_string(
      HKEY_CURRENT_USER,
      KEYBOARD_PATH,
      KEYBOARD_SPEED,
      "31",
    )
    .map_err(|e| e.to_string())?;

    let _ = registry::delete_reg_value(
      HKEY_CURRENT_USER,
      ACCESSIBILITY_PATH,
      AUTO_REPEAT_DELAY,
    );
    let _ = registry::delete_reg_value(
      HKEY_CURRENT_USER,
      ACCESSIBILITY_PATH,
      AUTO_REPEAT_RATE,
    );
    let _ = registry::delete_reg_value(
      HKEY_CURRENT_USER,
      ACCESSIBILITY_PATH,
      DELAY_BEFORE_ACCEPTANCE,
    );
    let _ = registry::delete_reg_value(
      HKEY_CURRENT_USER,
      ACCESSIBILITY_PATH,
      BOUNCE_TIME,
    );
    let _ =
      registry::delete_reg_value(HKEY_CURRENT_USER, ACCESSIBILITY_PATH, FLAGS);

    Ok(())
  }

  fn apply_ultrafast(&self) -> Result<(), String> {
    registry::write_reg_string(
      HKEY_CURRENT_USER,
      KEYBOARD_PATH,
      KEYBOARD_DELAY,
      "0",
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_string(
      HKEY_CURRENT_USER,
      KEYBOARD_PATH,
      KEYBOARD_SPEED,
      "31",
    )
    .map_err(|e| e.to_string())?;

    registry::write_reg_string(
      HKEY_CURRENT_USER,
      ACCESSIBILITY_PATH,
      AUTO_REPEAT_DELAY,
      "150",
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_string(
      HKEY_CURRENT_USER,
      ACCESSIBILITY_PATH,
      AUTO_REPEAT_RATE,
      "12",
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_string(
      HKEY_CURRENT_USER,
      ACCESSIBILITY_PATH,
      DELAY_BEFORE_ACCEPTANCE,
      "0",
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_string(
      HKEY_CURRENT_USER,
      ACCESSIBILITY_PATH,
      BOUNCE_TIME,
      "0",
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_string(
      HKEY_CURRENT_USER,
      ACCESSIBILITY_PATH,
      FLAGS,
      "27",
    )
    .map_err(|e| e.to_string())?;

    Ok(())
  }

  fn apply_aggressive(&self) -> Result<(), String> {
    registry::write_reg_string(
      HKEY_CURRENT_USER,
      KEYBOARD_PATH,
      KEYBOARD_DELAY,
      "0",
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_string(
      HKEY_CURRENT_USER,
      KEYBOARD_PATH,
      KEYBOARD_SPEED,
      "31",
    )
    .map_err(|e| e.to_string())?;

    registry::write_reg_string(
      HKEY_CURRENT_USER,
      ACCESSIBILITY_PATH,
      AUTO_REPEAT_DELAY,
      "100",
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_string(
      HKEY_CURRENT_USER,
      ACCESSIBILITY_PATH,
      AUTO_REPEAT_RATE,
      "8",
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_string(
      HKEY_CURRENT_USER,
      ACCESSIBILITY_PATH,
      DELAY_BEFORE_ACCEPTANCE,
      "0",
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_string(
      HKEY_CURRENT_USER,
      ACCESSIBILITY_PATH,
      BOUNCE_TIME,
      "0",
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_string(
      HKEY_CURRENT_USER,
      ACCESSIBILITY_PATH,
      FLAGS,
      "27",
    )
    .map_err(|e| e.to_string())?;

    Ok(())
  }
}
