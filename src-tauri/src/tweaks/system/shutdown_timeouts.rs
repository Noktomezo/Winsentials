use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const DESKTOP_PATH: &str = r"Control Panel\Desktop";
const CONTROL_PATH: &str = r"SYSTEM\CurrentControlSet\Control";
const WAIT_TO_KILL_APP: &str = "WaitToKillAppTimeout";
const WAIT_TO_KILL_SERVICE: &str = "WaitToKillServiceTimeout";
const HUNG_APP: &str = "HungAppTimeout";

pub struct ShutdownTimeoutsTweak {
  meta: TweakMeta,
}

impl ShutdownTimeoutsTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "shutdown_timeouts".to_string(),
        category: TweakCategory::System,
        name_key: "tweaks.shutdownTimeouts.name".to_string(),
        description_key: "tweaks.shutdownTimeouts.description".to_string(),
        details_key: "tweaks.shutdownTimeouts.details".to_string(),
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

impl Tweak for ShutdownTimeoutsTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let value = registry::read_reg_string(
      HKEY_CURRENT_USER,
      DESKTOP_PATH,
      WAIT_TO_KILL_APP,
    );
    let is_applied = value.map(|v| v == "1000").unwrap_or(false);
    Ok(TweakState {
      id: self.meta.id.clone(),
      current_value: Some(if is_applied { "1" } else { "0" }.to_string()),
      is_applied,
    })
  }

  fn apply(&self, _value: Option<&str>) -> Result<(), String> {
    registry::write_reg_string(
      HKEY_CURRENT_USER,
      DESKTOP_PATH,
      WAIT_TO_KILL_APP,
      "1000",
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_string(
      HKEY_LOCAL_MACHINE,
      CONTROL_PATH,
      WAIT_TO_KILL_SERVICE,
      "1000",
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_string(
      HKEY_CURRENT_USER,
      DESKTOP_PATH,
      HUNG_APP,
      "1000",
    )
    .map_err(|e| e.to_string())
  }

  fn revert(&self) -> Result<(), String> {
    registry::write_reg_string(
      HKEY_CURRENT_USER,
      DESKTOP_PATH,
      WAIT_TO_KILL_APP,
      "20000",
    )
    .ok();
    registry::write_reg_string(
      HKEY_LOCAL_MACHINE,
      CONTROL_PATH,
      WAIT_TO_KILL_SERVICE,
      "20000",
    )
    .ok();
    registry::write_reg_string(
      HKEY_CURRENT_USER,
      DESKTOP_PATH,
      HUNG_APP,
      "5000",
    )
    .ok();
    Ok(())
  }
}
