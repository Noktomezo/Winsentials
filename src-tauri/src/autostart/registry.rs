use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

use rayon::prelude::*;
use windows::Win32::System::Environment::ExpandEnvironmentStringsW;
use winreg::enums::*;
use winreg::{HKEY, RegKey, RegValue};

use crate::autostart::critical::get_critical_level;
use crate::autostart::file_info::get_file_version_info;
use crate::autostart::icons::get_icon;
use crate::autostart::types::{AutostartItem, AutostartSource};

const REGISTRY_KEYS: &[(&str, HKEY, &str)] = &[
  (
    "HKCU\\Run",
    HKEY_CURRENT_USER,
    r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run",
  ),
  (
    "HKLM\\Run",
    HKEY_LOCAL_MACHINE,
    r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run",
  ),
  (
    "HKLM\\Run (WOW64)",
    HKEY_LOCAL_MACHINE,
    r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Run",
  ),
];

struct RawRegistryItem {
  name: String,
  command: String,
  location_name: String,
  is_disabled: bool,
}

fn is_disabled_in_startup_approved(
  name: &str,
  hk_root: HKEY,
  base_path: &str,
) -> bool {
  let root = RegKey::predef(hk_root);

  let approved_path = base_path.replace(
    r"\CurrentVersion\Run",
    r"\CurrentVersion\Explorer\StartupApproved\Run",
  );

  if let Ok(key) = root.open_subkey(approved_path)
    && let Ok(value) = key.get_raw_value(name)
    && value.bytes.len() >= 2
  {
    let first_byte = value.bytes[0];
    // 0x01, 0x03, and 0x09 indicate disabled state in StartupApproved
    if first_byte == 0x03 || first_byte == 0x01 || first_byte == 0x09 {
      return true;
    }
  }

  false
}

fn get_target_path(command: &str) -> Option<String> {
  let cmd = command.trim();

  if cmd.is_empty() {
    return None;
  }

  // Handle quoted paths
  if let Some(stripped) = cmd.strip_prefix('"') {
    if let Some(end_quote) = stripped.find('"') {
      return Some(stripped[..end_quote].to_string());
    }
  }

  // For unquoted paths, find the executable by looking for .exe
  let lower_cmd = cmd.to_lowercase();
  if let Some(exe_end) = lower_cmd.find(".exe") {
    let exe_path = cmd[..exe_end + 4].trim();
    // Handle paths with spaces - take the longest valid path ending with .exe
    return Some(exe_path.to_string());
  }

  // Fallback: take first token
  let parts: Vec<&str> = cmd.split_whitespace().collect();
  if !parts.is_empty() {
    return Some(parts[0].to_string());
  }

  None
}

fn expand_env_vars(s: &str) -> String {
  let wide: Vec<u16> = s.encode_utf16().chain(std::iter::once(0)).collect();
  let input = windows::core::PCWSTR(wide.as_ptr());

  unsafe {
    let required_len = ExpandEnvironmentStringsW(input, None);
    if required_len == 0 {
      return s.to_string();
    }

    let mut buffer = vec![0u16; required_len as usize];
    let len = ExpandEnvironmentStringsW(input, Some(&mut buffer));
    if len == 0 {
      return s.to_string();
    }

    let result: OsString =
      OsStringExt::from_wide(&buffer[..(len as usize).saturating_sub(1)]);
    result.to_string_lossy().to_string()
  }
}

pub fn get_registry_autostart_items() -> Vec<AutostartItem> {
  let raw_items = collect_raw_registry_items();

  raw_items
    .into_par_iter()
    .map(enrich_registry_item)
    .collect()
}

pub fn get_registry_items_fast() -> Vec<AutostartItem> {
  let raw_items = collect_raw_registry_items();

  raw_items
    .into_iter()
    .map(|raw| {
      let target_path = get_target_path(&raw.command);
      let critical_level = get_critical_level(&raw.name, &raw.command);

      let id = format!(
        "registry|{}|{}",
        raw.location_name.replace('\\', "/"),
        raw.name
      );

      AutostartItem {
        id,
        name: raw.name,
        publisher: String::new(),
        command: raw.command,
        location: raw.location_name,
        source: AutostartSource::Registry,
        is_enabled: !raw.is_disabled,
        is_delayed: false,
        icon_base64: None,
        critical_level,
        file_path: target_path,
        start_type: None,
      }
    })
    .collect()
}

fn collect_raw_registry_items() -> Vec<RawRegistryItem> {
  let mut items = Vec::new();

  for (location_name, hk_root, path) in REGISTRY_KEYS {
    let root = RegKey::predef(*hk_root);

    if let Ok(key) = root.open_subkey(*path) {
      for (name, value) in key.enum_values().filter_map(|r| r.ok()) {
        let command: String = match value.vtype {
          RegType::REG_SZ => {
            if value.bytes.len() >= 2 && value.bytes.len() % 2 == 0 {
              let u16_slice: Vec<u16> = value
                .bytes
                .chunks_exact(2)
                .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
                .collect();
              String::from_utf16_lossy(&u16_slice)
                .trim_end_matches('\0')
                .to_string()
            } else {
              String::from_utf8_lossy(&value.bytes)
                .trim_end_matches('\0')
                .to_string()
            }
          }
          RegType::REG_EXPAND_SZ => {
            let raw = if value.bytes.len() >= 2 && value.bytes.len() % 2 == 0 {
              let u16_slice: Vec<u16> = value
                .bytes
                .chunks_exact(2)
                .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
                .collect();
              String::from_utf16_lossy(&u16_slice)
                .trim_end_matches('\0')
                .to_string()
            } else {
              String::from_utf8_lossy(&value.bytes)
                .trim_end_matches('\0')
                .to_string()
            };
            expand_env_vars(&raw)
          }
          _ => continue,
        };

        let is_disabled =
          is_disabled_in_startup_approved(&name, *hk_root, path);

        items.push(RawRegistryItem {
          name,
          command,
          location_name: location_name.to_string(),
          is_disabled,
        });
      }
    }
  }

  items
}

fn enrich_registry_item(raw: RawRegistryItem) -> AutostartItem {
  let target_path = get_target_path(&raw.command);
  let icon_base64 = target_path.as_ref().and_then(|p| get_icon(p));

  let critical_level = get_critical_level(&raw.name, &raw.command);

  let publisher = target_path
    .as_ref()
    .and_then(|p| get_file_version_info(p).ok())
    .and_then(|v| v.company_name)
    .unwrap_or_default();

  let id = format!(
    "registry|{}|{}",
    raw.location_name.replace('\\', "/"),
    raw.name
  );

  AutostartItem {
    id,
    name: raw.name,
    publisher,
    command: raw.command,
    location: raw.location_name,
    source: AutostartSource::Registry,
    is_enabled: !raw.is_disabled,
    is_delayed: false,
    icon_base64,
    critical_level,
    file_path: target_path,
    start_type: None,
  }
}

pub fn toggle_registry_item(id: &str, enable: bool) -> Result<(), String> {
  let parts: Vec<&str> = id.splitn(3, '|').collect();
  if parts.len() != 3 || parts[0] != "registry" {
    return Err("Invalid registry item ID".to_string());
  }

  let location = parts[1];
  let name = parts[2];

  let (hk_root, _base_path, approved_path) = match location {
    "HKCU/Run" => (
      HKEY_CURRENT_USER,
      r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run",
      r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run",
    ),
    "HKLM/Run" => (
      HKEY_LOCAL_MACHINE,
      r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run",
      r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run",
    ),
    "HKLM/Run (WOW64)" => (
      HKEY_LOCAL_MACHINE,
      r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Run",
      r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run",
    ),
    _ => return Err("Unknown registry location".to_string()),
  };

  let root = RegKey::predef(hk_root);

  if enable {
    let approved_key = root
      .create_subkey(approved_path)
      .map(|(k, _)| k)
      .map_err(|e| format!("Failed to open StartupApproved: {e}"))?;

    let _ = approved_key.delete_value(name);
  } else {
    let approved_key = root
      .create_subkey(approved_path)
      .map(|(k, _)| k)
      .map_err(|e| format!("Failed to open StartupApproved: {e}"))?;

    let data: Vec<u8> = vec![
      0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];
    approved_key
      .set_raw_value(
        name,
        &RegValue {
          vtype: RegType::REG_BINARY,
          bytes: data,
        },
      )
      .map_err(|e| format!("Failed to disable item: {e}"))?;
  }

  Ok(())
}

pub fn delete_registry_item(id: &str) -> Result<(), String> {
  let parts: Vec<&str> = id.splitn(3, '|').collect();
  if parts.len() != 3 || parts[0] != "registry" {
    return Err("Invalid registry item ID".to_string());
  }

  let location = parts[1];
  let name = parts[2];

  let (hk_root, base_path, approved_path) = match location {
    "HKCU/Run" => (
      HKEY_CURRENT_USER,
      r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run",
      r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run",
    ),
    "HKLM/Run" => (
      HKEY_LOCAL_MACHINE,
      r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run",
      r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run",
    ),
    "HKLM/Run (WOW64)" => (
      HKEY_LOCAL_MACHINE,
      r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Run",
      r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run",
    ),
    _ => return Err("Unknown registry location".to_string()),
  };

  let root = RegKey::predef(hk_root);

  let key = root
    .open_subkey_with_flags(base_path, KEY_WRITE | KEY_READ)
    .map_err(|e| format!("Failed to open registry key: {e}"))?;

  key
    .delete_value(name)
    .map_err(|e| format!("Failed to delete registry value: {e}"))?;

  if let Ok(approved_key) =
    root.open_subkey_with_flags(approved_path, KEY_WRITE)
  {
    let _ = approved_key.delete_value(name);
  }

  Ok(())
}
