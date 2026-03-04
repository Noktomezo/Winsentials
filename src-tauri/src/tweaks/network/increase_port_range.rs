use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const TCPIP_PARAMS_PATH: &str =
  r"SYSTEM\CurrentControlSet\Services\Tcpip\Parameters";
const MAX_USER_PORT: &str = "MaxUserPort";
const TCP_TIMED_WAIT_DELAY: &str = "TcpTimedWaitDelay";

pub struct IncreasePortRangeTweak {
  meta: TweakMeta,
}

impl IncreasePortRangeTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "increase_port_range".to_string(),
        category: TweakCategory::Network,
        name_key: "tweaks.increasePortRange.name".to_string(),
        description_key: "tweaks.increasePortRange.description".to_string(),
        details_key: "tweaks.increasePortRange.details".to_string(),
        risk_details_key: None,
        ui_type: TweakUiType::Toggle,
        options: vec![],
        requires_reboot: true,
        requires_logout: false,
        risk_level: RiskLevel::Low,
        min_windows_build: None,
      },
    }
  }
}

impl Tweak for IncreasePortRangeTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let max_port = registry::read_reg_u32(
      HKEY_LOCAL_MACHINE,
      TCPIP_PARAMS_PATH,
      MAX_USER_PORT,
    );
    let timed_wait = registry::read_reg_u32(
      HKEY_LOCAL_MACHINE,
      TCPIP_PARAMS_PATH,
      TCP_TIMED_WAIT_DELAY,
    );

    let is_applied = max_port.map(|v| v == 65534).unwrap_or(false)
      && timed_wait.map(|v| v == 30).unwrap_or(false);

    Ok(TweakState {
      id: self.meta.id.clone(),
      current_value: Some(if is_applied { "1" } else { "0" }.to_string()),
      is_applied,
    })
  }

  fn apply(&self, _value: Option<&str>) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      TCPIP_PARAMS_PATH,
      MAX_USER_PORT,
      65534,
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      TCPIP_PARAMS_PATH,
      TCP_TIMED_WAIT_DELAY,
      30,
    )
    .map_err(|e| e.to_string())?;
    Ok(())
  }

  fn revert(&self) -> Result<(), String> {
    registry::delete_reg_value(
      HKEY_LOCAL_MACHINE,
      TCPIP_PARAMS_PATH,
      MAX_USER_PORT,
    )
    .ok();
    registry::delete_reg_value(
      HKEY_LOCAL_MACHINE,
      TCPIP_PARAMS_PATH,
      TCP_TIMED_WAIT_DELAY,
    )
    .ok();
    Ok(())
  }
}
