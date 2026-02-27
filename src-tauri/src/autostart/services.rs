use std::mem::size_of;
use std::sync::atomic::{AtomicUsize, Ordering};

use log::{info, warn};
use rayon::prelude::*;
use windows::Win32::System::Services::*;
use windows::core::PCWSTR;
use winreg::RegKey;
use winreg::enums::*;

use crate::autostart::critical::get_critical_level;
use crate::autostart::types::{AutostartItem, AutostartSource};

const DELETE_ACCESS: u32 = 0x00010000;

const IGNORED_SERVICES: [&str; 10] = [
  "WinDefend",
  "WdNisSvc",
  "Sense",
  "wuauserv",
  "BITS",
  "EventLog",
  "PlugPlay",
  "RpcSs",
  "Winmgmt",
  "Schedule",
];

struct ServiceHandle(SC_HANDLE);

#[allow(dead_code)]
impl ServiceHandle {
  fn new(handle: SC_HANDLE) -> Option<Self> {
    if handle.is_invalid() {
      None
    } else {
      Some(Self(handle))
    }
  }
}

impl Drop for ServiceHandle {
  fn drop(&mut self) {
    unsafe {
      let _ = CloseServiceHandle(self.0);
    }
  }
}

struct ServiceConfig {
  command: Option<String>,
  exe_path: Option<String>,
}

struct RawServiceItem {
  name: String,
  display_name: String,
  exe_path: Option<String>,
  command: String,
  is_enabled: bool,
  is_delayed: bool,
  start_type: String,
}

fn open_scm(access: u32) -> Option<SC_HANDLE> {
  unsafe { OpenSCManagerW(None, None, access).ok() }
}

fn enum_all_services(manager: SC_HANDLE) -> Option<Vec<(String, String)>> {
  unsafe {
    let mut bytes_needed = 0u32;
    let mut services_returned = 0u32;
    let mut resume_handle = 0u32;

    let _ = EnumServicesStatusExW(
      manager,
      SC_ENUM_PROCESS_INFO,
      SERVICE_WIN32,
      SERVICE_STATE_ALL,
      None,
      &mut bytes_needed,
      &mut services_returned,
      Some(&mut resume_handle),
      PCWSTR::null(),
    );

    if bytes_needed == 0 {
      return Some(Vec::new());
    }

    let mut buffer = vec![0u8; bytes_needed as usize];
    EnumServicesStatusExW(
      manager,
      SC_ENUM_PROCESS_INFO,
      SERVICE_WIN32,
      SERVICE_STATE_ALL,
      Some(&mut buffer),
      &mut bytes_needed,
      &mut services_returned,
      Some(&mut resume_handle),
      PCWSTR::null(),
    )
    .ok()?;

    let mut result = Vec::with_capacity(services_returned as usize);
    let mut offset = 0usize;

    for _ in 0..services_returned {
      if offset + size_of::<ENUM_SERVICE_STATUS_PROCESSW>() > buffer.len() {
        break;
      }
      let service: ENUM_SERVICE_STATUS_PROCESSW =
        std::ptr::read_unaligned(buffer.as_ptr().add(offset) as *const _);
      offset += size_of::<ENUM_SERVICE_STATUS_PROCESSW>();

      let name = PCWSTR::from_raw(service.lpServiceName.as_ptr())
        .to_string()
        .unwrap_or_default();
      let display = PCWSTR::from_raw(service.lpDisplayName.as_ptr())
        .to_string()
        .unwrap_or_default();
      if !name.is_empty() {
        result.push((name, display));
      }
    }

    Some(result)
  }
}

fn query_service_config(name: &str) -> Option<ServiceConfig> {
  let manager = open_scm(SC_MANAGER_CONNECT)?;
  let _manager_guard = ServiceHandle(manager);

  unsafe {
    let name_wide: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();

    let service =
      OpenServiceW(manager, PCWSTR(name_wide.as_ptr()), SERVICE_QUERY_CONFIG)
        .ok()?;

    let _service_guard = ServiceHandle(service);

    let mut bytes_needed = 0u32;

    let _ = QueryServiceConfigW(service, None, 0, &mut bytes_needed);

    if bytes_needed == 0 {
      return None;
    }

    let mut buffer = vec![0u8; bytes_needed as usize];
    QueryServiceConfigW(
      service,
      Some(buffer.as_mut_ptr() as *mut QUERY_SERVICE_CONFIGW),
      bytes_needed,
      &mut bytes_needed,
    )
    .ok()?;

    let config: QUERY_SERVICE_CONFIGW =
      std::ptr::read_unaligned(buffer.as_ptr() as *const _);

    let binary_path = if !config.lpBinaryPathName.is_null() {
      PCWSTR::from_raw(config.lpBinaryPathName.as_ptr())
        .to_string()
        .unwrap_or_default()
    } else {
      String::new()
    };

    Some(ServiceConfig {
      command: Some(binary_path.clone()),
      exe_path: parse_exe_path(&binary_path),
    })
  }
}

fn parse_exe_path(raw: &str) -> Option<String> {
  let unquoted = raw.trim_matches('"');
  let lower = unquoted.to_ascii_lowercase();

  if let Some(exe_end) = lower.rfind(".exe") {
    let boundary = exe_end + 4;
    if boundary >= unquoted.len()
      || unquoted.as_bytes()[boundary].is_ascii_whitespace()
      || unquoted.as_bytes()[boundary] == b'"'
    {
      return Some(unquoted[..boundary].trim().to_string());
    }
  }

  if !unquoted.is_empty() {
    Some(unquoted.to_string())
  } else {
    None
  }
}

fn read_start_type_from_registry(name: &str) -> (u32, bool) {
  let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
  let path = format!(r"SYSTEM\CurrentControlSet\Services\{name}");

  if let Ok(key) = hklm.open_subkey(path) {
    let start: u32 = key.get_value("Start").unwrap_or(3);
    let delayed: u32 = key.get_value("DelayedAutoStart").unwrap_or(0);
    (start, delayed == 1)
  } else {
    (3, false)
  }
}

fn get_start_type_badge(start: u32, is_delayed: bool) -> (bool, String) {
  match start {
    0 => (true, "Boot".to_string()),
    1 => (true, "System".to_string()),
    2 => (
      true,
      if is_delayed {
        "Delayed".to_string()
      } else {
        "Auto".to_string()
      },
    ),
    3 => (false, "Manual".to_string()),
    4 => (false, "Disabled".to_string()),
    _ => (false, "Unknown".to_string()),
  }
}

fn is_ignored_service(name: &str) -> bool {
  IGNORED_SERVICES
    .iter()
    .any(|s| s.eq_ignore_ascii_case(name))
}

fn exe_name_from_raw<'a>(
  exe_path: &'a Option<String>,
  fallback: &'a str,
) -> &'a str {
  exe_path
    .as_ref()
    .and_then(|p| p.rsplit(['\\', '/']).next())
    .unwrap_or(fallback)
}

fn extract_service_name(id: &str) -> Result<String, String> {
  let parts: Vec<&str> = id.splitn(2, '|').collect();
  if parts.len() != 2 || parts[0] != "service" {
    return Err("Invalid service item ID".to_string());
  }
  Ok(parts[1].to_string())
}

fn collect_raw_service_items() -> Vec<RawServiceItem> {
  let manager = match open_scm(SC_MANAGER_ENUMERATE_SERVICE) {
    Some(m) => m,
    None => {
      warn!("[services] Failed to open SCM");
      return Vec::new();
    }
  };
  let _manager_guard = ServiceHandle(manager);

  let services = match enum_all_services(manager) {
    Some(s) => s,
    None => {
      warn!("[services] Failed to enumerate services");
      return Vec::new();
    }
  };

  info!("[services] Found {} services", services.len());

  let ignored_count = AtomicUsize::new(0);
  let error_count = AtomicUsize::new(0);

  let items: Vec<RawServiceItem> = services
    .into_par_iter()
    .filter_map(|(name, display_name)| {
      if is_ignored_service(&name) {
        ignored_count.fetch_add(1, Ordering::Relaxed);
        return None;
      }

      let config = match query_service_config(&name) {
        Some(c) => c,
        None => {
          error_count.fetch_add(1, Ordering::Relaxed);
          return None;
        }
      };

      let (start_type, is_delayed) = read_start_type_from_registry(&name);

      let (is_enabled, start_type_badge) =
        get_start_type_badge(start_type, is_delayed);

      Some(RawServiceItem {
        name,
        display_name,
        exe_path: config.exe_path,
        command: config.command.unwrap_or_default(),
        is_enabled,
        is_delayed,
        start_type: start_type_badge,
      })
    })
    .collect();

  info!(
    "[services] Collected {} services (ignored: {}, errors: {})",
    items.len(),
    ignored_count.load(Ordering::Relaxed),
    error_count.load(Ordering::Relaxed)
  );

  items
}

pub fn get_service_autostart_items() -> Vec<AutostartItem> {
  let raw_items = collect_raw_service_items();

  raw_items
    .into_par_iter()
    .map(|raw| {
      use crate::autostart::file_info::get_file_version_info;
      use crate::autostart::icons::get_icon;

      let icon_base64 = raw.exe_path.as_ref().and_then(|p| get_icon(p));
      let exe_name = exe_name_from_raw(&raw.exe_path, &raw.name);
      let critical_level = get_critical_level(exe_name, &raw.command);

      let publisher = raw
        .exe_path
        .as_ref()
        .and_then(|p| get_file_version_info(p).ok())
        .and_then(|v| v.company_name)
        .unwrap_or_default();

      let id = format!("service|{}", raw.name);

      AutostartItem {
        id,
        name: if raw.display_name.is_empty() {
          raw.name.clone()
        } else {
          raw.display_name.clone()
        },
        publisher,
        command: raw.command,
        location: format!("Service: {}", raw.name),
        source: AutostartSource::Service,
        is_enabled: raw.is_enabled,
        is_delayed: raw.is_delayed,
        icon_base64,
        critical_level,
        file_path: raw.exe_path,
        start_type: Some(raw.start_type),
      }
    })
    .collect()
}

pub fn get_service_items_fast() -> Vec<AutostartItem> {
  let raw_items = collect_raw_service_items();

  raw_items
    .into_par_iter()
    .map(|raw| {
      let exe_name = exe_name_from_raw(&raw.exe_path, &raw.name);
      let critical_level = get_critical_level(exe_name, &raw.command);

      let id = format!("service|{}", raw.name);

      AutostartItem {
        id,
        name: if raw.display_name.is_empty() {
          raw.name.clone()
        } else {
          raw.display_name.clone()
        },
        publisher: String::new(),
        command: raw.command,
        location: format!("Service: {}", raw.name),
        source: AutostartSource::Service,
        is_enabled: raw.is_enabled,
        is_delayed: raw.is_delayed,
        icon_base64: None,
        critical_level,
        file_path: raw.exe_path,
        start_type: Some(raw.start_type),
      }
    })
    .collect()
}

pub fn toggle_service_item(id: &str, enable: bool) -> Result<(), String> {
  let name = extract_service_name(id)?;

  let manager = open_scm(SC_MANAGER_ALL_ACCESS).ok_or("Failed to open SCM")?;
  let _manager_guard = ServiceHandle(manager);

  unsafe {
    let name_wide: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();

    let service = OpenServiceW(
      manager,
      PCWSTR(name_wide.as_ptr()),
      SERVICE_CHANGE_CONFIG | SERVICE_START | SERVICE_STOP,
    )
    .map_err(|e| format!("Failed to open service: {e}"))?;

    let _service_guard = ServiceHandle(service);

    if enable {
      ChangeServiceConfigW(
        service,
        ENUM_SERVICE_TYPE(SERVICE_NO_CHANGE),
        SERVICE_AUTO_START,
        SERVICE_ERROR(SERVICE_NO_CHANGE),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
      )
      .map_err(|e| format!("Failed to change config: {e}"))?;

      let _ = StartServiceW(service, None);
    } else {
      let mut status = SERVICE_STATUS::default();
      let _ = ControlService(service, SERVICE_CONTROL_STOP, &mut status);

      ChangeServiceConfigW(
        service,
        ENUM_SERVICE_TYPE(SERVICE_NO_CHANGE),
        SERVICE_DISABLED,
        SERVICE_ERROR(SERVICE_NO_CHANGE),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
      )
      .map_err(|e| format!("Failed to disable: {e}"))?;
    }

    Ok(())
  }
}

pub fn delete_service_item(id: &str) -> Result<(), String> {
  let name = extract_service_name(id)?;

  let manager = open_scm(SC_MANAGER_ALL_ACCESS).ok_or("Failed to open SCM")?;
  let _manager_guard = ServiceHandle(manager);

  unsafe {
    let name_wide: Vec<u16> = name.encode_utf16().chain(Some(0)).collect();

    let service = OpenServiceW(
      manager,
      PCWSTR(name_wide.as_ptr()),
      DELETE_ACCESS | SERVICE_STOP,
    )
    .map_err(|e| format!("Failed to open service: {e}"))?;

    let _service_guard = ServiceHandle(service);

    let mut status = SERVICE_STATUS::default();
    let _ = ControlService(service, SERVICE_CONTROL_STOP, &mut status);

    DeleteService(service).map_err(|e| format!("Failed to delete: {e}"))?;

    Ok(())
  }
}
