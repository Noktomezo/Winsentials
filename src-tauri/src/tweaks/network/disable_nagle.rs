use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::RegKey;
use winreg::enums::*;

const MSMQ_PARAMS_PATH: &str = r"SOFTWARE\Microsoft\MSMQ\Parameters";
const TCPIP_INTERFACES_PATH: &str =
  r"SYSTEM\CurrentControlSet\Services\Tcpip\Parameters\Interfaces";
const TCP_NO_DELAY: &str = "TCPNoDelay";
const TCP_ACK_FREQUENCY: &str = "TcpAckFrequency";
const TCP_DEL_ACK_TICKS: &str = "TcpDelAckTicks";

fn get_interface_guids() -> Result<Vec<String>, String> {
  let root = RegKey::predef(HKEY_LOCAL_MACHINE);
  let interfaces = root
    .open_subkey(TCPIP_INTERFACES_PATH)
    .map_err(|e| format!("Failed to open interfaces key: {e}"))?;

  Ok(
    interfaces
      .enum_keys()
      .filter_map(|k| k.ok())
      .filter(|k| k.starts_with('{') && k.ends_with('}'))
      .collect(),
  )
}

fn check_msmq() -> bool {
  registry::read_reg_u32(HKEY_LOCAL_MACHINE, MSMQ_PARAMS_PATH, TCP_NO_DELAY)
    .map(|v| v == 1)
    .unwrap_or(false)
}

fn check_interfaces() -> Result<bool, String> {
  let guids = get_interface_guids()?;

  for guid in &guids {
    let path = format!(r"{TCPIP_INTERFACES_PATH}\{guid}");
    let ack =
      registry::read_reg_u32(HKEY_LOCAL_MACHINE, &path, TCP_ACK_FREQUENCY);
    let del =
      registry::read_reg_u32(HKEY_LOCAL_MACHINE, &path, TCP_DEL_ACK_TICKS);

    let ack_ok = ack.map(|v| v == 1).unwrap_or(false);
    let del_ok = del.map(|v| v == 0).unwrap_or(false);

    if !(ack_ok && del_ok) {
      return Ok(false);
    }
  }

  Ok(true)
}

pub struct DisableNagleTweak {
  meta: TweakMeta,
}

impl DisableNagleTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "disable_nagle".to_string(),
        category: TweakCategory::Network,
        name_key: "tweaks.disableNagle.name".to_string(),
        description_key: "tweaks.disableNagle.description".to_string(),
        details_key: "tweaks.disableNagle.details".to_string(),
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

impl Tweak for DisableNagleTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let msmq_ok = check_msmq();
    let interfaces_ok = check_interfaces()?;
    let is_applied = msmq_ok && interfaces_ok;

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
    .map_err(|e| e.to_string())?;

    let guids = get_interface_guids()?;
    for guid in &guids {
      let path = format!(r"{TCPIP_INTERFACES_PATH}\{guid}");
      registry::write_reg_u32(HKEY_LOCAL_MACHINE, &path, TCP_ACK_FREQUENCY, 1)
        .map_err(|e| e.to_string())?;
      registry::write_reg_u32(HKEY_LOCAL_MACHINE, &path, TCP_DEL_ACK_TICKS, 0)
        .map_err(|e| e.to_string())?;
    }

    Ok(())
  }

  fn revert(&self) -> Result<(), String> {
    registry::delete_reg_value(
      HKEY_LOCAL_MACHINE,
      MSMQ_PARAMS_PATH,
      TCP_NO_DELAY,
    )
    .ok();

    let guids = get_interface_guids()?;
    for guid in &guids {
      let path = format!(r"{TCPIP_INTERFACES_PATH}\{guid}");
      registry::delete_reg_value(HKEY_LOCAL_MACHINE, &path, TCP_ACK_FREQUENCY)
        .ok();
      registry::delete_reg_value(HKEY_LOCAL_MACHINE, &path, TCP_DEL_ACK_TICKS)
        .ok();
    }

    Ok(())
  }
}
