pub mod folder;
pub mod registry;
pub mod services;
pub mod tasks;
pub mod types;

pub use types::{StartupEntry, StartupScope, StartupSource, StartupStatus};

pub fn fetch_all_startup_entries() -> Vec<StartupEntry> {
    let mut all = Vec::new();

    // 1. Registry startup
    all.extend(registry::scan_registry_startup());

    // 2. Startup folder
    all.extend(folder::scan_folder_startup());

    // 3. Custom services
    all.extend(services::scan_services_startup());

    // 4. Scheduled tasks
    all.extend(tasks::scan_tasks_startup());

    // Sort by name
    all.sort_by(|a, b| {
        a.display_name
            .to_lowercase()
            .cmp(&b.display_name.to_lowercase())
    });
    all
}

pub fn toggle_startup_entry(entry: &StartupEntry) -> bool {
    match entry.source {
        StartupSource::Registry => registry::toggle_registry_entry(entry),
        StartupSource::StartupFolder => folder::toggle_folder_entry(entry),
        StartupSource::Service => services::toggle_service_entry(entry),
        StartupSource::ScheduledTask => tasks::toggle_task_entry(entry),
    }
}

pub fn delete_startup_entry(entry: &StartupEntry) -> bool {
    match entry.source {
        StartupSource::Registry => registry::delete_registry_entry(entry),
        StartupSource::StartupFolder => folder::delete_folder_entry(entry),
        StartupSource::Service => services::delete_service_entry(entry),
        StartupSource::ScheduledTask => tasks::delete_task_entry(entry),
    }
}

pub fn open_startup_file_location(entry: &StartupEntry) {
    if let Some(ref path) = entry.target_path {
        let p = std::path::Path::new(path);
        let folder = if p.is_file() { p.parent() } else { Some(p) };
        if let Some(folder_path) = folder {
            #[cfg(target_os = "windows")]
            {
                let _ = std::process::Command::new("explorer.exe")
                    .arg(folder_path)
                    .spawn();
            }
            #[cfg(not(target_os = "windows"))]
            {
                let _ = folder_path;
            }
        }
    }
}

pub fn open_startup_source_manager(entry: &StartupEntry) {
    #[cfg(target_os = "windows")]
    {
        match entry.source {
            StartupSource::Registry => {
                let _ = std::process::Command::new("regedit.exe").spawn();
            }
            StartupSource::StartupFolder => {
                if let Some(ref path) = entry.target_path {
                    let _ = std::process::Command::new("explorer.exe").arg(path).spawn();
                }
            }
            StartupSource::Service => {
                let _ = std::process::Command::new("cmd")
                    .args(["/c", "start", "", "services.msc"])
                    .spawn();
            }
            StartupSource::ScheduledTask => {
                let _ = std::process::Command::new("cmd")
                    .args(["/c", "start", "", "taskschd.msc"])
                    .spawn();
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = entry;
    }
}
