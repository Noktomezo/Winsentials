use std::process::Command;

use crate::autostart::critical::get_critical_level;
use crate::autostart::file_info::get_file_version_info;
use crate::autostart::icons::get_icon;
use crate::autostart::types::{AutostartItem, AutostartSource};

const STARTUP_TRIGGERS: [&str; 4] =
  ["AtLogon", "AtStartup", "At system startup", "At logon"];

fn parse_csv_line(line: &str) -> Vec<String> {
  let mut fields = Vec::new();
  let mut current = String::new();
  let mut in_quotes = false;

  for ch in line.chars() {
    match ch {
      '"' => {
        in_quotes = !in_quotes;
      }
      ',' if !in_quotes => {
        fields.push(current.trim().trim_matches('"').to_string());
        current = String::new();
      }
      _ => {
        current.push(ch);
      }
    }
  }
  fields.push(current.trim().trim_matches('"').to_string());
  fields
}

pub fn get_task_autostart_items() -> Vec<AutostartItem> {
  let mut items = Vec::new();

  let output = match Command::new("schtasks")
    .args(["/query", "/fo", "CSV", "/v"])
    .output()
  {
    Ok(o) => o,
    Err(_) => return items,
  };

  let stdout = String::from_utf8_lossy(&output.stdout);
  let lines: Vec<&str> = stdout.lines().collect();

  if lines.is_empty() {
    return items;
  }

  let headers = parse_csv_line(lines[0]);
  let name_idx = headers
    .iter()
    .position(|h| h.to_lowercase().contains("taskname"));
  let trigger_idx = headers.iter().position(|h| {
    h.to_lowercase().contains("next run")
      || h.to_lowercase().contains("trigger")
  });
  let command_idx = headers.iter().position(|h| {
    h.to_lowercase().contains("task to run")
      || h.to_lowercase().contains("command")
  });
  let status_idx = headers
    .iter()
    .position(|h| h.to_lowercase().contains("status"));

  for line in lines.iter().skip(1) {
    let fields = parse_csv_line(line);

    if fields.is_empty() {
      continue;
    }

    let name = name_idx
      .and_then(|i| fields.get(i))
      .map(|s| s.trim_start_matches('\\').to_string())
      .unwrap_or_default();

    if name.is_empty() || name.starts_with("Microsoft") {
      continue;
    }

    let trigger = trigger_idx
      .and_then(|i| fields.get(i))
      .map(|s| s.to_string())
      .unwrap_or_default();

    let is_startup = STARTUP_TRIGGERS.iter().any(|t| trigger.contains(t));

    if !is_startup {
      continue;
    }

    let command = command_idx
      .and_then(|i| fields.get(i))
      .map(|s| s.to_string())
      .unwrap_or_default();

    let status = status_idx
      .and_then(|i| fields.get(i))
      .map(|s| s.to_lowercase())
      .unwrap_or_default();

    let is_enabled = !status.contains("disabled");
    let is_delayed = trigger.to_lowercase().contains("delay");

    let target_path = extract_exe_from_command(&command);
    let icon_base64 = target_path.as_ref().and_then(|p| get_icon(p));

    let publisher = target_path
      .as_ref()
      .and_then(|p| get_file_version_info(p).ok())
      .and_then(|v| v.company_name)
      .unwrap_or_default();

    let critical_level = get_critical_level(&name, &command);

    let id = format!("task|{}", name.replace('\\', "/"));

    items.push(AutostartItem {
      id,
      name: name.split('\\').last().unwrap_or(&name).to_string(),
      publisher,
      command,
      location: format!("Task: {}", name),
      source: AutostartSource::Task,
      is_enabled,
      is_delayed,
      icon_base64,
      critical_level,
      file_path: target_path,
    });
  }

  items
}

fn extract_exe_from_command(command: &str) -> Option<String> {
  let cmd = command.trim();

  if cmd.is_empty()
    || cmd == "COM"
    || cmd.to_lowercase().contains("custom handler")
  {
    return None;
  }

  if cmd.starts_with('"') {
    if let Some(end) = cmd[1..].find('"') {
      return Some(cmd[1..end + 1].to_string());
    }
  }

  let parts: Vec<&str> = cmd.split_whitespace().collect();
  if !parts.is_empty() {
    let exe = parts[0];
    if exe.to_lowercase().ends_with(".exe") {
      return Some(exe.to_string());
    }
  }

  None
}

pub fn toggle_task_item(id: &str, enable: bool) -> Result<(), String> {
  let parts: Vec<&str> = id.split('|').collect();
  if parts.len() != 2 || parts[0] != "task" {
    return Err("Invalid task item ID".to_string());
  }

  let task_name = parts[1].replace('/', "\\");

  let status = if enable { "enable" } else { "disable" };

  let output = Command::new("schtasks")
    .args(["/change", "/tn", &task_name, &format!("/{}", status)])
    .output()
    .map_err(|e| format!("Failed to execute schtasks: {}", e))?;

  if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    return Err(format!("Failed to {} task: {}", status, stderr));
  }

  Ok(())
}

pub fn delete_task_item(id: &str) -> Result<(), String> {
  let parts: Vec<&str> = id.split('|').collect();
  if parts.len() != 2 || parts[0] != "task" {
    return Err("Invalid task item ID".to_string());
  }

  let task_name = parts[1].replace('/', "\\");

  let output = Command::new("schtasks")
    .args(["/delete", "/tn", &task_name, "/f"])
    .output()
    .map_err(|e| format!("Failed to execute schtasks: {}", e))?;

  if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    return Err(format!("Failed to delete task: {}", stderr));
  }

  Ok(())
}
