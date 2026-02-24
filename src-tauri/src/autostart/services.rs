use std::process::Command;

use winreg::RegKey;
use winreg::enums::*;

use crate::autostart::critical::get_critical_level;
use crate::autostart::file_info::get_file_version_info;
use crate::autostart::icons::get_icon;
use crate::autostart::types::{AutostartItem, AutostartSource};

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

  let stdout = String::from_utf8_lossy(&output.stdout);

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

pub fn get_service_autostart_items() -> Vec<AutostartItem> {
  let mut items = Vec::new();

  let output = match Command::new("sc")
    .args(["query", "type=", "service", "state=", "all"])
    .output()
  {
    Ok(o) => o,
    Err(_) => return items,
  };

  let stdout = String::from_utf8_lossy(&output.stdout);
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

    let exe_path = config.exe_path;
    let command = config.command.clone().unwrap_or_default();
    let icon_base64 = exe_path.as_ref().and_then(|p| get_icon(p));

    let is_enabled = is_auto;

    let exe_name = exe_path
      .as_ref()
      .and_then(|p| p.rsplit(|c| c == '\\' || c == '/').next())
      .unwrap_or(&name);
    let critical_level = get_critical_level(exe_name, &command);

    let publisher = exe_path
      .as_ref()
      .and_then(|p| get_file_version_info(p).ok())
      .and_then(|v| v.company_name)
      .unwrap_or_default();

    let id = format!("service|{}", name);

    items.push(AutostartItem {
      id,
      name: if display_name.is_empty() {
        name.clone()
      } else {
        display_name.clone()
      },
      publisher,
      command,
      location: format!("Service: {}", name),
      source: AutostartSource::Service,
      is_enabled,
      is_delayed: start_info.is_delayed,
      icon_base64,
      critical_level,
      file_path: exe_path,
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
      let stderr = String::from_utf8_lossy(&output.stderr);
      let stdout = String::from_utf8_lossy(&output.stdout);
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
      let stderr = String::from_utf8_lossy(&output.stderr);
      let stdout = String::from_utf8_lossy(&output.stdout);
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
    let stdout = String::from_utf8_lossy(&stop_output.stdout);
    if !stdout.contains("not been started")
      && !stdout.contains("is not started")
    {
      let stderr = String::from_utf8_lossy(&stop_output.stderr);
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
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let msg = if !stderr.trim().is_empty() {
      stderr.trim()
    } else {
      stdout.trim()
    };
    return Err(format!("Failed to delete service: {}", msg));
  }

  Ok(())
}
