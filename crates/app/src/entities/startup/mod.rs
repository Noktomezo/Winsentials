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

#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
fn shell_open(file: &str, params: Option<&str>) {
    use windows_sys::Win32::UI::Shell::ShellExecuteW;
    use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

    let wide_file: Vec<u16> = file.encode_utf16().chain(Some(0)).collect();
    let wide_params: Option<Vec<u16>> = params.map(|p| p.encode_utf16().chain(Some(0)).collect());
    let params_ptr = wide_params
        .as_ref()
        .map_or(std::ptr::null(), std::vec::Vec::as_ptr);
    let open_verb: [u16; 5] = [
        u16::from(b'o'),
        u16::from(b'p'),
        u16::from(b'e'),
        u16::from(b'n'),
        0,
    ];

    unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            open_verb.as_ptr(),
            wide_file.as_ptr(),
            params_ptr,
            std::ptr::null(),
            SW_SHOWNORMAL,
        );
    }
}

pub fn open_startup_file_location(entry: &StartupEntry) {
    if let Some(ref path) = entry.target_path {
        let p = std::path::Path::new(path);
        let folder = if p.is_file() { p.parent() } else { Some(p) };
        if let Some(folder_path) = folder {
            #[cfg(target_os = "windows")]
            {
                if let Some(folder_str) = folder_path.to_str() {
                    shell_open(folder_str, None);
                }
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
                shell_open("regedit.exe", None);
            }
            StartupSource::StartupFolder => {
                if let Some(ref path) = entry.target_path {
                    shell_open(path, None);
                }
            }
            StartupSource::Service => {
                shell_open("services.msc", None);
            }
            StartupSource::ScheduledTask => {
                shell_open("taskschd.msc", None);
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = entry;
    }
}
