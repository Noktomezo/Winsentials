use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakOption, TweakState,
  TweakUiType,
};
use winreg::enums::*;

const PRIORITY_PATH: &str = r"SYSTEM\CurrentControlSet\Control\PriorityControl";
const PRIORITY_VALUE: &str = "Win32PrioritySeparation";

pub struct Win32PrioritySeparationTweak {
  meta: TweakMeta,
}

impl Win32PrioritySeparationTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "win32_priority_separation".to_string(),
        category: TweakCategory::System,
        name_key: "tweaks.win32PrioritySeparation.name".to_string(),
        description_key: "tweaks.win32PrioritySeparation.description"
          .to_string(),
        details_key: "tweaks.win32PrioritySeparation.details".to_string(),
        ui_type: TweakUiType::Radio,
        options: vec![
          TweakOption {
            value: "2".to_string(),
            label_key: "tweaks.win32PrioritySeparation.options.default"
              .to_string(),
            is_default: true,
            is_recommended: false,
          },
          TweakOption {
            value: "38".to_string(),
            label_key: "tweaks.win32PrioritySeparation.options.gaming"
              .to_string(),
            is_default: false,
            is_recommended: true,
          },
          TweakOption {
            value: "42".to_string(),
            label_key: "tweaks.win32PrioritySeparation.options.responsive"
              .to_string(),
            is_default: false,
            is_recommended: false,
          },
          TweakOption {
            value: "40".to_string(),
            label_key: "tweaks.win32PrioritySeparation.options.lowlatency"
              .to_string(),
            is_default: false,
            is_recommended: false,
          },
          TweakOption {
            value: "26".to_string(),
            label_key: "tweaks.win32PrioritySeparation.options.balanced"
              .to_string(),
            is_default: false,
            is_recommended: false,
          },
          TweakOption {
            value: "24".to_string(),
            label_key: "tweaks.win32PrioritySeparation.options.services"
              .to_string(),
            is_default: false,
            is_recommended: false,
          },
        ],
        requires_reboot: true,
        risk_level: RiskLevel::Low,
        min_windows_build: None,
      },
    }
  }
}

impl Tweak for Win32PrioritySeparationTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let value =
      registry::read_reg_u32(HKEY_LOCAL_MACHINE, PRIORITY_PATH, PRIORITY_VALUE)
        .map(|v| v.to_string());
    let is_applied = value.as_ref().map(|v| v != "2").unwrap_or(false);
    Ok(TweakState {
      id: self.meta.id.clone(),
      current_value: value,
      is_applied,
    })
  }

  fn apply(&self, value: Option<&str>) -> Result<(), String> {
    let val = value.and_then(|v| v.parse::<u32>().ok()).unwrap_or(38);
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      PRIORITY_PATH,
      PRIORITY_VALUE,
      val,
    )
    .map_err(|e| e.to_string())
  }

  fn revert(&self) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      PRIORITY_PATH,
      PRIORITY_VALUE,
      2,
    )
    .map_err(|e| e.to_string())
  }
}
