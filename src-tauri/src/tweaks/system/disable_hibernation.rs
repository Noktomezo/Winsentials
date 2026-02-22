use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const POWER_PATH: &str = r"SYSTEM\CurrentControlSet\Control\Power";
const SESSION_POWER_PATH: &str =
  r"SYSTEM\CurrentControlSet\Control\Session Manager\Power";
const HIBERNATE_ENABLED: &str = "HibernateEnabled";
const HIBERNATE_ENABLED_DEFAULT: &str = "HibernateEnabledDefault";
const HIBERBOOT_ENABLED: &str = "HiberbootEnabled";

pub struct DisableHibernationTweak {
  meta: TweakMeta,
}

impl DisableHibernationTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "disable_hibernation".to_string(),
        category: TweakCategory::System,
        name_key: "tweaks.disableHibernation.name".to_string(),
        description_key: "tweaks.disableHibernation.description".to_string(),
        details_key: "tweaks.disableHibernation.details".to_string(),
        ui_type: TweakUiType::Toggle,
        options: vec![],
        requires_reboot: true,
        risk_level: RiskLevel::Low,
        min_windows_build: None,
      },
    }
  }
}

impl Tweak for DisableHibernationTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let value =
      registry::read_reg_u32(HKEY_LOCAL_MACHINE, POWER_PATH, HIBERNATE_ENABLED);
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
      SESSION_POWER_PATH,
      HIBERBOOT_ENABLED,
      0,
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      POWER_PATH,
      HIBERNATE_ENABLED_DEFAULT,
      0,
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      POWER_PATH,
      HIBERNATE_ENABLED,
      0,
    )
    .map_err(|e| e.to_string())
  }

  fn revert(&self) -> Result<(), String> {
    registry::delete_reg_value(
      HKEY_LOCAL_MACHINE,
      SESSION_POWER_PATH,
      HIBERBOOT_ENABLED,
    )
    .ok();
    registry::delete_reg_value(
      HKEY_LOCAL_MACHINE,
      POWER_PATH,
      HIBERNATE_ENABLED_DEFAULT,
    )
    .ok();
    registry::delete_reg_value(
      HKEY_LOCAL_MACHINE,
      POWER_PATH,
      HIBERNATE_ENABLED,
    )
    .ok();
    Ok(())
  }
}
