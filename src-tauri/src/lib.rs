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
    state
        .live_cache
        .lock()
        .map_err(|_| AppError::message("live_cache lock poisoned"))?
        .clone()
        .ok_or_else(|| AppError::message("live info not yet available"))
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
        .setup(|app| {
            use std::time::{Duration, Instant};
            use sysinfo::{CpuRefreshKind, MemoryRefreshKind, Networks, RefreshKind, System};

            let handle = app.handle().clone();
            std::thread::spawn(move || {
                let mut bg_system = System::new_with_specifics(
                    RefreshKind::nothing()
                        .with_cpu(CpuRefreshKind::everything())
                        .with_memory(MemoryRefreshKind::everything()),
                );
                let mut bg_networks = Networks::new_with_refreshed_list();
                let mut bg_prev_net: Option<std::collections::HashMap<String, (u64, u64)>> = None;

                #[cfg(target_os = "windows")]
                let mut pdh = system_info::pdh_open_gpu_query();
                #[cfg(target_os = "windows")]
                let cpu_pdh = system_info::pdh_open_cpu_perf_query();

                loop {
                    let start = Instant::now();

                    #[cfg(target_os = "windows")]
                    if pdh.is_none() {
                        pdh = system_info::pdh_open_gpu_query();
                    }
                    let state = handle.state::<system_info::SystemInfoState>();

                    let (gpus, base_freq_mhz) = {
                        let cache = state.static_cache.lock().unwrap();
                        let gpus = cache.as_ref().map(|s| s.gpus.clone()).unwrap_or_default();
                        let base_freq = cache.as_ref().map(|s| s.cpu.base_freq_mhz).unwrap_or(0);
                        (gpus, base_freq)
                    };

                    let live = system_info::gather_live_info(
                        &mut bg_system,
                        &mut bg_networks,
                        &mut bg_prev_net,
                        &gpus,
                        #[cfg(target_os = "windows")]
                        pdh.map(|p| p.0).unwrap_or(0),
                        #[cfg(target_os = "windows")]
                        pdh.map(|p| p.1).unwrap_or(0),
                        #[cfg(target_os = "windows")]
                        cpu_pdh.map(|p| p.0).unwrap_or(0),
                        #[cfg(target_os = "windows")]
                        cpu_pdh.map(|p| p.1).unwrap_or(0),
                        #[cfg(target_os = "windows")]
                        base_freq_mhz,
                    );

                    *state.live_cache.lock().unwrap() = Some(live);

                    let elapsed = start.elapsed();
                    if let Some(remaining) = Duration::from_millis(1000).checked_sub(elapsed) {
                        std::thread::sleep(remaining);
                    }
                }
            });
            Ok(())
        })
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
