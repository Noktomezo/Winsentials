pub mod backup;
pub mod commands;
pub mod error;
pub mod registry;
pub mod shell;
pub mod startup;
pub mod system_info;
pub mod tweaks;

use crate::backup::{backup_create, backup_delete, backup_list, backup_rename, backup_restore};
use crate::commands::app::greet;
use crate::commands::startup::{
    startup_delete, startup_details, startup_disable, startup_enable, startup_hydrate_entries,
    startup_list_registry, startup_list_scheduled_tasks, startup_list_startup_folder,
};
use crate::commands::system::restart_pc;
use crate::commands::system_info::{
    get_device_inventory_info, get_live_cpu_info, get_live_disk_info, get_live_gpu_info,
    get_live_home_info, get_live_network_info, get_live_ram_info, get_live_system_info,
    get_static_system_info,
};
use crate::commands::tweaks::{
    get_windows_build, tweak_apply, tweak_extra, tweak_reset, tweak_status, tweaks_by_category,
};
use crate::commands::window::set_chrome_acrylic;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(debug_assertions)]
    let devtools = tauri_plugin_devtools::init();

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

    #[cfg(debug_assertions)]
    let builder = builder.plugin(devtools);

    #[cfg(target_os = "windows")]
    let builder = builder
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build());

    builder
        .manage(system_info::SystemInfoState::new())
        .setup(|app| {
            crate::backup::ensure_initial_backup();
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
                let mut bg_prev_net: Option<system_info::PreviousNetSnapshot> = None;

                #[cfg(target_os = "windows")]
                let mut pdh = system_info::pdh_open_gpu_query();
                #[cfg(target_os = "windows")]
                let mut disk_pdh = system_info::pdh_open_disk_query();
                #[cfg(target_os = "windows")]
                let mut cpu_pdh = system_info::pdh_open_cpu_perf_query();

                loop {
                    let start = Instant::now();

                    #[cfg(target_os = "windows")]
                    if pdh.is_none() {
                        pdh = system_info::pdh_open_gpu_query();
                    }
                    #[cfg(target_os = "windows")]
                    if disk_pdh.is_none() {
                        disk_pdh = system_info::pdh_open_disk_query();
                    }
                    #[cfg(target_os = "windows")]
                    if cpu_pdh.is_none() {
                        cpu_pdh = system_info::pdh_open_cpu_perf_query();
                    }
                    let state: tauri::State<'_, system_info::SystemInfoState> =
                        handle.state::<system_info::SystemInfoState>();

                    let (gpus, base_freq_mhz) = match state.static_cache.lock() {
                        Ok(cache) => {
                            let gpus = cache.as_ref().map(|s| s.gpus.clone()).unwrap_or_default();
                            let base_freq =
                                cache.as_ref().map(|s| s.cpu.base_freq_mhz).unwrap_or(0);
                            (gpus, base_freq)
                        }
                        Err(_) => (vec![], 0),
                    };

                    let (live, gpu_pdh_failed) = system_info::gather_live_info(
                        &mut bg_system,
                        &mut bg_networks,
                        &mut bg_prev_net,
                        &gpus,
                        #[cfg(target_os = "windows")]
                        system_info::PdhHandles {
                            gpu_query: pdh.map(|p| p.0).unwrap_or(0),
                            gpu_counter: pdh.map(|p| p.1).unwrap_or(0),
                            disk_query: disk_pdh.map(|p| p.0).unwrap_or(0),
                            disk_active_counter: disk_pdh.map(|p| p.1).unwrap_or(0),
                            disk_response_counter: disk_pdh.map(|p| p.2).unwrap_or(0),
                            disk_read_counter: disk_pdh.map(|p| p.3).unwrap_or(0),
                            disk_write_counter: disk_pdh.map(|p| p.4).unwrap_or(0),
                            cpu_query: cpu_pdh.map(|p| p.0).unwrap_or(0),
                            cpu_counter: cpu_pdh.map(|p| p.1).unwrap_or(0),
                            base_freq_mhz,
                        },
                    );

                    if let Ok(mut cache) = state.live_cache.lock() {
                        *cache = Some(live);
                    }

                    #[cfg(target_os = "windows")]
                    if gpu_pdh_failed {
                        if let Some(handles) = pdh.take() {
                            system_info::pdh_close_gpu_query(handles);
                        }
                        pdh = system_info::pdh_open_gpu_query();
                    }

                    let elapsed = start.elapsed();
                    if let Some(remaining) = Duration::from_millis(1000).checked_sub(elapsed) {
                        std::thread::sleep(remaining);
                    }
                }
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            backup_create,
            backup_list,
            backup_restore,
            backup_rename,
            backup_delete,
            greet,
            set_chrome_acrylic,
            tweaks_by_category,
            tweak_apply,
            tweak_reset,
            tweak_status,
            tweak_extra,
            get_windows_build,
            get_static_system_info,
            get_device_inventory_info,
            startup_list_registry,
            startup_list_startup_folder,
            startup_list_scheduled_tasks,
            startup_hydrate_entries,
            startup_enable,
            startup_disable,
            startup_delete,
            startup_details,
            get_live_system_info,
            get_live_home_info,
            get_live_cpu_info,
            get_live_ram_info,
            get_live_disk_info,
            get_live_network_info,
            get_live_gpu_info,
            restart_pc,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
