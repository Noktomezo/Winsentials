mod critical;
pub mod file_info;
mod folder;
mod icons;
mod registry;
mod services;
mod tasks;
mod types;

use rayon::prelude::*;

pub use types::{AutostartItem, EnrichRequest, EnrichmentData, FileProperties};

fn collect_items<F1, F2, F3, F4>(
  registry_fn: F1,
  folder_fn: F2,
  tasks_fn: F3,
  services_fn: F4,
) -> Vec<AutostartItem>
where
  F1: FnOnce() -> Vec<AutostartItem> + Send,
  F2: FnOnce() -> Vec<AutostartItem> + Send,
  F3: FnOnce() -> Vec<AutostartItem> + Send,
  F4: FnOnce() -> Vec<AutostartItem> + Send,
{
  let ((registry, folder), (tasks, services)) = rayon::join(
    || rayon::join(registry_fn, folder_fn),
    || rayon::join(tasks_fn, services_fn),
  );

  let mut items = Vec::new();
  items.extend(registry);
  items.extend(folder);
  items.extend(tasks);
  items.extend(services);

  items.par_sort_by_cached_key(|it| it.name.to_lowercase());
  items
}

pub fn get_all_autostart_items() -> Vec<AutostartItem> {
  collect_items(
    registry::get_registry_autostart_items,
    folder::get_folder_autostart_items,
    tasks::get_task_autostart_items,
    services::get_service_autostart_items,
  )
}

pub fn get_autostart_items_fast() -> Vec<AutostartItem> {
  collect_items(
    registry::get_registry_items_fast,
    folder::get_folder_items_fast,
    tasks::get_task_items_fast,
    services::get_service_items_fast,
  )
}

pub fn enrich_autostart_items(
  requests: Vec<EnrichRequest>,
) -> Vec<EnrichmentData> {
  if requests.is_empty() {
    return Vec::new();
  }

  use crate::autostart::file_info::get_file_version_info;
  use crate::autostart::icons::get_icon;

  requests
    .par_iter()
    .map(|req| {
      let icon_base64 = req.file_path.as_ref().and_then(|p| get_icon(p));
      let publisher = req
        .file_path
        .as_ref()
        .and_then(|p| get_file_version_info(p).ok())
        .and_then(|v| v.company_name)
        .unwrap_or_default();

      EnrichmentData {
        id: req.id.clone(),
        icon_base64,
        publisher,
      }
    })
    .collect()
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
