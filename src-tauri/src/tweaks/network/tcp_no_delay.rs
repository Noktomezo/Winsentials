use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const MSMQ_PARAMS_PATH: &str = r"SOFTWARE\Microsoft\MSMQ\Parameters";
const TCP_NO_DELAY: &str = "TCPNoDelay";

pub struct TcpNoDelayTweak {
  meta: TweakMeta,
}

impl TcpNoDelayTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "tcp_no_delay".to_string(),
        category: TweakCategory::Network,
        name_key: "tweaks.tcpNoDelay.name".to_string(),
        description_key: "tweaks.tcpNoDelay.description".to_string(),
        details_key: "tweaks.tcpNoDelay.details".to_string(),
        ui_type: TweakUiType::Toggle,
        options: vec![],
        requires_reboot: false,
        risk_level: RiskLevel::Low,
        min_windows_build: None,
      },
    }
  }
}

impl Tweak for TcpNoDelayTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let value = registry::read_reg_u32(
      HKEY_LOCAL_MACHINE,
      MSMQ_PARAMS_PATH,
      TCP_NO_DELAY,
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
      MSMQ_PARAMS_PATH,
      TCP_NO_DELAY,
      1,
    )
    .map_err(|e| e.to_string())
  }

  fn revert(&self) -> Result<(), String> {
    registry::delete_reg_value(
      HKEY_LOCAL_MACHINE,
      MSMQ_PARAMS_PATH,
      TCP_NO_DELAY,
    )
    .ok();
    Ok(())
  }
}
