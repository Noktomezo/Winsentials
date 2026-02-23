use std::process::Command;

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

struct ServiceConfig {
  start_type: String,
  is_delayed: bool,
  exe_path: Option<String>,
}

fn query_service_config(name: &str) -> Option<ServiceConfig> {
  let output = Command::new("sc").args(["qc", name]).output().ok()?;

  let stdout = String::from_utf8_lossy(&output.stdout);

  let mut start_type = String::new();
  let mut is_delayed = false;
  let mut exe_path = None;

  for line in stdout.lines() {
    let line = line.trim();
    if line.starts_with("START_TYPE") {
      if let Some((_, value)) = line.split_once(':') {
        let trimmed = value.trim();
        start_type = trimmed.to_string();
        is_delayed = trimmed.to_lowercase().contains("(delayed)");
      }
    } else if line.starts_with("BINARY_PATH_NAME") {
      let path = line.split_once(':').map(|x| x.1).unwrap_or("").trim();
      let path = path.trim_matches('"');

      let lower_path = path.to_lowercase();
      if let Some(exe_idx) = lower_path.find(".exe") {
        exe_path = Some(path[..exe_idx + 4].to_string());
      } else {
        exe_path = Some(path.to_string());
      }
    }
  }

  Some(ServiceConfig {
    start_type,
    is_delayed,
    exe_path,
  })
}

fn parse_sc_output(output: &str) -> Vec<(String, String, String)> {
  let mut services = Vec::new();
  let mut current_name = String::new();
  let mut current_display = String::new();
  let mut current_state = String::new();

  for line in output.lines() {
    let line = line.trim();

    if line.starts_with("SERVICE_NAME:") {
      if !current_name.is_empty() {
        services.push((
          current_name.clone(),
          current_display.clone(),
          current_state.clone(),
        ));
      }
      current_name =
        line.trim_start_matches("SERVICE_NAME:").trim().to_string();
      current_display = String::new();
      current_state = String::new();
    } else if line.starts_with("DISPLAY_NAME:") {
      current_display =
        line.trim_start_matches("DISPLAY_NAME:").trim().to_string();
    } else if line.starts_with("STATE") {
      if let Some((_, value)) = line.split_once(':') {
        current_state = value.trim().to_string();
      }
    }
  }

  if !current_name.is_empty() {
    services.push((current_name, current_display, current_state));
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

  for (name, display_name, _state) in services {
    if IGNORED_SERVICES.contains(&name.as_str()) {
      continue;
    }

    let config = match query_service_config(&name) {
      Some(c) => c,
      None => continue,
    };

    let is_auto = config.start_type.to_lowercase().contains("auto");
    let is_disabled = config.start_type.to_lowercase().contains("disabled");
    if !is_auto && !is_disabled {
      continue;
    }

    let exe_path = config.exe_path;
    let icon_base64 = exe_path.as_ref().and_then(|p| get_icon(p));

    let is_enabled = is_auto;

    let command = exe_path.clone().unwrap_or_default();
    let critical_level = get_critical_level(&name, &command);

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
      command: exe_path.clone().unwrap_or_default(),
      location: format!("Service: {}", name),
      source: AutostartSource::Service,
      is_enabled,
      is_delayed: config.is_delayed,
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
    let config = query_service_config(service_name)
      .ok_or_else(|| "Failed to query service config".to_string())?;

    let start_value = if config.is_delayed {
      "delayed-auto"
    } else {
      "auto"
    };

    let output = Command::new("sc")
      .args(["config", service_name, "start=", start_value])
      .output()
      .map_err(|e| format!("Failed to execute sc: {}", e))?;

    if !output.status.success() {
      let stdout = String::from_utf8_lossy(&output.stdout);
      return Err(format!("Failed to enable service: {}", stdout.trim()));
    }

    let _ = Command::new("sc").args(["start", service_name]).output();
  } else {
    let _ = Command::new("sc").args(["stop", service_name]).output();

    let output = Command::new("sc")
      .args(["config", service_name, "start=", "disabled"])
      .output()
      .map_err(|e| format!("Failed to execute sc: {}", e))?;

    if !output.status.success() {
      let stdout = String::from_utf8_lossy(&output.stdout);
      return Err(format!("Failed to disable service: {}", stdout.trim()));
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

  let output = Command::new("sc")
    .args(["delete", service_name])
    .output()
    .map_err(|e| format!("Failed to execute sc: {}", e))?;

  if !output.status.success() {
    let stdout = String::from_utf8_lossy(&output.stdout);
    return Err(format!("Failed to delete service: {}", stdout.trim()));
  }

  Ok(())
}
