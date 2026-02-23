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

fn parse_sc_output(
  output: &str,
) -> Vec<(String, String, String, String, bool)> {
  let mut services = Vec::new();
  let mut current_name = String::new();
  let mut current_display = String::new();
  let mut current_state = String::new();
  let mut current_start_type = String::new();

  for line in output.lines() {
    let line = line.trim();

    if line.starts_with("SERVICE_NAME:") {
      if !current_name.is_empty() {
        let is_auto = current_start_type.to_lowercase().contains("auto");
        if is_auto {
          let is_delayed = current_start_type.to_lowercase().contains("delay");
          services.push((
            current_name.clone(),
            current_display.clone(),
            current_state.clone(),
            current_start_type.clone(),
            is_delayed,
          ));
        }
      }
      current_name =
        line.trim_start_matches("SERVICE_NAME:").trim().to_string();
      current_display = String::new();
      current_state = String::new();
      current_start_type = String::new();
    } else if line.starts_with("DISPLAY_NAME:") {
      current_display =
        line.trim_start_matches("DISPLAY_NAME:").trim().to_string();
    } else if line.starts_with("STATE") {
      current_state = line.split_whitespace().nth(2).unwrap_or("").to_string();
    } else if line.starts_with("START_TYPE") {
      current_start_type =
        line.split_whitespace().nth(2).unwrap_or("").to_string();
    }
  }

  if !current_name.is_empty() {
    let is_auto = current_start_type.to_lowercase().contains("auto");
    if is_auto {
      let is_delayed = current_start_type.to_lowercase().contains("delay");
      services.push((
        current_name,
        current_display,
        current_state,
        current_start_type,
        is_delayed,
      ));
    }
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

  for (name, display_name, state, start_type, is_delayed) in services {
    if IGNORED_SERVICES.contains(&name.as_str()) {
      continue;
    }

    let exe_path = get_service_path(&name);
    let icon_base64 = exe_path.as_ref().and_then(|p| get_icon(p));

    let is_enabled = state.to_lowercase().contains("running")
      || start_type.to_lowercase().contains("auto");

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
      is_delayed,
      icon_base64,
      critical_level,
      file_path: exe_path,
    });
  }

  items
}

fn get_service_path(name: &str) -> Option<String> {
  let output = Command::new("sc").args(["qc", name]).output().ok()?;

  let stdout = String::from_utf8_lossy(&output.stdout);

  for line in stdout.lines() {
    let line = line.trim();
    if line.starts_with("BINARY_PATH_NAME") {
      let path = line.split_once(':').map(|x| x.1).unwrap_or("").trim();
      let path = path.trim_matches('"');

      if path.to_lowercase().ends_with(".exe")
        && let Some(space_idx) = path.find(".exe")
      {
        return Some(path[..space_idx + 4].to_string());
      }
      return Some(path.to_string());
    }
  }

  None
}

pub fn toggle_service_item(id: &str, enable: bool) -> Result<(), String> {
  let parts: Vec<&str> = id.split('|').collect();
  if parts.len() != 2 || parts[0] != "service" {
    return Err("Invalid service item ID".to_string());
  }

  let service_name = parts[1];

  if enable {
    let output = Command::new("sc")
      .args(["config", service_name, "start=", "auto"])
      .output()
      .map_err(|e| format!("Failed to execute sc: {}", e))?;

    if !output.status.success() {
      let stderr = String::from_utf8_lossy(&output.stderr);
      return Err(format!("Failed to enable service: {}", stderr));
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
      return Err(format!("Failed to disable service: {}", stderr));
    }
  }

  Ok(())
}

pub fn delete_service_item(id: &str) -> Result<(), String> {
  let parts: Vec<&str> = id.split('|').collect();
  if parts.len() != 2 || parts[0] != "service" {
    return Err("Invalid service item ID".to_string());
  }

  let service_name = parts[1];

  let output = Command::new("sc")
    .args(["delete", service_name])
    .output()
    .map_err(|e| format!("Failed to execute sc: {}", e))?;

  if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    return Err(format!("Failed to delete service: {}", stderr));
  }

  Ok(())
}
