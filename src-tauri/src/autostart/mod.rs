mod critical;
pub mod file_info;
mod folder;
mod icons;
mod registry;
mod services;
mod tasks;
mod types;

pub use types::{AutostartItem, AutostartSource, FileProperties};

pub fn get_all_autostart_items() -> Vec<AutostartItem> {
  let mut items = Vec::new();

  items.extend(registry::get_registry_autostart_items());
  items.extend(folder::get_folder_autostart_items());
  items.extend(tasks::get_task_autostart_items());
  items.extend(services::get_service_autostart_items());

  items.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

  items
}

pub fn toggle_autostart_item(id: &str, enable: bool) -> Result<(), String> {
  let parts: Vec<&str> = id.split('|').collect();

  if parts.is_empty() {
    return Err("Invalid item ID".to_string());
  }

  match parts[0] {
    "registry" => registry::toggle_registry_item(id, enable),
    "folder" => folder::toggle_folder_item(id, enable),
    "task" => tasks::toggle_task_item(id, enable),
    "service" => services::toggle_service_item(id, enable),
    _ => Err("Unknown item type".to_string()),
  }
}

pub fn delete_autostart_item(id: &str) -> Result<(), String> {
  let parts: Vec<&str> = id.split('|').collect();

  if parts.is_empty() {
    return Err("Invalid item ID".to_string());
  }

  match parts[0] {
    "registry" => registry::delete_registry_item(id),
    "folder" => folder::delete_folder_item(id),
    "task" => tasks::delete_task_item(id),
    "service" => services::delete_service_item(id),
    _ => Err("Unknown item type".to_string()),
  }
}

pub fn get_file_properties(path: &str) -> Result<FileProperties, String> {
  file_info::get_file_properties(path)
}

pub fn open_file_location(path: &str) -> Result<(), String> {
  file_info::open_file_location(path)
}

pub fn export_autostart_csv(items: &[AutostartItem]) -> String {
  let mut csv = String::new();

  csv.push_str("Name,Publisher,Location,Command,Status,Delayed,Source\n");

  for item in items {
    let status = if item.is_enabled {
      "Enabled"
    } else {
      "Disabled"
    };
    let delayed = if item.is_delayed { "Yes" } else { "No" };
    let source = match item.source {
      AutostartSource::Registry => "Registry",
      AutostartSource::Folder => "Folder",
      AutostartSource::Task => "Task Scheduler",
      AutostartSource::Service => "Service",
    };

    let name = escape_csv(&item.name);
    let publisher = escape_csv(&item.publisher);
    let location = escape_csv(&item.location);
    let command = escape_csv(&item.command);

    csv.push_str(&format!(
      "{},{},{},{},{},{},{}\n",
      name, publisher, location, command, status, delayed, source
    ));
  }

  csv
}

fn escape_csv(s: &str) -> String {
  if s.contains(',') || s.contains('"') || s.contains('\n') {
    format!("\"{}\"", s.replace('"', "\"\""))
  } else {
    s.to_string()
  }
}
