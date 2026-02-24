mod autostart;
mod system_info;
mod tweaks;
mod wmi_queries;

use autostart::{
  delete_autostart_item, enrich_autostart_items, get_all_autostart_items,
  get_autostart_items_fast as get_fast_items, get_file_properties,
  open_file_location, toggle_autostart_item, AutostartItem, EnrichmentData,
  FileProperties,
};
use serde::{Deserialize, Serialize};
use sysinfo::System;
use system_info::{
  get_dynamic_system_info, get_static_system_info, get_system_info,
};
use tweaks::{get_all_tweaks, get_tweak_by_id, TweakMeta, TweakState};

const WIN11_BUILD: u32 = 22000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweakInfo {
  meta: TweakMeta,
  state: TweakState,
  is_available: bool,
  windows_version_required: Option<String>,
}

fn get_windows_build_number() -> u32 {
  System::kernel_version()
    .and_then(|v| v.parse().ok())
    .unwrap_or(0)
}

#[tauri::command]
fn get_windows_build() -> u32 {
  get_windows_build_number()
}

fn check_tweak_available(meta: &TweakMeta, current_build: u32) -> bool {
  match meta.min_windows_build {
    Some(min_build) => current_build >= min_build,
    None => true,
  }
}

fn get_windows_version_name(min_build: Option<u32>) -> Option<String> {
  min_build.and_then(|build| {
    if build >= WIN11_BUILD {
      Some("Windows 11".to_string())
    } else {
      None
    }
  })
}

#[tauri::command]
fn get_tweaks_by_category(category: String) -> Vec<TweakInfo> {
  let current_build = get_windows_build_number();

  get_all_tweaks()
    .into_iter()
    .filter(|t| {
      let cat = t.meta().category.clone();
      let cat_str = serde_json::to_string(&cat).unwrap_or_default();
      cat_str.to_lowercase().contains(&category.to_lowercase())
    })
    .map(|t| {
      let meta = t.meta().clone();
      let state = t.check().unwrap_or_else(|_| TweakState {
        id: meta.id.clone(),
        current_value: None,
        is_applied: false,
      });
      let is_available = check_tweak_available(&meta, current_build);
      let windows_version_required =
        get_windows_version_name(meta.min_windows_build);
      TweakInfo {
        meta,
        state,
        is_available,
        windows_version_required,
      }
    })
    .collect()
}

#[tauri::command]
fn get_tweak_info(id: String) -> Option<TweakInfo> {
  let current_build = get_windows_build_number();

  get_tweak_by_id(&id).map(|t| {
    let meta = t.meta().clone();
    let state = t.check().unwrap_or_else(|_| TweakState {
      id: meta.id.clone(),
      current_value: None,
      is_applied: false,
    });
    let is_available = check_tweak_available(&meta, current_build);
    let windows_version_required =
      get_windows_version_name(meta.min_windows_build);
    TweakInfo {
      meta,
      state,
      is_available,
      windows_version_required,
    }
  })
}

#[tauri::command]
fn apply_tweak(
  id: String,
  value: Option<String>,
) -> Result<TweakState, String> {
  let tweak = get_tweak_by_id(&id).ok_or("Tweak not found")?;
  tweak.apply(value.as_deref())?;
  tweak.check().map_err(|e| e.to_string())
}

#[tauri::command]
fn revert_tweak(id: String) -> Result<TweakState, String> {
  let tweak = get_tweak_by_id(&id).ok_or("Tweak not found")?;
  tweak.revert()?;
  tweak.check().map_err(|e| e.to_string())
}

#[tauri::command]
fn check_tweak(id: String) -> Result<TweakState, String> {
  let tweak = get_tweak_by_id(&id).ok_or("Tweak not found")?;
  tweak.check().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_all_tweaks_info() -> Vec<TweakInfo> {
  let current_build = get_windows_build_number();

  get_all_tweaks()
    .into_iter()
    .map(|t| {
      let meta = t.meta().clone();
      let state = t.check().unwrap_or_else(|_| TweakState {
        id: meta.id.clone(),
        current_value: None,
        is_applied: false,
      });
      let is_available = check_tweak_available(&meta, current_build);
      let windows_version_required =
        get_windows_version_name(meta.min_windows_build);
      TweakInfo {
        meta,
        state,
        is_available,
        windows_version_required,
      }
    })
    .collect()
}

#[tauri::command]
fn get_autostart_items() -> Vec<AutostartItem> {
  get_all_autostart_items()
}

#[tauri::command]
fn get_autostart_items_fast() -> Vec<AutostartItem> {
  get_fast_items()
}

#[tauri::command]
fn enrich_autostart(ids: Vec<String>) -> Vec<EnrichmentData> {
  enrich_autostart_items(ids)
}

#[tauri::command]
fn toggle_autostart(id: String, enable: bool) -> Result<(), String> {
  toggle_autostart_item(&id, enable)
}

#[tauri::command]
fn delete_autostart(id: String) -> Result<(), String> {
  delete_autostart_item(&id)
}

#[tauri::command]
fn open_location(path: String) -> Result<(), String> {
  open_file_location(&path)
}

#[tauri::command]
fn get_properties(path: String) -> Result<FileProperties, String> {
  get_file_properties(&path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(tauri_plugin_shell::init())
    .plugin(tauri_plugin_fs::init())
    .plugin(tauri_plugin_opener::init())
    .plugin(tauri_plugin_clipboard_manager::init())
    .invoke_handler(tauri::generate_handler![
      get_tweaks_by_category,
      get_tweak_info,
      apply_tweak,
      revert_tweak,
      check_tweak,
      get_all_tweaks_info,
      get_system_info,
      get_static_system_info,
      get_dynamic_system_info,
      get_windows_build,
      get_autostart_items,
      get_autostart_items_fast,
      enrich_autostart,
      toggle_autostart,
      delete_autostart,
      open_location,
      get_properties
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
