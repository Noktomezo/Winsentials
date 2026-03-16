pub mod backup;
pub mod error;
pub mod registry;
pub mod shell;
pub mod system_info;
pub mod tweaks;

use error::AppError;
use rayon::prelude::*;
use tauri::{AppHandle, Manager, Runtime, State};
use tweaks::{get_windows_build_number, tweak_by_id, tweaks_for_category};
#[cfg(target_os = "windows")]
use window_vibrancy::{apply_acrylic, clear_acrylic};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn set_chrome_acrylic<R: Runtime>(
    app: AppHandle<R>,
    enabled: bool,
    theme: String,
) -> Result<bool, AppError> {
    #[cfg(target_os = "windows")]
    {
        let window = app
            .get_webview_window("main")
            .ok_or_else(|| AppError::message("main window not found"))?;

        if enabled {
            let tint = match theme.as_str() {
                "light" => Some((232, 238, 242, 128)),
                _ => Some((28, 34, 38, 128)),
            };

            apply_acrylic(&window, tint).map_err(|error| AppError::message(error.to_string()))?;
            return Ok(true);
        }

        clear_acrylic(&window).map_err(|error| AppError::message(error.to_string()))?;
        Ok(false)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (app, enabled, theme);
        Ok(false)
    }
}

#[tauri::command]
fn tweaks_by_category(category: String) -> Result<Vec<tweaks::TweakMeta>, AppError> {
    tweaks_for_category(&category)
        .into_par_iter()
        .map(|tweak| {
            let mut meta = tweak.meta().clone();
            meta.current_value = tweak.get_status()?.current_value;
            Ok(meta)
        })
        .collect()
}

#[tauri::command]
fn tweak_apply(id: String, value: String) -> Result<tweaks::TweakResult, AppError> {
    let tweak = tweak_by_id(&id)?;
    tweak.apply(&value)?;
    tweak.get_status().map(|status| tweaks::TweakResult {
        success: true,
        current_value: status.current_value,
    })
}

#[tauri::command]
fn tweak_reset(id: String) -> Result<tweaks::TweakResult, AppError> {
    let tweak = tweak_by_id(&id)?;
    tweak.reset()?;
    tweak.get_status().map(|status| tweaks::TweakResult {
        success: true,
        current_value: status.current_value,
    })
}

#[tauri::command]
fn tweak_status(id: String) -> Result<tweaks::TweakStatus, AppError> {
    tweak_by_id(&id)?.get_status()
}

#[tauri::command]
fn tweak_extra(id: String) -> Result<(), AppError> {
    tweak_by_id(&id)?.extra()
}

#[tauri::command]
fn get_windows_build() -> Result<tweaks::WindowsVersion, AppError> {
    get_windows_build_number()
}

#[tauri::command]
fn get_static_system_info(
    state: State<system_info::SystemInfoState>,
) -> Result<system_info::StaticSystemInfo, AppError> {
    let mut cache = state
        .static_cache
        .lock()
        .map_err(|_| AppError::message("static_cache lock poisoned"))?;
    if let Some(info) = cache.as_ref() {
        return Ok(info.clone());
    }
    let system = state
        .system
        .lock()
        .map_err(|_| AppError::message("system lock poisoned"))?;
    let info = system_info::gather_static_info(&system)?;
    *cache = Some(info.clone());
    Ok(info)
}

#[tauri::command]
fn get_live_system_info(
    state: State<system_info::SystemInfoState>,
) -> Result<system_info::LiveSystemInfo, AppError> {
    let mut system = state
        .system
        .lock()
        .map_err(|_| AppError::message("system lock poisoned"))?;
    let mut networks = state
        .networks
        .lock()
        .map_err(|_| AppError::message("networks lock poisoned"))?;
    let mut prev_net = state
        .prev_net
        .lock()
        .map_err(|_| AppError::message("prev_net lock poisoned"))?;

    Ok(system_info::gather_live_info(
        &mut system,
        &mut networks,
        &mut prev_net,
        #[cfg(target_os = "windows")]
        &state.nvml,
    ))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = tauri::Builder::default()
        .plugin(
            tauri_plugin_prevent_default::Builder::new()
                .with_flags(if cfg!(debug_assertions) {
                    tauri_plugin_prevent_default::Flags::empty()
                } else {
                    tauri_plugin_prevent_default::Flags::all()
                })
                .build(),
        )
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build());

    #[cfg(target_os = "windows")]
    let builder = builder
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build());

    builder
        .manage(system_info::SystemInfoState::new())
        .invoke_handler(tauri::generate_handler![
            greet,
            set_chrome_acrylic,
            tweaks_by_category,
            tweak_apply,
            tweak_reset,
            tweak_status,
            tweak_extra,
            get_windows_build,
            get_static_system_info,
            get_live_system_info,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
