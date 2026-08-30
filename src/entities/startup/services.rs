use windows_registry::LOCAL_MACHINE;

use super::types::{StartupEntry, StartupScope, StartupSource, StartupStatus};

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
        let Ok(service_key) = services_root.open(&service_name) else {
            continue;
        };

        // 1. Check service type (Only Win32 services: 0x10 Own Process, 0x20 Share Process, 0x100/0x200 interactive)
        let Ok(svc_type) = service_key.get_u32("Type") else {
            continue;
        };
        if svc_type & 0x30 == 0 && svc_type & 0x100 == 0 && svc_type & 0x200 == 0 {
            continue; // Skip kernel / file system drivers
        }

        // 2. Read ImagePath
        let Ok(image_path) = service_key.get_string("ImagePath") else {
            continue;
        };

        // 3. Filter out Windows core internal services
        if is_windows_core_service(&service_name, &image_path) {
            continue;
        }

        // 4. Read Start type (2 = Auto, 3 = Manual, 4 = Disabled)
        let start_type = service_key.get_u32("Start").unwrap_or(3);
        let status = if start_type == 2 {
            StartupStatus::Enabled
        } else {
            StartupStatus::Disabled
        };

        // 5. Read DisplayName
        let display_name = service_key
            .get_string("DisplayName")
            .unwrap_or_else(|_| service_name.clone());

        let target_path = super::registry::extract_target_path(&image_path);

        entries.push(StartupEntry {
            id: format!("svc_{service_name}"),
            name: service_name.clone(),
            display_name,
            source: StartupSource::Service,
            scope: StartupScope::AllUsers,
            status,
            command: Some(image_path),
            target_path,
            location_label: "HKLM\\...\\Services".to_string(),
            raw_id: service_name,
        });
    }

    entries
}

fn is_windows_core_service(name: &str, image_path: &str) -> bool {
    let lower_path = image_path.to_ascii_lowercase();
    let lower_name = name.to_ascii_lowercase();

    for bin in SYSTEM_BINARIES {
        if lower_path.contains(bin) {
            return true;
        }
    }

    // Windows / Microsoft driver or standard services
    if (lower_path.starts_with(r"c:\windows\system32\")
        || lower_path.starts_with(r"%systemroot%\system32\")
        || lower_path.starts_with(r"\systemroot\system32\"))
        && !lower_path.contains("driver")
        && !lower_path.contains("thirdparty")
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
