use std::path::Path;

use windows_registry::LOCAL_MACHINE;

use super::types::{StartupEntry, StartupScope, StartupSource, StartupStatus};
use super::vendor::{extract_clean_exe_path, get_file_publisher};

const REG_SERVICES: &str = r"SYSTEM\CurrentControlSet\Services";

// Known Windows system hosts and binaries
const SYSTEM_BINARIES: &[&str] = &[
    "svchost.exe",
    "lsass.exe",
    "services.exe",
    "spoolsv.exe",
    "csrss.exe",
    "wininit.exe",
    "smss.exe",
    "dwm.exe",
    "sihost.exe",
    "fontdrvhost.exe",
    "audiodg.exe",
    "dashost.exe",
    "conhost.exe",
    "searchindexer.exe",
    "trustedinstaller.exe",
    "vds.exe",
    "vssvc.exe",
    "wmiprvse.exe",
    "taskhostw.exe",
    "dllhost.exe",
    "sppsvc.exe",
    "msmpeng.exe",
    "nissrv.exe",
    "werfault.exe",
    "wermgr.exe",
    "msiexec.exe",
    "consent.exe",
    "smartscreen.exe",
    "securityhealthservice.exe",
    "alg.exe",
    "appvclient.exe",
    "gameinputsvc.exe",
    "presentationhost.exe",
];

pub fn scan_services_startup() -> Vec<StartupEntry> {
    let mut entries = Vec::new();

    let Ok(services_root) = LOCAL_MACHINE.open(REG_SERVICES) else {
        return entries;
    };

    let Ok(service_names) = services_root.keys() else {
        return entries;
    };

    for service_name in service_names {
        if service_name.trim().is_empty() {
            continue;
        }

        let Ok(service_key) = services_root.open(&service_name) else {
            continue;
        };

        // 1. Skip Per-User services by UserServiceFlags
        if service_key.get_u32("UserServiceFlags").is_ok() {
            continue;
        }

        // 2. Check service type (Only Win32 services: 0x10 Own Process, 0x20 Share Process, 0x100/0x200 interactive)
        let Ok(svc_type) = service_key.get_u32("Type") else {
            continue;
        };
        if svc_type & 0x30 == 0 && svc_type & 0x100 == 0 && svc_type & 0x200 == 0 {
            continue; // Skip kernel / file system drivers
        }

        // 3. Read ImagePath
        let Ok(image_path) = service_key.get_string("ImagePath") else {
            continue;
        };
        let trimmed_path = image_path.trim();
        if trimmed_path.is_empty() {
            continue;
        }

        // 4. Read raw DisplayName
        let raw_display_name = service_key
            .get_string("DisplayName")
            .unwrap_or_else(|_| service_name.clone());

        // 5. Filter out Windows core internal services
        if is_windows_core_service(&service_name, trimmed_path, &raw_display_name) {
            continue;
        }

        // 6. Read Start type (2 = Auto, 3 = Manual, 4 = Disabled)
        let start_type = service_key.get_u32("Start").unwrap_or(3);
        let status = if start_type == 2 {
            StartupStatus::Enabled
        } else {
            StartupStatus::Disabled
        };

        let target_exe = extract_clean_exe_path(trimmed_path);
        let target_str = target_exe.as_ref().map(|p| p.to_string_lossy().to_string());
        let display_name =
            clean_service_display_name(&raw_display_name, &service_name, target_exe.as_deref());
        let publisher = target_exe.as_deref().and_then(get_file_publisher);

        entries.push(StartupEntry {
            id: format!("svc_{service_name}"),
            name: service_name.clone(),
            display_name,
            publisher,
            source: StartupSource::Service,
            scope: StartupScope::AllUsers,
            status,
            command: Some(image_path),
            target_path: target_str,
            icon_path: None,
            location_label: "HKLM\\...\\Services".to_string(),
            raw_id: service_name,
        });
    }

    entries
}

fn clean_service_display_name(
    raw_display_name: &str,
    service_name: &str,
    target_exe: Option<&Path>,
) -> String {
    let trimmed = raw_display_name.trim();

    // 1. If it starts with '@': e.g. "@oem40.inf,%amd3dvcacheSvc.DisplayName%;AMD 3D V-Cache Performance Optimizer Service"
    if let Some(after_at) = trimmed.strip_prefix('@') {
        if let Some(semi_pos) = after_at.find(';') {
            let name_part = after_at[semi_pos + 1..].trim();
            if !name_part.is_empty() {
                return name_part.to_string();
            }
        }
    }

    // 2. If it is non-empty and doesn't contain corrupted replacement characters ''
    if !trimmed.is_empty() && !trimmed.contains('\u{FFFD}') && !trimmed.starts_with('@') {
        return trimmed.to_string();
    }

    // 3. Fallback: try stem from target exe or service_name
    if let Some(path) = target_exe {
        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
            return stem.to_string();
        }
    }

    service_name.to_string()
}

fn is_windows_core_service(name: &str, image_path: &str, raw_display_name: &str) -> bool {
    let lower_path = image_path.to_ascii_lowercase();
    let lower_name = name.to_ascii_lowercase();
    let trimmed_dn = raw_display_name.trim();

    if lower_path.contains("svchost") || lower_path.contains("presentationhost") {
        return true;
    }

    if lower_name.ends_with("usersvc") || is_per_user_service_name(&lower_name) {
        return true;
    }

    if trimmed_dn.starts_with('@') && !trimmed_dn.contains(';') {
        return true;
    }

    for bin in SYSTEM_BINARIES {
        if lower_path.contains(bin) {
            return true;
        }
    }

    // Windows / Microsoft driver or standard services
    if (lower_path.starts_with(r"c:\windows\system32\")
        || lower_path.starts_with(r"%systemroot%\system32\")
        || lower_path.starts_with(r"\systemroot\system32\"))
        && !lower_path.contains("driverstore")
        && !lower_path.contains("driver")
        && !lower_path.contains("thirdparty")
        && !lower_path.contains("nvidia")
        && !lower_path.contains("amd")
        && !lower_path.contains("realtek")
        && !lower_path.contains("razer")
        && !lower_name.starts_with("amd")
        && !lower_name.starts_with("nv")
    {
        return true;
    }

    if lower_name.starts_with("appx")
        || lower_name.starts_with("wuauserv")
        || lower_name.starts_with("windefend")
        || lower_name.starts_with("sense")
    {
        return true;
    }

    false
}

fn is_per_user_service_name(name: &str) -> bool {
    if let Some((_, suffix)) = name.rsplit_once('_') {
        suffix.len() >= 4 && suffix.len() <= 8 && suffix.chars().all(|c| c.is_ascii_hexdigit())
    } else {
        false
    }
}

pub fn toggle_service_entry(entry: &StartupEntry) -> bool {
    let service_name = &entry.raw_id;
    let new_mode = if entry.status == StartupStatus::Enabled {
        "demand"
    } else {
        "auto"
    };

    #[cfg(target_os = "windows")]
    {
        let status = std::process::Command::new("sc.exe")
            .args(["config", service_name, &format!("start={new_mode}")])
            .status();
        matches!(status, Ok(s) if s.success())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (service_name, new_mode);
        true
    }
}

pub fn delete_service_entry(entry: &StartupEntry) -> bool {
    let service_name = &entry.raw_id;

    #[cfg(target_os = "windows")]
    {
        let status = std::process::Command::new("sc.exe")
            .args(["delete", service_name])
            .status();
        matches!(status, Ok(s) if s.success())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = service_name;
        true
    }
}
