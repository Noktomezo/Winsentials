use std::os::windows::ffi::OsStringExt;
use std::process::Command;

use rayon::prelude::*;
use windows::Win32::Globalization::{
  MultiByteToWideChar, CP_OEMCP, MB_PRECOMPOSED,
};
use winreg::enums::*;
use winreg::RegKey;

use crate::autostart::critical::get_critical_level;
use crate::autostart::file_info::get_file_version_info;
use crate::autostart::icons::get_icon;
use crate::autostart::types::{AutostartItem, AutostartSource, EnrichmentData};

fn oem_to_utf8(bytes: &[u8]) -> String {
  if bytes.is_empty() {
    return String::new();
  }

  unsafe {
    let required_len =
      MultiByteToWideChar(CP_OEMCP, MB_PRECOMPOSED, bytes, None);
    if required_len <= 0 {
      return String::from_utf8_lossy(bytes).to_string();
    }

    let mut buffer = vec![0u16; required_len as usize];
    let len =
      MultiByteToWideChar(CP_OEMCP, MB_PRECOMPOSED, bytes, Some(&mut buffer));
    if len <= 0 {
      return String::from_utf8_lossy(bytes).to_string();
    }

    std::ffi::OsString::from_wide(&buffer[..len as usize])
      .to_string_lossy()
      .to_string()
  }
}

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

fn is_ignored_service(name: &str) -> bool {
  IGNORED_SERVICES
    .iter()
    .any(|s| s.eq_ignore_ascii_case(name))
}

struct ServiceStartInfo {
  start: u32,
  is_delayed: bool,
}

fn read_service_start_info(name: &str) -> Option<ServiceStartInfo> {
  let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
  let path = format!(r"SYSTEM\CurrentControlSet\Services\{}", name);
  let key = hklm.open_subkey(path).ok()?;

  let start: u32 = key.get_value("Start").ok()?;
  let is_delayed: u32 = key.get_value("DelayedAutoStart").unwrap_or(0);

  Some(ServiceStartInfo {
    start,
    is_delayed: is_delayed == 1,
  })
}

struct ServiceConfig {
  command: Option<String>,
  exe_path: Option<String>,
}

fn query_service_config(name: &str) -> Option<ServiceConfig> {
  let output = Command::new("sc").args(["qc", name]).output().ok()?;

  if !output.status.success() {
    return None;
  }

  let stdout = oem_to_utf8(&output.stdout);

  let mut command = None;
  let mut exe_path = None;

  for line in stdout.lines() {
    let line = line.trim();
    if line.starts_with("BINARY_PATH_NAME") {
      let raw = line.split_once(':').map(|x| x.1).unwrap_or("").trim();
      command = Some(raw.to_string());

      let unquoted = raw.trim_matches('"');
      let lower_unquoted = unquoted.to_ascii_lowercase();
      if let Some(exe_end) = lower_unquoted.rfind(".exe") {
        let boundary_idx = exe_end + 4;
        if boundary_idx >= unquoted.len()
          || unquoted.as_bytes()[boundary_idx].is_ascii_whitespace()
          || unquoted.as_bytes()[boundary_idx] == b'"'
          || unquoted.as_bytes()[boundary_idx] == b'\''
        {
          exe_path = Some(unquoted[..boundary_idx].trim().to_string());
        } else {
          exe_path = Some(unquoted.to_string());
        }
      } else {
        exe_path = Some(unquoted.to_string());
      }
    }
  }

  Some(ServiceConfig { command, exe_path })
}

fn parse_sc_output(output: &str) -> Vec<(String, String)> {
  let mut services = Vec::new();
  let mut current_name = String::new();
  let mut current_display = String::new();

  for line in output.lines() {
    let line = line.trim();

    if line.starts_with("SERVICE_NAME:") {
      if !current_name.is_empty() {
        services.push((current_name.clone(), current_display.clone()));
      }
      current_name =
        line.trim_start_matches("SERVICE_NAME:").trim().to_string();
      current_display = String::new();
    } else if line.starts_with("DISPLAY_NAME:") {
      current_display =
        line.trim_start_matches("DISPLAY_NAME:").trim().to_string();
    }
  }

  if !current_name.is_empty() {
    services.push((current_name, current_display));
  }

  services
}

struct RawServiceItem {
  name: String,
  display_name: String,
  exe_path: Option<String>,
  command: String,
  is_enabled: bool,
  is_delayed: bool,
}

fn enrich_service_item(raw: RawServiceItem) -> AutostartItem {
  let icon_base64 = raw.exe_path.as_ref().and_then(|p| get_icon(p));

  let exe_name = raw
    .exe_path
    .as_ref()
    .and_then(|p| p.rsplit(|c| c == '\\' || c == '/').next())
    .unwrap_or(&raw.name);
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
  }
}

pub fn get_service_autostart_items() -> Vec<AutostartItem> {
  let raw_items = collect_raw_service_items();

  raw_items.into_par_iter().map(enrich_service_item).collect()
}

pub fn get_service_items_fast() -> Vec<AutostartItem> {
  let raw_items = collect_raw_service_items();

  raw_items
    .into_iter()
    .map(|raw| {
      let exe_name = raw
        .exe_path
        .as_ref()
        .and_then(|p| p.rsplit(|c| c == '\\' || c == '/').next())
        .unwrap_or(&raw.name);
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
      }
    })
    .collect()
}

pub fn enrich_service_items(
  items: &mut [AutostartItem],
) -> Vec<EnrichmentData> {
  items
    .iter()
    .filter(|item| item.source == AutostartSource::Service)
    .map(|item| {
      let icon_base64 = item.file_path.as_ref().and_then(|p| get_icon(p));
      let publisher = item
        .file_path
        .as_ref()
        .and_then(|p| get_file_version_info(p).ok())
        .and_then(|v| v.company_name)
        .unwrap_or_default();

      EnrichmentData {
        id: item.id.clone(),
        icon_base64,
        publisher,
      }
    })
    .collect()
}

fn collect_raw_service_items() -> Vec<RawServiceItem> {
  let mut items = Vec::new();

  let output = match Command::new("sc")
    .args(["query", "type=", "service", "state=", "all"])
    .output()
  {
    Ok(o) => o,
    Err(_) => return items,
  };

  let stdout = oem_to_utf8(&output.stdout);
  let services = parse_sc_output(&stdout);

  for (name, display_name) in services {
    if is_ignored_service(&name) {
      continue;
    }

    let start_info = match read_service_start_info(&name) {
      Some(info) => info,
      None => continue,
    };

    let is_auto = start_info.start == 2;
    let is_disabled = start_info.start == 4;
    if !is_auto && !is_disabled {
      continue;
    }

    let config = match query_service_config(&name) {
      Some(c) => c,
      None => continue,
    };

    items.push(RawServiceItem {
      name,
      display_name,
      exe_path: config.exe_path,
      command: config.command.unwrap_or_default(),
      is_enabled: is_auto,
      is_delayed: start_info.is_delayed,
    });
  }

  items
}

pub fn toggle_service_item(id: &str, enable: bool) -> Result<(), String> {
  let parts: Vec<&str> = id.splitn(2, '|').collect();
  if parts.len() != 2 || parts[0] != "service" {
    return Err("Invalid service item ID".to_string());
  }

  let service_name = parts[1];

  if enable {
    let start_info =
      read_service_start_info(service_name).ok_or_else(|| {
        "Failed to read service config from registry".to_string()
      })?;

    let start_value = if start_info.is_delayed {
      "delayed-auto"
    } else {
      "auto"
    };

    let output = Command::new("sc")
      .args(["config", service_name, "start=", start_value])
      .output()
      .map_err(|e| format!("Failed to execute sc: {}", e))?;

    if !output.status.success() {
      let stderr = oem_to_utf8(&output.stderr);
      let stdout = oem_to_utf8(&output.stdout);
      let msg = if !stderr.trim().is_empty() {
        stderr.trim()
      } else {
        stdout.trim()
      };
      return Err(format!("Failed to enable service: {}", msg));
    }

    let _ = Command::new("sc").args(["start", service_name]).output();
  } else {
    let _ = Command::new("sc").args(["stop", service_name]).output();

    let output = Command::new("sc")
      .args(["config", service_name, "start=", "disabled"])
      .output()
      .map_err(|e| format!("Failed to execute sc: {}", e))?;

    if !output.status.success() {
      let stderr = oem_to_utf8(&output.stderr);
      let stdout = oem_to_utf8(&output.stdout);
      let msg = if !stderr.trim().is_empty() {
        stderr.trim()
      } else {
        stdout.trim()
      };
      return Err(format!("Failed to disable service: {}", msg));
    }
  }

  Ok(())
}

pub fn delete_service_item(id: &str) -> Result<(), String> {
  let parts: Vec<&str> = id.splitn(2, '|').collect();
  if parts.len() != 2 || parts[0] != "service" {
    return Err("Invalid service item ID".to_string());
  }

  let service_name = parts[1];

  let stop_output = Command::new("sc")
    .args(["stop", service_name])
    .output()
    .map_err(|e| format!("Failed to execute sc: {}", e))?;

  if !stop_output.status.success() {
    let stdout = oem_to_utf8(&stop_output.stdout);
    if !stdout.contains("not been started")
      && !stdout.contains("is not started")
    {
      let stderr = oem_to_utf8(&stop_output.stderr);
      let msg = if !stderr.trim().is_empty() {
        stderr.trim()
      } else {
        stdout.trim()
      };
      return Err(format!("Failed to stop service: {}", msg));
    }
  }

  let output = Command::new("sc")
    .args(["delete", service_name])
    .output()
    .map_err(|e| format!("Failed to execute sc: {}", e))?;

  if !output.status.success() {
    let stderr = oem_to_utf8(&output.stderr);
    let stdout = oem_to_utf8(&output.stdout);
    let msg = if !stderr.trim().is_empty() {
      stderr.trim()
    } else {
      stdout.trim()
    };
    return Err(format!("Failed to delete service: {}", msg));
  }

  Ok(())
}
