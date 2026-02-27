use crate::autostart::critical::get_critical_level;
use crate::autostart::file_info::get_file_version_info;
use crate::autostart::icons::get_icon;
use crate::autostart::types::{AutostartItem, AutostartSource};
use crate::utils::command::hidden_command;
use rayon::prelude::*;
use windows::Win32::System::Com::{
  CLSCTX_INPROC_SERVER, COINIT, CoCreateInstance, CoInitializeEx,
  CoUninitialize,
};
use windows::Win32::System::TaskScheduler::{
  IBootTrigger, IExecAction, ILogonTrigger, IRegisteredTask, ITaskFolder,
  ITaskService, TASK_ACTION_EXEC, TASK_ENUM_HIDDEN, TASK_STATE_DISABLED,
  TASK_TRIGGER_BOOT, TASK_TRIGGER_LOGON, TaskScheduler,
};
use windows::Win32::System::Variant::VARIANT;
use windows::core::{BSTR, Interface};

struct TaskData {
  path: String,
  command: String,
  is_enabled: bool,
  is_delayed: bool,
}

pub fn get_task_autostart_items() -> Vec<AutostartItem> {
  let raw_items = collect_tasks_via_com();
  raw_items.into_par_iter().map(enrich_task_item).collect()
}

pub fn get_task_items_fast() -> Vec<AutostartItem> {
  let raw_items = collect_tasks_via_com();
  raw_items
    .into_iter()
    .map(|raw| {
      let target_path = extract_exe_from_command(&raw.command);
      let critical_level = get_critical_level(&raw.path, &raw.command);

      let display_name = raw
        .path
        .split('\\')
        .next_back()
        .unwrap_or(&raw.path)
        .to_string();

      let id = format!("task|{}", raw.path.replace('\\', "/"));

      AutostartItem {
        id,
        name: display_name,
        publisher: String::new(),
        command: raw.command,
        location: format!("Task: {}", raw.path),
        source: AutostartSource::Task,
        is_enabled: raw.is_enabled,
        is_delayed: raw.is_delayed,
        icon_base64: None,
        critical_level,
        file_path: target_path,
        start_type: None,
      }
    })
    .collect()
}

fn collect_tasks_via_com() -> Vec<TaskData> {
  unsafe {
    let hr = CoInitializeEx(None, COINIT(0));

    if hr.is_err() {
      return Vec::new();
    }

    let Ok(service): Result<ITaskService, _> =
      CoCreateInstance(&TaskScheduler, None, CLSCTX_INPROC_SERVER)
    else {
      CoUninitialize();
      return Vec::new();
    };

    let Ok(()) = service.Connect(
      &VARIANT::default(),
      &VARIANT::default(),
      &VARIANT::default(),
      &VARIANT::default(),
    ) else {
      CoUninitialize();
      return Vec::new();
    };

    let Ok(root) = service.GetFolder(&BSTR::from("\\")) else {
      CoUninitialize();
      return Vec::new();
    };

    let mut items = Vec::new();
    collect_tasks_recursive(&root, &mut items);

    CoUninitialize();
    items
  }
}

unsafe fn collect_tasks_recursive(
  folder: &ITaskFolder,
  items: &mut Vec<TaskData>,
) {
  unsafe {
    let Ok(tasks) = folder.GetTasks(TASK_ENUM_HIDDEN.0) else {
      return;
    };

    let Ok(count) = tasks.Count() else {
      return;
    };

    for i in 1..=count {
      let Ok(task) = tasks.get_Item(&VARIANT::from(i)) else {
        continue;
      };
      if let Some(data) = process_task(&task) {
        items.push(data);
      }
    }

    let Ok(subfolders) = folder.GetFolders(0) else {
      return;
    };

    let Ok(folder_count) = subfolders.Count() else {
      return;
    };

    for i in 1..=folder_count {
      let Ok(subfolder) = subfolders.get_Item(&VARIANT::from(i)) else {
        continue;
      };
      collect_tasks_recursive(&subfolder, items);
    }
  }
}

unsafe fn process_task(task: &IRegisteredTask) -> Option<TaskData> {
  unsafe {
    let path = task.Path().ok()?.to_string();

    if path.starts_with("\\Microsoft") {
      return None;
    }

    if !has_startup_trigger(task) {
      return None;
    }

    let state = task.State().ok().unwrap_or(TASK_STATE_DISABLED);
    let is_enabled = state != TASK_STATE_DISABLED;
    let is_delayed = has_delay(task);

    Some(TaskData {
      path,
      command: get_task_command(task),
      is_enabled,
      is_delayed,
    })
  }
}

unsafe fn has_startup_trigger(task: &IRegisteredTask) -> bool {
  unsafe {
    let Ok(definition) = task.Definition() else {
      return false;
    };
    let Ok(triggers) = definition.Triggers() else {
      return false;
    };

    let mut count: i32 = 0;
    if triggers.Count(&mut count).is_err() {
      return false;
    }

    for i in 1..=count {
      let Ok(trigger) = triggers.get_Item(i) else {
        continue;
      };

      let mut trigger_type = Default::default();
      if trigger.Type(&mut trigger_type).is_ok()
        && (trigger_type == TASK_TRIGGER_LOGON
          || trigger_type == TASK_TRIGGER_BOOT)
      {
        return true;
      }
    }
    false
  }
}

unsafe fn has_delay(task: &IRegisteredTask) -> bool {
  unsafe {
    let Ok(definition) = task.Definition() else {
      return false;
    };
    let Ok(triggers) = definition.Triggers() else {
      return false;
    };

    let mut count: i32 = 0;
    if triggers.Count(&mut count).is_err() {
      return false;
    }

    for i in 1..=count {
      let Ok(trigger) = triggers.get_Item(i) else {
        continue;
      };

      let mut trigger_type = Default::default();
      if trigger.Type(&mut trigger_type).is_ok()
        && (trigger_type == TASK_TRIGGER_LOGON
          || trigger_type == TASK_TRIGGER_BOOT)
      {
        if trigger_type == TASK_TRIGGER_LOGON {
          if let Ok(logon_trigger) = trigger.cast::<ILogonTrigger>() {
            let mut delay = BSTR::new();
            if logon_trigger.Delay(&mut delay).is_ok() && !delay.is_empty() {
              return true;
            }
          }
        } else if trigger_type == TASK_TRIGGER_BOOT
          && let Ok(boot_trigger) = trigger.cast::<IBootTrigger>()
        {
          let mut delay = BSTR::new();
          if boot_trigger.Delay(&mut delay).is_ok() && !delay.is_empty() {
            return true;
          }
        }
      }
    }
    false
  }
}

unsafe fn get_task_command(task: &IRegisteredTask) -> String {
  unsafe {
    let Ok(definition) = task.Definition() else {
      return String::new();
    };
    let Ok(actions) = definition.Actions() else {
      return String::new();
    };

    let mut count: i32 = 0;
    if actions.Count(&mut count).is_err() {
      return String::new();
    }

    for i in 1..=count {
      let Ok(action) = actions.get_Item(i) else {
        continue;
      };

      let mut action_type = Default::default();
      if action.Type(&mut action_type).is_ok()
        && action_type == TASK_ACTION_EXEC
        && let Ok(exec) = action.cast::<IExecAction>()
      {
        let mut path = BSTR::new();
        if exec.Path(&mut path).is_ok() {
          let path_str = path.to_string();
          if !path_str.is_empty() {
            let quoted_path = format!("\"{path_str}\"");
            let mut args = BSTR::new();
            if exec.Arguments(&mut args).is_ok() {
              let args_str = args.to_string();
              return if args_str.is_empty() {
                quoted_path
              } else {
                format!("{quoted_path} {args_str}")
              };
            }
            return quoted_path;
          }
        }
      }
    }
    String::new()
  }
}

fn enrich_task_item(raw: TaskData) -> AutostartItem {
  let target_path = extract_exe_from_command(&raw.command);
  let icon_base64 = target_path.as_ref().and_then(|p| get_icon(p));

  let publisher = target_path
    .as_ref()
    .and_then(|p| get_file_version_info(p).ok())
    .and_then(|v| v.company_name)
    .unwrap_or_default();

  let critical_level = get_critical_level(&raw.path, &raw.command);

  let display_name = raw
    .path
    .split('\\')
    .next_back()
    .unwrap_or(&raw.path)
    .to_string();

  let id = format!("task|{}", raw.path.replace('\\', "/"));

  AutostartItem {
    id,
    name: display_name,
    publisher,
    command: raw.command,
    location: format!("Task: {}", raw.path),
    source: AutostartSource::Task,
    is_enabled: raw.is_enabled,
    is_delayed: raw.is_delayed,
    icon_base64,
    critical_level,
    file_path: target_path,
    start_type: None,
  }
}

fn extract_exe_from_command(command: &str) -> Option<String> {
  let cmd = command.trim();

  if cmd.is_empty()
    || cmd == "COM"
    || cmd.to_lowercase().contains("custom handler")
  {
    return None;
  }

  let first_token = if let Some(stripped) = cmd.strip_prefix('"') {
    if let Some(end) = stripped.find('"') {
      &stripped[..end]
    } else {
      cmd
    }
  } else {
    cmd.split_whitespace().next().unwrap_or(cmd)
  };

  let lower_token = first_token.to_ascii_lowercase();
  if let Some(exe_end) = lower_token.rfind(".exe") {
    let boundary_idx = exe_end + 4;
    if boundary_idx >= first_token.len()
      || first_token.as_bytes()[boundary_idx].is_ascii_whitespace()
      || first_token.as_bytes()[boundary_idx] == b'"'
      || first_token.as_bytes()[boundary_idx] == b'\''
    {
      let exe_path = first_token[..boundary_idx].trim();
      return Some(exe_path.to_string());
    }
  }

  None
}

pub fn toggle_task_item(id: &str, enable: bool) -> Result<(), String> {
  let task_path = extract_task_path_from_id(id)?;

  let status = if enable { "enable" } else { "disable" };

  let output = hidden_command("schtasks")
    .args(["/change", "/tn", &task_path, &format!("/{status}")])
    .output()
    .map_err(|e| format!("Failed to execute schtasks: {e}"))?;

  if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    return Err(format!("Failed to {status} task: {stderr}"));
  }

  Ok(())
}

pub fn delete_task_item(id: &str) -> Result<(), String> {
  let task_path = extract_task_path_from_id(id)?;

  unsafe {
    let hr = CoInitializeEx(None, COINIT(0));

    if hr.is_err() {
      return Err("Failed to initialize COM".to_string());
    }

    let result = (|| {
      let service: ITaskService =
        CoCreateInstance(&TaskScheduler, None, CLSCTX_INPROC_SERVER)
          .map_err(|e| format!("Failed to create TaskService: {e}"))?;

      service
        .Connect(
          &VARIANT::default(),
          &VARIANT::default(),
          &VARIANT::default(),
          &VARIANT::default(),
        )
        .map_err(|e| format!("Failed to connect: {e}"))?;

      let folder_path = get_parent_folder_path(&task_path);
      let task_name = get_task_name(&task_path);

      let folder = service
        .GetFolder(&BSTR::from(folder_path))
        .map_err(|e| format!("Failed to get folder: {e}"))?;

      folder
        .DeleteTask(&BSTR::from(task_name), 0)
        .map_err(|e| format!("Failed to delete task: {e}"))?;

      Ok(())
    })();

    CoUninitialize();
    result
  }
}

fn extract_task_path_from_id(id: &str) -> Result<String, String> {
  let parts: Vec<&str> = id.splitn(2, '|').collect();
  if parts.len() != 2 || parts[0] != "task" {
    return Err("Invalid task item ID".to_string());
  }

  let path = parts[1].trim();
  if path.is_empty() {
    return Err("Invalid task item ID".to_string());
  }

  let is_rooted = path.starts_with('\\')
    || path.starts_with('/')
    || (path.len() >= 2 && path.chars().nth(1) == Some(':'));

  if !is_rooted {
    return Err("Invalid task item ID".to_string());
  }

  Ok(path.replace('/', "\\"))
}

fn get_parent_folder_path(task_path: &str) -> String {
  task_path
    .rsplit_once('\\')
    .map(|(folder, _)| {
      if folder.is_empty() {
        "\\".to_string()
      } else {
        folder.to_string()
      }
    })
    .unwrap_or_else(|| "\\".to_string())
}

fn get_task_name(task_path: &str) -> String {
  task_path
    .rsplit_once('\\')
    .map(|(_, name)| name)
    .unwrap_or(task_path)
    .to_string()
}
