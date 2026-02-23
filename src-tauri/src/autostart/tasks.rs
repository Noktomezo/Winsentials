use std::process::Command;

use quick_xml::Reader;
use quick_xml::events::Event;

use crate::autostart::critical::get_critical_level;
use crate::autostart::file_info::get_file_version_info;
use crate::autostart::icons::get_icon;
use crate::autostart::types::{AutostartItem, AutostartSource};

const STARTUP_TRIGGERS: &[&str] = &["LogonTrigger", "BootTrigger"];
const STARTUP_TRIGGERS_BYTES: &[&[u8]] = &[b"LogonTrigger", b"BootTrigger"];

struct TaskInfo {
  name: String,
  command: String,
  state: String,
  triggers: Vec<String>,
  is_delayed: bool,
}

fn parse_task_xml(xml: &str) -> Option<TaskInfo> {
  let mut reader = Reader::from_str(xml);
  reader.config_mut().trim_text(true);

  let mut name = String::new();
  let mut command = String::new();
  let mut state = String::new();
  let mut triggers = Vec::new();
  let mut is_delayed = false;

  let mut in_registration_info = false;
  let mut in_uri = false;
  let mut in_exec = false;
  let mut in_command = false;
  let mut in_state = false;
  let mut in_triggers = false;
  let mut in_settings = false;
  let mut in_enabled_setting = false;
  let mut in_delay = false;
  let mut current_trigger_is_startup = false;

  let mut buf = Vec::new();

  loop {
    match reader.read_event_into(&mut buf) {
      Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
        match e.local_name().as_ref() {
          b"RegistrationInfo" => in_registration_info = true,
          b"URI" if in_registration_info => in_uri = true,
          b"Exec" => in_exec = true,
          b"Command" if in_exec => in_command = true,
          b"State" => in_state = true,
          b"Triggers" => in_triggers = true,
          b"Settings" => in_settings = true,
          b"Enabled" if in_settings => in_enabled_setting = true,
          b"Delay" => in_delay = true,
          trigger_name if in_triggers => {
            if STARTUP_TRIGGERS_BYTES.contains(&trigger_name) {
              current_trigger_is_startup = true;
              let trigger_name_str =
                String::from_utf8_lossy(trigger_name).to_string();
              triggers.push(trigger_name_str);
            }
          }
          _ => {}
        }
      }
      Ok(Event::End(ref e)) => match e.local_name().as_ref() {
        b"RegistrationInfo" => in_registration_info = false,
        b"URI" => in_uri = false,
        b"Exec" => in_exec = false,
        b"Command" => in_command = false,
        b"State" => in_state = false,
        b"Triggers" => in_triggers = false,
        b"Settings" => in_settings = false,
        b"Enabled" => in_enabled_setting = false,
        b"Delay" => {
          in_delay = false;
          if current_trigger_is_startup {
            current_trigger_is_startup = false;
          }
        }
        b"LogonTrigger" | b"BootTrigger" => {
          current_trigger_is_startup = false;
        }
        _ => {}
      },
      Ok(Event::Text(ref e)) => {
        let text = e.unescape().unwrap_or_default();
        if in_uri && name.is_empty() {
          name = text.to_string();
        }
        if in_command {
          command = text.to_string();
        }
        if in_state {
          state = text.to_string();
        }
        if in_enabled_setting && text.to_lowercase() == "false" {
          state = "Disabled".to_string();
        }
        if in_delay && current_trigger_is_startup && !text.is_empty() {
          is_delayed = true;
        }
      }
      Ok(Event::Eof) => break,
      Err(e) => {
        eprintln!("XML parsing error: {:?}", e);
        break;
      }
      _ => {}
    }
    buf.clear();
  }

  if name.is_empty() {
    return None;
  }

  Some(TaskInfo {
    name,
    command,
    state,
    triggers,
    is_delayed,
  })
}

fn get_tasks_xml() -> Option<String> {
  let output = Command::new("schtasks")
    .args(["/query", "/xml"])
    .output()
    .ok()?;

  if !output.status.success() {
    return None;
  }

  let stdout = &output.stdout;
  if stdout.len() >= 2 && stdout[0] == 0xFF && stdout[1] == 0xFE {
    let utf16_chars: Vec<u16> = stdout[2..]
      .chunks_exact(2)
      .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
      .collect();
    String::from_utf16(&utf16_chars).ok()
  } else {
    Some(String::from_utf8_lossy(stdout).to_string())
  }
}

pub fn get_task_autostart_items() -> Vec<AutostartItem> {
  let mut items = Vec::new();

  let xml = match get_tasks_xml() {
    Some(x) => x,
    None => return items,
  };

  let mut reader = Reader::from_str(&xml);
  reader.config_mut().trim_text(true);

  let mut buf = Vec::new();
  let mut task_xml = String::new();
  let mut in_task = false;
  let mut depth = 0;

  loop {
    match reader.read_event_into(&mut buf) {
      Ok(Event::Start(ref e)) => {
        if e.local_name().as_ref() == b"Task" && !in_task {
          in_task = true;
          task_xml = String::from("<?xml version=\"1.0\"?>\n<Task>");
          depth = 1;
        } else if in_task {
          depth += 1;
          task_xml.push_str(&format!(
            "<{}>",
            String::from_utf8_lossy(e.local_name().as_ref())
          ));
        }
      }
      Ok(Event::Empty(ref e)) => {
        if in_task {
          task_xml.push_str(&format!(
            "<{}/>",
            String::from_utf8_lossy(e.local_name().as_ref())
          ));
        }
      }
      Ok(Event::Text(ref e)) => {
        if in_task {
          let text = e.unescape().unwrap_or_default();
          let escaped = quick_xml::escape::escape(text.as_ref());
          task_xml.push_str(&escaped);
        }
      }
      Ok(Event::End(ref e)) => {
        if in_task {
          depth -= 1;
          if depth == 0 {
            task_xml.push_str("</Task>");
            in_task = false;

            if let Some(task_info) = parse_task_xml(&task_xml) {
              if should_include_task(&task_info) {
                if let Some(item) = create_autostart_item(task_info) {
                  items.push(item);
                }
              }
            }
          } else {
            task_xml.push_str(&format!(
              "</{}>",
              String::from_utf8_lossy(e.local_name().as_ref())
            ));
          }
        }
      }
      Ok(Event::Eof) => break,
      Err(e) => {
        eprintln!("Task XML parsing error: {:?}", e);
        in_task = false;
        depth = 0;
      }
      _ => {}
    }
    buf.clear();
  }

  items
}

fn should_include_task(task: &TaskInfo) -> bool {
  if task.name.is_empty() {
    return false;
  }

  if task.name.starts_with("\\Microsoft") {
    return false;
  }

  task
    .triggers
    .iter()
    .any(|t| STARTUP_TRIGGERS.contains(&t.as_str()))
}

fn create_autostart_item(task: TaskInfo) -> Option<AutostartItem> {
  let is_enabled = task.state != "Disabled";
  let is_delayed = task.is_delayed;

  let target_path = extract_exe_from_command(&task.command);
  let icon_base64 = target_path.as_ref().and_then(|p| get_icon(p));

  let publisher = target_path
    .as_ref()
    .and_then(|p| get_file_version_info(p).ok())
    .and_then(|v| v.company_name)
    .unwrap_or_default();

  let critical_level = get_critical_level(&task.name, &task.command);

  let display_name = task
    .name
    .split('\\')
    .next_back()
    .unwrap_or(&task.name)
    .to_string();

  let id = format!("task|{}", task.name.replace('\\', "/"));

  Some(AutostartItem {
    id,
    name: display_name,
    publisher,
    command: task.command,
    location: format!("Task: {}", task.name),
    source: AutostartSource::Task,
    is_enabled,
    is_delayed,
    icon_base64,
    critical_level,
    file_path: target_path,
  })
}

fn extract_exe_from_command(command: &str) -> Option<String> {
  let cmd = command.trim();

  if cmd.is_empty()
    || cmd == "COM"
    || cmd.to_lowercase().contains("custom handler")
  {
    return None;
  }

  if cmd.starts_with('"')
    && let Some(end) = cmd[1..].find('"')
  {
    return Some(cmd[1..end + 1].to_string());
  }

  let lower_cmd = cmd.to_lowercase();
  if let Some(exe_end) = lower_cmd.find(".exe") {
    let exe_path = cmd[..exe_end + 4].trim();
    return Some(exe_path.to_string());
  }

  None
}

pub fn toggle_task_item(id: &str, enable: bool) -> Result<(), String> {
  let parts: Vec<&str> = id.splitn(2, '|').collect();
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
  let parts: Vec<&str> = id.splitn(2, '|').collect();
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
