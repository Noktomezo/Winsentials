pub mod folder;
pub mod icon;
pub mod registry;
pub mod search;
pub mod services;
pub mod tasks;
pub mod types;
pub mod vendor;

pub use types::{StartupEntry, StartupScope, StartupSource, StartupStatus};

const fn source_priority(source: StartupSource) -> u8 {
    match source {
        StartupSource::StartupFolder => 0,
        StartupSource::ScheduledTask => 1,
        StartupSource::Registry => 2,
        StartupSource::Service => 3,
    }
}

pub fn fetch_all_startup_entries() -> Vec<StartupEntry> {
    let mut all = Vec::new();

    // 1. Startup folder (Папка)
    all.extend(folder::scan_folder_startup());

    // 2. Scheduled tasks (Планировщик)
    all.extend(tasks::scan_tasks_startup());

    // 3. Registry startup (Реестр)
    all.extend(registry::scan_registry_startup());

    // 4. Custom services (Службы)
    all.extend(services::scan_services_startup());

    // 5. Resolve application icons
    for entry in &mut all {
        entry.icon_path =
            icon::resolve_entry_icon(entry.target_path.as_deref(), entry.command.as_deref());
    }

    // Sort: StartupFolder -> ScheduledTask -> Registry -> Service, then alphabetically by display_name
    all.sort_by(|a, b| {
        source_priority(a.source)
            .cmp(&source_priority(b.source))
            .then_with(|| {
                a.display_name
                    .to_lowercase()
                    .cmp(&b.display_name.to_lowercase())
            })
    });
    all
}

pub fn toggle_startup_entry(entry: &StartupEntry) -> bool {
    match entry.source {
        StartupSource::StartupFolder => folder::toggle_folder_entry(entry),
        StartupSource::ScheduledTask => tasks::toggle_task_entry(entry),
        StartupSource::Registry => registry::toggle_registry_entry(entry),
        StartupSource::Service => services::toggle_service_entry(entry),
    }
}

pub fn delete_startup_entry(entry: &StartupEntry) -> bool {
    match entry.source {
        StartupSource::StartupFolder => folder::delete_folder_entry(entry),
        StartupSource::ScheduledTask => tasks::delete_task_entry(entry),
        StartupSource::Registry => registry::delete_registry_entry(entry),
        StartupSource::Service => services::delete_service_entry(entry),
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
