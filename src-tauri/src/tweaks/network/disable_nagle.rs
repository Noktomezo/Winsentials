use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use windows::Win32::NetworkManagement::IpHelper::{
  GET_ADAPTERS_ADDRESSES_FLAGS, GetAdaptersAddresses, IP_ADAPTER_ADDRESSES_LH,
  MIB_IF_TYPE_ETHERNET,
};
use winreg::RegKey;
use winreg::enums::*;

const MSMQ_PARAMS_PATH: &str = r"SOFTWARE\Microsoft\MSMQ\Parameters";
const TCPIP_INTERFACES_PATH: &str =
  r"SYSTEM\CurrentControlSet\Services\Tcpip\Parameters\Interfaces";
const TCP_NO_DELAY: &str = "TCPNoDelay";
const TCP_ACK_FREQUENCY: &str = "TcpAckFrequency";
const TCP_DEL_ACK_TICKS: &str = "TcpDelAckTicks";

const IF_TYPE_IEEE80211: u32 = 71;

fn get_network_adapter_guids() -> Result<Vec<String>, String> {
  let mut size: u32 = 0;
  let flags = GET_ADAPTERS_ADDRESSES_FLAGS(0);

  unsafe {
    GetAdaptersAddresses(0, flags, None, None, &mut size);
  }

  if size == 0 {
    return Ok(vec![]);
  }

  let buffer = vec![0u8; size as usize];
  let mut addresses = buffer.as_ptr() as *mut IP_ADAPTER_ADDRESSES_LH;

  unsafe {
    let result =
      GetAdaptersAddresses(0, flags, None, Some(addresses), &mut size);
    if result != 0 {
      return Err(format!("GetAdaptersAddresses failed: {}", result));
    }
  }

  let mut guids = Vec::new();

  unsafe {
    while !addresses.is_null() {
      let adapter = &*addresses;
      let if_type = adapter.IfType;

      if if_type == MIB_IF_TYPE_ETHERNET || if_type == IF_TYPE_IEEE80211 {
        let name_ptr = adapter.AdapterName;
        if !name_ptr.is_null() {
          let mut guid_bytes = Vec::new();
          let mut offset = 0;
          let raw_ptr = name_ptr.0;
          while *raw_ptr.add(offset) != 0 {
            guid_bytes.push(*raw_ptr.add(offset));
            offset += 1;
          }
          if let Ok(guid_str) = String::from_utf8(guid_bytes) {
            guids.push(guid_str);
          }
        }
      }

      addresses = adapter.Next;
    }
  }

  Ok(guids)
}

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

fn get_filtered_interface_guids() -> Result<Vec<String>, String> {
  let network_guids = get_network_adapter_guids()?;
  let all_interfaces = get_interface_guids()?;

  Ok(
    all_interfaces
      .into_iter()
      .filter(|g| network_guids.iter().any(|ng| ng.eq_ignore_ascii_case(g)))
      .collect(),
  )
}

fn check_msmq() -> bool {
  registry::read_reg_u32(HKEY_LOCAL_MACHINE, MSMQ_PARAMS_PATH, TCP_NO_DELAY)
    .map(|v| v == 1)
    .unwrap_or(false)
}

fn check_interfaces() -> Result<bool, String> {
  let guids = get_filtered_interface_guids()?;

  for guid in &guids {
    let path = format!(r"{TCPIP_INTERFACES_PATH}\{guid}");
    let no_delay =
      registry::read_reg_u32(HKEY_LOCAL_MACHINE, &path, TCP_NO_DELAY);
    let ack =
      registry::read_reg_u32(HKEY_LOCAL_MACHINE, &path, TCP_ACK_FREQUENCY);
    let del =
      registry::read_reg_u32(HKEY_LOCAL_MACHINE, &path, TCP_DEL_ACK_TICKS);

    let no_delay_ok = no_delay.map(|v| v == 1).unwrap_or(false);
    let ack_ok = ack.map(|v| v == 1).unwrap_or(false);
    let del_ok = del.map(|v| v == 0).unwrap_or(false);

    if !(no_delay_ok && ack_ok && del_ok) {
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

    let guids = get_filtered_interface_guids()?;
    for guid in &guids {
      let path = format!(r"{TCPIP_INTERFACES_PATH}\{guid}");
      registry::write_reg_u32(HKEY_LOCAL_MACHINE, &path, TCP_NO_DELAY, 1)
        .map_err(|e| e.to_string())?;
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

    let guids = get_filtered_interface_guids()?;
    for guid in &guids {
      let path = format!(r"{TCPIP_INTERFACES_PATH}\{guid}");
      registry::delete_reg_value(HKEY_LOCAL_MACHINE, &path, TCP_NO_DELAY).ok();
      registry::delete_reg_value(HKEY_LOCAL_MACHINE, &path, TCP_ACK_FREQUENCY)
        .ok();
      registry::delete_reg_value(HKEY_LOCAL_MACHINE, &path, TCP_DEL_ACK_TICKS)
        .ok();
    }

    Ok(())
  }
}
