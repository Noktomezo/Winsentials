pub mod types;

use std::collections::HashSet;
use std::env;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};
use std::process::Command;

use types::{
    CleanupAccessEntry, CleanupAccessReport, CleanupCategoryReport, CleanupEntry,
    CleanupEntryStatus, CleanupScheduleEntry, CleanupScheduleReport,
};

use crate::error::AppError;

const WINDOWS_TEMP_CATEGORY: &str = "windows_temp";
const THUMBNAIL_CACHE_CATEGORY: &str = "thumbnail_cache";
const BROWSER_CACHE_CATEGORY: &str = "browser_cache";
const DRIVER_CACHE_CATEGORY: &str = "driver_cache";
const GAME_CACHE_CATEGORY: &str = "game_cache";
const WINDOWS_LOGS_CATEGORY: &str = "windows_logs";
const SYSTEM_ERROR_REPORTS_CATEGORY: &str = "system_error_reports";
const APP_CACHE_CATEGORY: &str = "app_cache";
const UNUSED_DEVICES_CATEGORY: &str = "unused_devices";

struct CleanupTarget {
    id: &'static str,
    name: &'static str,
    path: &'static str,
}

struct ResolvedTarget {
    id: String,
    name: String,
    path: PathBuf,
}

const WINDOWS_TEMP_TARGETS: &[CleanupTarget] = &[
    CleanupTarget {
        id: "user_temp",
        name: "User Temp",
        path: "{TEMP}",
    },
    CleanupTarget {
        id: "local_temp",
        name: "Local AppData Temp",
        path: "{LOCALAPPDATA}\\Temp",
    },
    CleanupTarget {
        id: "windows_temp",
        name: "Windows Temp",
        path: "{WINDIR}\\Temp",
    },
    CleanupTarget {
        id: "root_temp",
        name: "Root Temp",
        path: "C:\\Temp",
    },
    CleanupTarget {
        id: "prefetch",
        name: "Windows Prefetch",
        path: "{WINDIR}\\Prefetch",
    },
    CleanupTarget {
        id: "recent_items",
        name: "Recent Items",
        path: "{APPDATA}\\Microsoft\\Windows\\Recent",
    },
    CleanupTarget {
        id: "inet_cache",
        name: "Windows INetCache",
        path: "{LOCALAPPDATA}\\Microsoft\\Windows\\INetCache",
    },
    CleanupTarget {
        id: "delivery_optimization_cache",
        name: "Delivery Optimization Cache",
        path: "{WINDIR}\\SoftwareDistribution\\Download",
    },
];

const THUMBNAIL_CACHE_TARGETS: &[CleanupTarget] = &[
    CleanupTarget {
        id: "explorer_thumbcache",
        name: "Explorer Thumbnail Cache",
        path: "{LOCALAPPDATA}\\Microsoft\\Windows\\Explorer\\thumbcache_*.db",
    },
    CleanupTarget {
        id: "explorer_iconcache",
        name: "Explorer Icon Cache",
        path: "{LOCALAPPDATA}\\Microsoft\\Windows\\Explorer\\iconcache_*.db",
    },
    CleanupTarget {
        id: "iconcache_db",
        name: "Icon Cache Database",
        path: "{LOCALAPPDATA}\\IconCache.db",
    },
];

const BROWSER_CACHE_TARGETS: &[CleanupTarget] = &[
    CleanupTarget {
        id: "chrome_cache",
        name: "Google Chrome Cache",
        path: "{LOCALAPPDATA}\\Google\\Chrome\\User Data\\*\\Cache",
    },
    CleanupTarget {
        id: "chrome_code_cache",
        name: "Google Chrome Code Cache",
        path: "{LOCALAPPDATA}\\Google\\Chrome\\User Data\\*\\Code Cache",
    },
    CleanupTarget {
        id: "chrome_gpu_cache",
        name: "Google Chrome GPUCache",
        path: "{LOCALAPPDATA}\\Google\\Chrome\\User Data\\*\\GPUCache",
    },
    CleanupTarget {
        id: "edge_cache",
        name: "Microsoft Edge Cache",
        path: "{LOCALAPPDATA}\\Microsoft\\Edge\\User Data\\*\\Cache",
    },
    CleanupTarget {
        id: "edge_code_cache",
        name: "Microsoft Edge Code Cache",
        path: "{LOCALAPPDATA}\\Microsoft\\Edge\\User Data\\*\\Code Cache",
    },
    CleanupTarget {
        id: "edge_gpu_cache",
        name: "Microsoft Edge GPUCache",
        path: "{LOCALAPPDATA}\\Microsoft\\Edge\\User Data\\*\\GPUCache",
    },
    CleanupTarget {
        id: "brave_cache",
        name: "Brave Cache",
        path: "{LOCALAPPDATA}\\BraveSoftware\\Brave-Browser\\User Data\\*\\Cache",
    },
    CleanupTarget {
        id: "vivaldi_cache",
        name: "Vivaldi Cache",
        path: "{LOCALAPPDATA}\\Vivaldi\\User Data\\*\\Cache",
    },
    CleanupTarget {
        id: "opera_cache",
        name: "Opera Cache",
        path: "{APPDATA}\\Opera Software\\Opera Stable\\Cache",
    },
    CleanupTarget {
        id: "opera_gx_cache",
        name: "Opera GX Cache",
        path: "{APPDATA}\\Opera Software\\Opera GX Stable\\Cache",
    },
    CleanupTarget {
        id: "firefox_cache",
        name: "Mozilla Firefox Cache",
        path: "{LOCALAPPDATA}\\Mozilla\\Firefox\\Profiles\\*\\cache2",
    },
    CleanupTarget {
        id: "yandex_cache",
        name: "Yandex Browser Cache",
        path: "{LOCALAPPDATA}\\Yandex\\YandexBrowser\\User Data\\*\\Cache",
    },
];

const DRIVER_CACHE_TARGETS: &[CleanupTarget] = &[
    CleanupTarget {
        id: "direct3d_shader_cache",
        name: "Direct3D Shader Cache",
        path: "{LOCALAPPDATA}\\D3DSCache",
    },
    CleanupTarget {
        id: "amd_dx_cache",
        name: "AMD DirectX Shader Cache",
        path: "{LOCALAPPDATA}\\AMD\\DxCache",
    },
    CleanupTarget {
        id: "amd_dxc_cache",
        name: "AMD DXC Shader Cache",
        path: "{LOCALAPPDATA}\\AMD\\DxcCache",
    },
    CleanupTarget {
        id: "amd_vk_cache",
        name: "AMD Vulkan Shader Cache",
        path: "{LOCALAPPDATA}\\AMD\\VkCache",
    },
    CleanupTarget {
        id: "amd_legacy_dx_cache",
        name: "AMD Legacy Shader Cache",
        path: "{USERPROFILE}\\AppData\\LocalLow\\AMD\\DxCache",
    },
    CleanupTarget {
        id: "nvidia_dx_cache",
        name: "NVIDIA DirectX Shader Cache",
        path: "{LOCALAPPDATA}\\NVIDIA\\DXCache",
    },
    CleanupTarget {
        id: "nvidia_gl_cache",
        name: "NVIDIA OpenGL Shader Cache",
        path: "{LOCALAPPDATA}\\NVIDIA\\GLCache",
    },
    CleanupTarget {
        id: "nvidia_per_driver_dx_cache",
        name: "NVIDIA Per-Driver DirectX Shader Cache",
        path: "{USERPROFILE}\\AppData\\LocalLow\\NVIDIA\\PerDriverVersion\\DXCache",
    },
    CleanupTarget {
        id: "nvidia_per_driver_gl_cache",
        name: "NVIDIA Per-Driver OpenGL Shader Cache",
        path: "{USERPROFILE}\\AppData\\LocalLow\\NVIDIA\\PerDriverVersion\\GLCache",
    },
    CleanupTarget {
        id: "nvidia_compute_cache",
        name: "NVIDIA Compute Cache",
        path: "{APPDATA}\\NVIDIA\\ComputeCache",
    },
    CleanupTarget {
        id: "nvidia_downloader_cache",
        name: "NVIDIA Driver Download Cache",
        path: "{PROGRAMDATA}\\NVIDIA Corporation\\Downloader",
    },
    CleanupTarget {
        id: "nvidia_nv_cache",
        name: "NVIDIA NV Cache",
        path: "{LOCALAPPDATA}\\NVIDIA\\NvBackend\\Cache",
    },
    CleanupTarget {
        id: "nvidia_physx_cache",
        name: "NVIDIA PhysX Cache",
        path: "{LOCALAPPDATA}\\NVIDIA Corporation\\PhysX\\Cache",
    },
    CleanupTarget {
        id: "intel_shader_cache",
        name: "Intel Shader Cache",
        path: "{USERPROFILE}\\AppData\\LocalLow\\Intel\\ShaderCache",
    },
    CleanupTarget {
        id: "intel_local_shader_cache",
        name: "Intel Local Shader Cache",
        path: "{LOCALAPPDATA}\\Intel\\ShaderCache",
    },
    CleanupTarget {
        id: "amd_installer_leftovers",
        name: "AMD Driver Installer Leftovers",
        path: "C:\\AMD",
    },
    CleanupTarget {
        id: "amd_temp_leftovers",
        name: "AMD Driver Temp Leftovers",
        path: "{TEMP}\\AMD",
    },
    CleanupTarget {
        id: "amd_tmp_leftovers",
        name: "AMD Driver TMP Leftovers",
        path: "{TMP}\\AMD",
    },
    CleanupTarget {
        id: "amd_local_temp_leftovers",
        name: "AMD Local Temp Leftovers",
        path: "{LOCALAPPDATA}\\Temp\\AMD",
    },
    CleanupTarget {
        id: "nvidia_temp_leftovers",
        name: "NVIDIA Driver Temp Leftovers",
        path: "{TEMP}\\NVIDIA",
    },
    CleanupTarget {
        id: "nvidia_tmp_leftovers",
        name: "NVIDIA Driver TMP Leftovers",
        path: "{TMP}\\NVIDIA",
    },
    CleanupTarget {
        id: "nvidia_local_temp_leftovers",
        name: "NVIDIA Local Temp Leftovers",
        path: "{LOCALAPPDATA}\\Temp\\NVIDIA",
    },
    CleanupTarget {
        id: "intel_temp_leftovers",
        name: "Intel Driver Temp Leftovers",
        path: "{TEMP}\\Intel",
    },
    CleanupTarget {
        id: "intel_tmp_leftovers",
        name: "Intel Driver TMP Leftovers",
        path: "{TMP}\\Intel",
    },
    CleanupTarget {
        id: "intel_local_temp_leftovers",
        name: "Intel Local Temp Leftovers",
        path: "{LOCALAPPDATA}\\Temp\\Intel",
    },
];

const GAME_CACHE_TARGETS: &[CleanupTarget] = &[
    CleanupTarget {
        id: "steam_htmlcache",
        name: "Steam HTML Cache",
        path: "{LOCALAPPDATA}\\Steam\\htmlcache",
    },
    CleanupTarget {
        id: "steam_shader_cache",
        name: "Steam Shader Cache",
        path: "{PROGRAMFILES_X86}\\Steam\\steamapps\\shadercache",
    },
    CleanupTarget {
        id: "steam_downloading_cache",
        name: "Steam Download Cache",
        path: "{PROGRAMFILES_X86}\\Steam\\steamapps\\downloading",
    },
    CleanupTarget {
        id: "epic_webcache",
        name: "Epic Games Launcher Web Cache",
        path: "{LOCALAPPDATA}\\EpicGamesLauncher\\Saved\\webcache*",
    },
    CleanupTarget {
        id: "ea_desktop_cache",
        name: "EA Desktop Cache",
        path: "{LOCALAPPDATA}\\Electronic Arts\\EA Desktop\\cache",
    },
    CleanupTarget {
        id: "origin_cache",
        name: "Origin Cache",
        path: "{PROGRAMDATA}\\Origin\\DownloadCache",
    },
];

const WINDOWS_LOGS_TARGETS: &[CleanupTarget] = &[
    CleanupTarget {
        id: "windows_cbs_logs",
        name: "Windows CBS Logs",
        path: "{WINDIR}\\Logs\\CBS",
    },
    CleanupTarget {
        id: "windows_dism_logs",
        name: "Windows DISM Logs",
        path: "{WINDIR}\\Logs\\DISM",
    },
    CleanupTarget {
        id: "windows_setup_logs",
        name: "Windows Setup Logs",
        path: "{WINDIR}\\Panther",
    },
    CleanupTarget {
        id: "windows_logfiles",
        name: "Windows LogFiles",
        path: "{WINDIR}\\System32\\LogFiles",
    },
    CleanupTarget {
        id: "setupapi_logs",
        name: "SetupAPI Logs",
        path: "{WINDIR}\\INF\\setupapi*.log",
    },
];

const SYSTEM_ERROR_REPORTS_TARGETS: &[CleanupTarget] = &[
    CleanupTarget {
        id: "windows_error_reports_archive",
        name: "Windows Error Reports Archive",
        path: "{PROGRAMDATA}\\Microsoft\\Windows\\WER\\ReportArchive",
    },
    CleanupTarget {
        id: "windows_error_reports_queue",
        name: "Windows Error Reports Queue",
        path: "{PROGRAMDATA}\\Microsoft\\Windows\\WER\\ReportQueue",
    },
    CleanupTarget {
        id: "user_error_reports_archive",
        name: "User Error Reports Archive",
        path: "{LOCALAPPDATA}\\Microsoft\\Windows\\WER\\ReportArchive",
    },
    CleanupTarget {
        id: "user_error_reports_queue",
        name: "User Error Reports Queue",
        path: "{LOCALAPPDATA}\\Microsoft\\Windows\\WER\\ReportQueue",
    },
    CleanupTarget {
        id: "minidumps",
        name: "Windows Minidumps",
        path: "{WINDIR}\\Minidump",
    },
    CleanupTarget {
        id: "memory_dump",
        name: "Windows Memory Dump",
        path: "{WINDIR}\\MEMORY.DMP",
    },
];

const APP_CACHE_TARGETS: &[CleanupTarget] = &[
    CleanupTarget {
        id: "microsoft_store_cache",
        name: "Microsoft Store Cache",
        path: "{LOCALAPPDATA}\\Packages\\Microsoft.WindowsStore_*\\LocalCache",
    },
    CleanupTarget {
        id: "windows_live_tiles_cache",
        name: "Windows Live Tiles Cache",
        path: "{LOCALAPPDATA}\\Packages\\Microsoft.Windows.StartMenuExperienceHost_*\\LocalState\\Cache",
    },
    CleanupTarget {
        id: "windows_notifications_cache",
        name: "Windows Notifications Cache",
        path: "{LOCALAPPDATA}\\Microsoft\\Windows\\Notifications",
    },
    CleanupTarget {
        id: "onedrive_logs",
        name: "OneDrive Logs",
        path: "{LOCALAPPDATA}\\Microsoft\\OneDrive\\logs",
    },
    CleanupTarget {
        id: "office_file_cache",
        name: "Microsoft Office File Cache",
        path: "{LOCALAPPDATA}\\Microsoft\\Office\\16.0\\OfficeFileCache",
    },
    CleanupTarget {
        id: "teams_cache",
        name: "Microsoft Teams Cache",
        path: "{APPDATA}\\Microsoft\\Teams\\Cache",
    },
    CleanupTarget {
        id: "discord_cache",
        name: "Discord Cache",
        path: "{APPDATA}\\discord\\Cache",
    },
];

pub fn cleanup_scan_category(category_id: &str) -> Result<CleanupCategoryReport, AppError> {
    build_report(category_id, false)
}

pub fn cleanup_clean_category(category_id: &str) -> Result<CleanupCategoryReport, AppError> {
    build_report(category_id, true)
}

pub fn cleanup_prepare_access() -> Result<CleanupAccessReport, AppError> {
    let targets = dedupe_resolved_targets(
        [
            WINDOWS_TEMP_TARGETS,
            THUMBNAIL_CACHE_TARGETS,
            BROWSER_CACHE_TARGETS,
            DRIVER_CACHE_TARGETS,
            GAME_CACHE_TARGETS,
            WINDOWS_LOGS_TARGETS,
            SYSTEM_ERROR_REPORTS_TARGETS,
            APP_CACHE_TARGETS,
        ]
        .into_iter()
        .flatten()
        .flat_map(resolve_target)
        .collect(),
    );

    let entries = targets
        .into_iter()
        .filter(|target| target.path.exists())
        .map(|target| prepare_target_access(&target))
        .collect();

    Ok(CleanupAccessReport { entries })
}

pub fn cleanup_schedule_delete_on_reboot(
    paths: &[String],
) -> Result<CleanupScheduleReport, AppError> {
    let allowed_roots = resolved_cleanup_roots();
    let entries = paths
        .iter()
        .map(|path| schedule_requested_path(path, &allowed_roots))
        .collect();

    Ok(CleanupScheduleReport { entries })
}

fn all_resolved_cleanup_targets() -> Vec<ResolvedTarget> {
    dedupe_resolved_targets(
        [
            WINDOWS_TEMP_TARGETS,
            THUMBNAIL_CACHE_TARGETS,
            BROWSER_CACHE_TARGETS,
            DRIVER_CACHE_TARGETS,
            GAME_CACHE_TARGETS,
            WINDOWS_LOGS_TARGETS,
            SYSTEM_ERROR_REPORTS_TARGETS,
            APP_CACHE_TARGETS,
        ]
        .into_iter()
        .flatten()
        .flat_map(resolve_target)
        .collect(),
    )
}

fn build_report(category_id: &str, clean: bool) -> Result<CleanupCategoryReport, AppError> {
    if category_id == UNUSED_DEVICES_CATEGORY {
        return build_unused_devices_report(clean);
    }

    let targets = dedupe_resolved_targets(
        targets_for_category(category_id)?
            .iter()
            .flat_map(resolve_target)
            .collect(),
    );

    let entries = targets
        .iter()
        .map(|target| scan_or_clean_target(target, clean))
        .collect();

    Ok(CleanupCategoryReport {
        id: category_id.to_string(),
        entries,
    })
}

fn targets_for_category(category_id: &str) -> Result<&'static [CleanupTarget], AppError> {
    match category_id {
        WINDOWS_TEMP_CATEGORY => Ok(WINDOWS_TEMP_TARGETS),
        THUMBNAIL_CACHE_CATEGORY => Ok(THUMBNAIL_CACHE_TARGETS),
        BROWSER_CACHE_CATEGORY => Ok(BROWSER_CACHE_TARGETS),
        DRIVER_CACHE_CATEGORY => Ok(DRIVER_CACHE_TARGETS),
        GAME_CACHE_CATEGORY => Ok(GAME_CACHE_TARGETS),
        WINDOWS_LOGS_CATEGORY => Ok(WINDOWS_LOGS_TARGETS),
        SYSTEM_ERROR_REPORTS_CATEGORY => Ok(SYSTEM_ERROR_REPORTS_TARGETS),
        APP_CACHE_CATEGORY => Ok(APP_CACHE_TARGETS),
        _ => Err(AppError::message(format!(
            "unknown cleanup category `{category_id}`"
        ))),
    }
}

fn resolve_target(target: &CleanupTarget) -> Vec<ResolvedTarget> {
    let Some(path) = expand_env_path(target.path) else {
        return vec![];
    };

    expand_wildcard_path(PathBuf::from(path))
        .into_iter()
        .enumerate()
        .map(|(index, path)| ResolvedTarget {
            id: if index == 0 {
                target.id.to_string()
            } else {
                format!("{}::{index}", target.id)
            },
            name: resolved_target_name(target.name, &path, index),
            path,
        })
        .collect()
}

fn expand_wildcard_path(path: PathBuf) -> Vec<PathBuf> {
    if !path.to_string_lossy().contains('*') {
        return vec![path];
    }

    let mut paths = vec![PathBuf::new()];
    for component in path.components() {
        let pattern = match component {
            Component::Normal(value) => value.to_string_lossy().to_string(),
            Component::Prefix(_) | Component::RootDir | Component::CurDir => {
                paths
                    .iter_mut()
                    .for_each(|path| path.push(component.as_os_str()));
                continue;
            }
            Component::ParentDir => return vec![],
        };

        if !pattern.contains('*') {
            paths.iter_mut().for_each(|path| path.push(&pattern));
            continue;
        }

        paths = paths
            .into_iter()
            .flat_map(|parent| {
                let Ok(entries) = fs::read_dir(&parent) else {
                    return vec![];
                };

                entries
                    .flatten()
                    .filter_map(|entry| {
                        let name = entry.file_name().to_string_lossy().to_string();
                        wildcard_match(&pattern, &name).then(|| entry.path())
                    })
                    .collect::<Vec<_>>()
            })
            .collect();
    }

    paths
}

fn wildcard_match(pattern: &str, value: &str) -> bool {
    let pattern = pattern.to_ascii_lowercase();
    let value = value.to_ascii_lowercase();
    let parts: Vec<&str> = pattern.split('*').collect();

    if parts.len() == 1 {
        return pattern == value;
    }

    let mut cursor = 0;
    for (index, part) in parts.iter().enumerate() {
        if part.is_empty() {
            continue;
        }

        let Some(position) = value[cursor..].find(part) else {
            return false;
        };

        if index == 0 && position != 0 {
            return false;
        }

        cursor += position + part.len();
    }

    pattern.ends_with('*') || parts.last().is_none_or(|part| value.ends_with(part))
}

fn resolved_target_name(base_name: &str, path: &Path, index: usize) -> String {
    if index == 0 {
        return base_name.to_string();
    }

    path.file_name()
        .map(|name| format!("{} ({})", base_name, name.to_string_lossy()))
        .unwrap_or_else(|| format!("{} ({index})", base_name))
}

fn dedupe_resolved_targets(targets: Vec<ResolvedTarget>) -> Vec<ResolvedTarget> {
    let mut seen_paths = HashSet::new();
    targets
        .into_iter()
        .filter(|target| seen_paths.insert(cleanup_path_key(&target.path)))
        .collect()
}

fn cleanup_path_key(path: &Path) -> String {
    normalize_cleanup_path(path)
        .unwrap_or_else(|| path.to_path_buf())
        .to_string_lossy()
        .to_ascii_lowercase()
}

#[derive(Clone)]
struct GhostDevice {
    instance_id: String,
    description: String,
    friendly_name: Option<String>,
    class_name: Option<String>,
    problem_code: u32,
    config_flags: u32,
    driver_inf_path: Option<String>,
    driver_provider: Option<String>,
    icon_data_url: Option<String>,
}

fn build_unused_devices_report(clean: bool) -> Result<CleanupCategoryReport, AppError> {
    let entries = if clean {
        clean_unused_devices_entries().map_err(AppError::message)?
    } else {
        scan_unused_devices_entries().map_err(AppError::message)?
    };

    Ok(CleanupCategoryReport {
        id: UNUSED_DEVICES_CATEGORY.to_string(),
        entries,
    })
}

#[cfg(target_os = "windows")]
fn scan_unused_devices_entries() -> Result<Vec<CleanupEntry>, String> {
    Ok(enumerate_ghost_devices()?
        .into_iter()
        .map(ghost_device_to_cleanup_entry)
        .collect())
}

#[cfg(not(target_os = "windows"))]
fn scan_unused_devices_entries() -> Result<Vec<CleanupEntry>, String> {
    Err("unused device cleanup is supported only on Windows".to_string())
}

#[cfg(target_os = "windows")]
fn clean_unused_devices_entries() -> Result<Vec<CleanupEntry>, String> {
    let devices = enumerate_ghost_devices()?;
    let mut failed_entries = vec![];

    for device in devices {
        if let Err(error) = remove_ghost_device(&device.instance_id) {
            let mut entry = ghost_device_to_cleanup_entry(device);
            entry.status = CleanupEntryStatus::Failed;
            entry.error = Some(error);
            failed_entries.push(entry);
        }
    }

    let failed_ids = failed_entries
        .iter()
        .map(|entry| entry.id.to_ascii_lowercase())
        .collect::<HashSet<_>>();

    let mut entries = failed_entries;
    entries.extend(
        enumerate_ghost_devices()?
            .into_iter()
            .filter(|device| !failed_ids.contains(&device.instance_id.to_ascii_lowercase()))
            .map(ghost_device_to_cleanup_entry),
    );

    Ok(entries)
}

#[cfg(not(target_os = "windows"))]
fn clean_unused_devices_entries() -> Result<Vec<CleanupEntry>, String> {
    Err("unused device cleanup is supported only on Windows".to_string())
}

fn ghost_device_to_cleanup_entry(device: GhostDevice) -> CleanupEntry {
    let mut details = vec![];

    if let Some(inf_path) = device
        .driver_inf_path
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        details.push(format!("INF: {inf_path}"));
    }

    details.push(format!("Status: {}", ghost_device_status_label(&device)));

    if let Some(provider) = device
        .driver_provider
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        details.push(format!("Provider: {provider}"));
    }

    if let Some(class_name) = device
        .class_name
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        details.push(format!("Class: {class_name}"));
    }

    let detail = details.join(" • ");

    CleanupEntry {
        id: device.instance_id,
        name: device.friendly_name.unwrap_or(device.description),
        path: detail,
        status: CleanupEntryStatus::Pending,
        size_bytes: 0,
        error: None,
        icon_data_url: device.icon_data_url,
    }
}

fn ghost_device_status_label(device: &GhostDevice) -> String {
    const CONFIGFLAG_DISABLED: u32 = 0x0000_0001;

    if device.config_flags & CONFIGFLAG_DISABLED != 0 {
        return "disabled".to_string();
    }

    if device.problem_code != 0 {
        return format!("problem code {}", device.problem_code);
    }

    "disconnected".to_string()
}

#[cfg(target_os = "windows")]
fn enumerate_ghost_devices() -> Result<Vec<GhostDevice>, String> {
    let present_ids = enumerate_device_instance_ids(true)?
        .into_iter()
        .map(|id| id.to_ascii_lowercase())
        .collect::<HashSet<_>>();

    enumerate_devices(false).map(|devices| {
        devices
            .into_iter()
            .filter(|device| !present_ids.contains(&device.instance_id.to_ascii_lowercase()))
            .collect()
    })
}

#[cfg(target_os = "windows")]
fn enumerate_device_instance_ids(present_only: bool) -> Result<Vec<String>, String> {
    enumerate_devices(present_only).map(|devices| {
        devices
            .into_iter()
            .map(|device| device.instance_id)
            .collect()
    })
}

#[cfg(target_os = "windows")]
fn enumerate_devices(present_only: bool) -> Result<Vec<GhostDevice>, String> {
    use std::mem::size_of;

    use windows::Win32::Devices::DeviceAndDriverInstallation::{
        CM_DEVNODE_STATUS_FLAGS, CM_Get_DevNode_Status, CM_PROB, CR_SUCCESS, DIGCF_ALLCLASSES,
        DIGCF_PRESENT, SP_DEVINFO_DATA, SPDRP_CLASS, SPDRP_CONFIGFLAGS, SPDRP_DEVICEDESC,
        SPDRP_DRIVER, SPDRP_FRIENDLYNAME, SetupDiDestroyDeviceInfoList, SetupDiEnumDeviceInfo,
        SetupDiGetClassDevsW,
    };
    use windows::core::PCWSTR;

    let flags = if present_only {
        DIGCF_ALLCLASSES | DIGCF_PRESENT
    } else {
        DIGCF_ALLCLASSES
    };

    let device_info_set = unsafe { SetupDiGetClassDevsW(None, PCWSTR::null(), None, flags) }
        .map_err(|error| error.to_string())?;

    let mut devices = vec![];
    let mut index = 0;

    loop {
        let mut device_info = SP_DEVINFO_DATA {
            cbSize: size_of::<SP_DEVINFO_DATA>() as u32,
            ..Default::default()
        };

        if unsafe { SetupDiEnumDeviceInfo(device_info_set, index, &mut device_info) }.is_err() {
            break;
        }
        index += 1;

        let Some(instance_id) = device_instance_id(device_info_set, &device_info) else {
            continue;
        };
        let description = device_registry_string(device_info_set, &device_info, SPDRP_DEVICEDESC)
            .unwrap_or_else(|| instance_id.clone());
        let friendly_name =
            device_registry_string(device_info_set, &device_info, SPDRP_FRIENDLYNAME);
        let class_name = device_registry_string(device_info_set, &device_info, SPDRP_CLASS);
        let config_flags =
            device_registry_u32(device_info_set, &device_info, SPDRP_CONFIGFLAGS).unwrap_or(0);
        let driver_key = device_registry_string(device_info_set, &device_info, SPDRP_DRIVER);
        let driver_info = driver_key
            .as_deref()
            .and_then(device_driver_registry_info)
            .unwrap_or_default();
        let icon_data_url = device_icon_data_url(device_info_set, &device_info);

        let mut status = CM_DEVNODE_STATUS_FLAGS(0);
        let mut problem = CM_PROB(0);
        let problem_code =
            if unsafe { CM_Get_DevNode_Status(&mut status, &mut problem, device_info.DevInst, 0) }
                == CR_SUCCESS
            {
                problem.0
            } else {
                0
            };

        devices.push(GhostDevice {
            instance_id,
            description,
            friendly_name,
            class_name,
            problem_code,
            config_flags,
            driver_inf_path: driver_info.inf_path,
            driver_provider: driver_info.provider,
            icon_data_url,
        });
    }

    let _ = unsafe { SetupDiDestroyDeviceInfoList(device_info_set) };
    Ok(devices)
}

#[cfg(target_os = "windows")]
fn device_icon_data_url(
    device_info_set: windows::Win32::Devices::DeviceAndDriverInstallation::HDEVINFO,
    device_info: &windows::Win32::Devices::DeviceAndDriverInstallation::SP_DEVINFO_DATA,
) -> Option<String> {
    use windows::Win32::Devices::DeviceAndDriverInstallation::SetupDiLoadDeviceIcon;
    use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, HICON};

    const ICON_SIZE: u32 = 32;

    let mut icon = HICON::default();
    unsafe {
        SetupDiLoadDeviceIcon(
            device_info_set,
            device_info,
            ICON_SIZE,
            ICON_SIZE,
            0,
            &mut icon,
        )
    }
    .ok()?;

    if icon.0.is_null() {
        return None;
    }

    let encoded = icon_to_png_data_url(icon);
    let _ = unsafe { DestroyIcon(icon) };
    encoded
}

#[cfg(target_os = "windows")]
fn icon_to_png_data_url(icon: windows::Win32::UI::WindowsAndMessaging::HICON) -> Option<String> {
    use std::ffi::c_void;
    use std::mem::size_of;
    use std::ptr::null_mut;

    use base64::Engine;
    use png::{BitDepth, ColorType, Encoder};
    use windows::Win32::Graphics::Gdi::{
        BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleDC, CreateDIBSection, DIB_RGB_COLORS,
        DeleteDC, DeleteObject, GetDIBits, HGDIOBJ, SelectObject,
    };
    use windows::Win32::UI::WindowsAndMessaging::{DI_NORMAL, DrawIconEx};

    const ICON_SIZE: i32 = 32;

    let mut bitmap_info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: ICON_SIZE,
            biHeight: -ICON_SIZE,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };

    let device_context = unsafe { CreateCompatibleDC(None) };
    if device_context.0.is_null() {
        return None;
    }

    let mut bits = null_mut();
    let bitmap =
        match unsafe { CreateDIBSection(None, &bitmap_info, DIB_RGB_COLORS, &mut bits, None, 0) } {
            Ok(bitmap) => bitmap,
            Err(_) => {
                let _ = unsafe { DeleteDC(device_context) };
                return None;
            }
        };

    let previous = unsafe { SelectObject(device_context, HGDIOBJ(bitmap.0)) };
    if previous.0.is_null() {
        let _ = unsafe { DeleteObject(HGDIOBJ(bitmap.0)) };
        let _ = unsafe { DeleteDC(device_context) };
        return None;
    }

    if unsafe {
        DrawIconEx(
            device_context,
            0,
            0,
            icon,
            ICON_SIZE,
            ICON_SIZE,
            0,
            None,
            DI_NORMAL,
        )
    }
    .is_err()
    {
        let _ = unsafe { SelectObject(device_context, previous) };
        let _ = unsafe { DeleteObject(HGDIOBJ(bitmap.0)) };
        let _ = unsafe { DeleteDC(device_context) };
        return None;
    }

    let mut pixels = vec![0u8; (ICON_SIZE * ICON_SIZE * 4) as usize];
    let scanlines = unsafe {
        GetDIBits(
            device_context,
            bitmap,
            0,
            ICON_SIZE as u32,
            Some(pixels.as_mut_ptr() as *mut c_void),
            &mut bitmap_info,
            DIB_RGB_COLORS,
        )
    };

    let _ = unsafe { SelectObject(device_context, previous) };
    let _ = unsafe { DeleteObject(HGDIOBJ(bitmap.0)) };
    let _ = unsafe { DeleteDC(device_context) };

    if scanlines == 0 {
        return None;
    }

    for chunk in pixels.chunks_exact_mut(4) {
        chunk.swap(0, 2);
    }

    let mut png_data = Vec::new();
    let mut encoder = Encoder::new(&mut png_data, ICON_SIZE as u32, ICON_SIZE as u32);
    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);

    {
        let mut writer = encoder.write_header().ok()?;
        writer.write_image_data(&pixels).ok()?;
    }

    Some(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(png_data)
    ))
}

#[cfg(target_os = "windows")]
fn device_instance_id(
    device_info_set: windows::Win32::Devices::DeviceAndDriverInstallation::HDEVINFO,
    device_info: &windows::Win32::Devices::DeviceAndDriverInstallation::SP_DEVINFO_DATA,
) -> Option<String> {
    use windows::Win32::Devices::DeviceAndDriverInstallation::SetupDiGetDeviceInstanceIdW;

    let mut required_size = 0;
    let _ = unsafe {
        SetupDiGetDeviceInstanceIdW(device_info_set, device_info, None, Some(&mut required_size))
    };
    let mut buffer = vec![0u16; required_size.max(512) as usize];

    unsafe {
        SetupDiGetDeviceInstanceIdW(
            device_info_set,
            device_info,
            Some(&mut buffer),
            Some(&mut required_size),
        )
    }
    .ok()?;

    Some(utf16_null_terminated_to_string(&buffer))
}

#[cfg(target_os = "windows")]
fn device_registry_string(
    device_info_set: windows::Win32::Devices::DeviceAndDriverInstallation::HDEVINFO,
    device_info: &windows::Win32::Devices::DeviceAndDriverInstallation::SP_DEVINFO_DATA,
    property: windows::Win32::Devices::DeviceAndDriverInstallation::SETUP_DI_REGISTRY_PROPERTY,
) -> Option<String> {
    let mut buffer = vec![0u8; 4096];
    let mut property_type = 0;
    unsafe {
        windows::Win32::Devices::DeviceAndDriverInstallation::SetupDiGetDeviceRegistryPropertyW(
            device_info_set,
            device_info,
            property,
            Some(&mut property_type),
            Some(&mut buffer),
            None,
        )
    }
    .ok()?;

    let value = utf16_bytes_to_strings(&buffer).into_iter().next()?;
    (!value.is_empty()).then_some(value)
}

#[derive(Default)]
#[cfg(target_os = "windows")]
struct DeviceDriverRegistryInfo {
    inf_path: Option<String>,
    provider: Option<String>,
}

#[cfg(target_os = "windows")]
fn device_driver_registry_info(driver_key: &str) -> Option<DeviceDriverRegistryInfo> {
    use winreg::RegKey as WinRegKey;
    use winreg::enums::HKEY_LOCAL_MACHINE;

    let hklm = WinRegKey::predef(HKEY_LOCAL_MACHINE);
    let key = hklm
        .open_subkey(format!(
            r"SYSTEM\CurrentControlSet\Control\Class\{driver_key}"
        ))
        .ok()?;

    Some(DeviceDriverRegistryInfo {
        inf_path: key.get_value("InfPath").ok(),
        provider: key.get_value("ProviderName").ok(),
    })
}

#[cfg(target_os = "windows")]
fn device_registry_u32(
    device_info_set: windows::Win32::Devices::DeviceAndDriverInstallation::HDEVINFO,
    device_info: &windows::Win32::Devices::DeviceAndDriverInstallation::SP_DEVINFO_DATA,
    property: windows::Win32::Devices::DeviceAndDriverInstallation::SETUP_DI_REGISTRY_PROPERTY,
) -> Option<u32> {
    use std::mem::size_of;

    let mut buffer = [0u8; size_of::<u32>()];
    let mut property_type = 0;
    unsafe {
        windows::Win32::Devices::DeviceAndDriverInstallation::SetupDiGetDeviceRegistryPropertyW(
            device_info_set,
            device_info,
            property,
            Some(&mut property_type),
            Some(&mut buffer),
            None,
        )
    }
    .ok()?;

    Some(u32::from_le_bytes(buffer))
}

#[cfg(target_os = "windows")]
fn utf16_bytes_to_strings(buffer: &[u8]) -> Vec<String> {
    let words = buffer
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect::<Vec<_>>();

    let mut strings = vec![];
    let mut start = 0;
    for (index, value) in words.iter().enumerate() {
        if *value != 0 {
            continue;
        }

        if index > start {
            strings.push(String::from_utf16_lossy(&words[start..index]));
        }
        start = index + 1;
    }

    strings
}

#[cfg(target_os = "windows")]
fn utf16_null_terminated_to_string(buffer: &[u16]) -> String {
    let end = buffer
        .iter()
        .position(|value| *value == 0)
        .unwrap_or(buffer.len());
    String::from_utf16_lossy(&buffer[..end])
}

#[cfg(target_os = "windows")]
fn remove_ghost_device(instance_id: &str) -> Result<(), String> {
    use std::mem::size_of;

    use windows::Win32::Devices::DeviceAndDriverInstallation::{
        DI_REMOVEDEVICE_GLOBAL, DIF_REMOVE, DIGCF_ALLCLASSES, SP_CLASSINSTALL_HEADER,
        SP_DEVINFO_DATA, SP_REMOVEDEVICE_PARAMS, SetupDiCallClassInstaller,
        SetupDiDestroyDeviceInfoList, SetupDiEnumDeviceInfo, SetupDiGetClassDevsW,
        SetupDiSetClassInstallParamsW,
    };
    use windows::core::PCWSTR;

    let device_info_set =
        unsafe { SetupDiGetClassDevsW(None, PCWSTR::null(), None, DIGCF_ALLCLASSES) }
            .map_err(|error| error.to_string())?;
    let instance_id_key = instance_id.to_ascii_lowercase();
    let mut index = 0;
    let mut result = Err("device instance was not found".to_string());

    loop {
        let mut device_info = SP_DEVINFO_DATA {
            cbSize: size_of::<SP_DEVINFO_DATA>() as u32,
            ..Default::default()
        };

        if unsafe { SetupDiEnumDeviceInfo(device_info_set, index, &mut device_info) }.is_err() {
            break;
        }
        index += 1;

        let Some(current_instance_id) = device_instance_id(device_info_set, &device_info) else {
            continue;
        };
        if current_instance_id.to_ascii_lowercase() != instance_id_key {
            continue;
        }

        let remove_params = SP_REMOVEDEVICE_PARAMS {
            ClassInstallHeader: SP_CLASSINSTALL_HEADER {
                cbSize: size_of::<SP_CLASSINSTALL_HEADER>() as u32,
                InstallFunction: DIF_REMOVE,
            },
            Scope: DI_REMOVEDEVICE_GLOBAL,
            HwProfile: 0,
        };

        result = unsafe {
            SetupDiSetClassInstallParamsW(
                device_info_set,
                Some(&device_info),
                Some(&remove_params.ClassInstallHeader),
                size_of::<SP_REMOVEDEVICE_PARAMS>() as u32,
            )
        }
        .map_err(|error| error.to_string())
        .and_then(|()| {
            unsafe { SetupDiCallClassInstaller(DIF_REMOVE, device_info_set, Some(&device_info)) }
                .map_err(|error| error.to_string())
        });
        break;
    }

    let _ = unsafe { SetupDiDestroyDeviceInfoList(device_info_set) };
    result
}

fn expand_env_path(path: &str) -> Option<String> {
    expand_env_path_with(path, |key| env::var(key).ok())
}

fn expand_env_path_with(path: &str, lookup: impl Fn(&str) -> Option<String>) -> Option<String> {
    let mut expanded = path.to_string();
    for (placeholder_key, env_key) in [
        ("TEMP", "TEMP"),
        ("TMP", "TMP"),
        ("LOCALAPPDATA", "LOCALAPPDATA"),
        ("APPDATA", "APPDATA"),
        ("USERPROFILE", "USERPROFILE"),
        ("PROGRAMDATA", "PROGRAMDATA"),
        ("PROGRAMFILES", "PROGRAMFILES"),
        ("PROGRAMFILES_X86", "ProgramFiles(x86)"),
        ("WINDIR", "WINDIR"),
    ] {
        let placeholder = format!("{{{placeholder_key}}}");
        if expanded.contains(&placeholder) {
            let value = lookup(env_key)?;
            expanded = expanded.replace(&placeholder, &value);
        }
    }
    Some(expanded)
}

fn scan_or_clean_target(target: &ResolvedTarget, clean: bool) -> CleanupEntry {
    if clean {
        let delete_result = delete_target_contents(&target.path);
        let mut entry = scan_target(target);

        if matches!(delete_result, Ok(DeleteOutcome::ScheduledOnReboot)) {
            entry.status = CleanupEntryStatus::Busy;
            entry.error = Some("Scheduled for deletion on reboot".to_string());
        }

        if let Err(error) = delete_result {
            entry.status = cleanup_status_from_error(&error);
            entry.error = Some(error.to_string());
        }

        return entry;
    }

    scan_target(target)
}

fn scan_target(target: &ResolvedTarget) -> CleanupEntry {
    let path = target.path.to_string_lossy().to_string();
    match target_size_bytes(&target.path) {
        Ok(size_bytes) => CleanupEntry {
            id: target.id.clone(),
            name: target.name.clone(),
            path,
            status: if size_bytes == 0 {
                CleanupEntryStatus::Clean
            } else {
                CleanupEntryStatus::Pending
            },
            size_bytes,
            error: None,
            icon_data_url: None,
        },
        Err(error) if error.kind() == io::ErrorKind::NotFound => CleanupEntry {
            id: target.id.clone(),
            name: target.name.clone(),
            path,
            status: CleanupEntryStatus::Clean,
            size_bytes: 0,
            error: None,
            icon_data_url: None,
        },
        Err(error) => CleanupEntry {
            id: target.id.clone(),
            name: target.name.clone(),
            path,
            status: cleanup_status_from_error(&error),
            size_bytes: 0,
            error: Some(error.to_string()),
            icon_data_url: None,
        },
    }
}

fn target_size_bytes(path: &Path) -> io::Result<u64> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.is_file() {
        return Ok(metadata.len());
    }
    if !metadata.is_dir() {
        return Ok(0);
    }

    let mut total = 0;
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let metadata = fs::symlink_metadata(entry.path())?;
        if metadata.is_dir() {
            total += target_size_bytes(&entry.path())?;
        } else if metadata.is_file() {
            total += metadata.len();
        }
    }
    Ok(total)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum DeleteOutcome {
    Deleted,
    ScheduledOnReboot,
}

fn delete_target_contents(path: &Path) -> io::Result<DeleteOutcome> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(DeleteOutcome::Deleted),
        Err(error) => return Err(error),
    };

    if metadata.is_file() {
        return force_remove_path(path, false);
    }
    if !metadata.is_dir() {
        return Ok(DeleteOutcome::Deleted);
    }

    let mut first_error = None;
    let mut scheduled_on_reboot = false;
    for entry in fs::read_dir(path)? {
        let entry = match entry {
            Ok(entry) => entry,
            Err(error) => {
                first_error.get_or_insert(error);
                continue;
            }
        };
        let entry_path = entry.path();
        let result = match fs::symlink_metadata(&entry_path) {
            Ok(metadata) if metadata.is_dir() => force_remove_path(&entry_path, true),
            Ok(_) => force_remove_path(&entry_path, false),
            Err(error) => Err(error),
        };

        match result {
            Ok(DeleteOutcome::Deleted) => {}
            Ok(DeleteOutcome::ScheduledOnReboot) => scheduled_on_reboot = true,
            Err(error) => {
                first_error.get_or_insert(error);
            }
        }
    }

    match first_error {
        Some(error) => Err(error),
        None if scheduled_on_reboot => Ok(DeleteOutcome::ScheduledOnReboot),
        None => Ok(DeleteOutcome::Deleted),
    }
}

fn force_remove_path(path: &Path, recursive: bool) -> io::Result<DeleteOutcome> {
    clear_readonly(path);

    let result = if recursive {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    };

    match result {
        Ok(()) => Ok(DeleteOutcome::Deleted),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(DeleteOutcome::Deleted),
        Err(first_error) => force_remove_path_fallback(path, recursive, first_error),
    }
}

#[allow(
    clippy::permissions_set_readonly_false,
    reason = "Windows cleanup needs to clear the read-only DOS attribute before deletion"
)]
fn clear_readonly(path: &Path) {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return;
    };
    let mut permissions = metadata.permissions();
    if permissions.readonly() {
        permissions.set_readonly(false);
        let _ = fs::set_permissions(path, permissions);
    }
}

#[cfg(target_os = "windows")]
fn force_remove_path_fallback(
    path: &Path,
    recursive: bool,
    first_error: io::Error,
) -> io::Result<DeleteOutcome> {
    let path_text = path.to_string_lossy().to_string();
    let _ = run_command("attrib", &["-R", path_text.as_str(), "/S", "/D"]);
    let _ = run_command(
        "takeown",
        &["/F", path_text.as_str(), "/A", "/R", "/D", "Y"],
    );
    let _ = run_command(
        "icacls",
        &[path_text.as_str(), "/grant", "*S-1-5-32-544:F", "/T", "/C"],
    );

    clear_readonly(path);
    let retry_result = if recursive {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    };

    match retry_result {
        Ok(()) => Ok(DeleteOutcome::Deleted),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(DeleteOutcome::Deleted),
        Err(retry_error) => schedule_force_remove_path_on_reboot(path, recursive)
            .map(|()| DeleteOutcome::ScheduledOnReboot)
            .map_err(|schedule_error| {
                io::Error::new(
                    retry_error.kind(),
                    format!(
                        "delete failed: {first_error}; retry failed: {retry_error}; reboot delete failed: {schedule_error}"
                    ),
                )
            }),
    }
}

#[cfg(not(target_os = "windows"))]
fn force_remove_path_fallback(
    _path: &Path,
    _recursive: bool,
    first_error: io::Error,
) -> io::Result<DeleteOutcome> {
    Err(first_error)
}

#[cfg(target_os = "windows")]
fn schedule_force_remove_path_on_reboot(path: &Path, recursive: bool) -> Result<(), String> {
    if recursive {
        return schedule_path_tree_delete_on_reboot(path);
    }

    schedule_single_path_delete_on_reboot(path)
}

fn cleanup_status_from_error(error: &io::Error) -> CleanupEntryStatus {
    match error.kind() {
        io::ErrorKind::PermissionDenied | io::ErrorKind::WouldBlock => CleanupEntryStatus::Busy,
        _ => CleanupEntryStatus::Failed,
    }
}

fn resolved_cleanup_roots() -> Vec<PathBuf> {
    all_resolved_cleanup_targets()
        .into_iter()
        .map(|target| target.path)
        .collect()
}

fn schedule_requested_path(path: &str, allowed_roots: &[PathBuf]) -> CleanupScheduleEntry {
    let path_buf = PathBuf::from(path);
    let Some((allowed_path, is_target_root)) = allowed_cleanup_path(&path_buf, allowed_roots)
    else {
        return CleanupScheduleEntry {
            path: path.to_string(),
            success: false,
            error: Some("path is not a known cleanup target".to_string()),
        };
    };

    match schedule_allowed_path_delete_on_reboot(&allowed_path, is_target_root) {
        Ok(()) => CleanupScheduleEntry {
            path: path.to_string(),
            success: true,
            error: None,
        },
        Err(error) => CleanupScheduleEntry {
            path: path.to_string(),
            success: false,
            error: Some(error),
        },
    }
}

fn allowed_cleanup_path(path: &Path, allowed_roots: &[PathBuf]) -> Option<(PathBuf, bool)> {
    let path = normalize_cleanup_path(path)?;
    allowed_roots.iter().find_map(|root| {
        let root = normalize_cleanup_path(root)?;
        if path == root {
            return Some((path.clone(), true));
        }
        if path.starts_with(root) {
            return Some((path.clone(), false));
        }
        None
    })
}

fn normalize_cleanup_path(path: &Path) -> Option<PathBuf> {
    if let Ok(canonical_path) = path.canonicalize() {
        return Some(canonical_path);
    }

    if !path.is_absolute() {
        return None;
    }

    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => return None,
        }
    }

    Some(normalized)
}

fn schedule_allowed_path_delete_on_reboot(path: &Path, preserve_root: bool) -> Result<(), String> {
    if !preserve_root {
        return schedule_path_tree_delete_on_reboot(path);
    }

    if fs::symlink_metadata(path)
        .map(|metadata| metadata.is_file())
        .unwrap_or(false)
    {
        return schedule_single_path_delete_on_reboot(path);
    }

    let paths = collect_delete_on_reboot_paths(path, false)?;
    for path in paths {
        schedule_single_path_delete_on_reboot(&path)?;
    }
    Ok(())
}

fn collect_delete_on_reboot_paths(path: &Path, include_root: bool) -> Result<Vec<PathBuf>, String> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(vec![]),
        Err(error) => return Err(error.to_string()),
    };

    if metadata.is_dir() {
        let mut paths = vec![];
        let entries = fs::read_dir(path).map_err(|error| error.to_string())?;
        for entry in entries {
            let entry = entry.map_err(|error| error.to_string())?;
            paths.extend(collect_delete_on_reboot_paths(&entry.path(), true)?);
        }
        if include_root {
            paths.push(path.to_path_buf());
        }
        return Ok(paths);
    }

    if include_root {
        return Ok(vec![path.to_path_buf()]);
    }

    Ok(vec![])
}

fn schedule_path_tree_delete_on_reboot(path: &Path) -> Result<(), String> {
    for path in collect_delete_on_reboot_paths(path, true)? {
        schedule_single_path_delete_on_reboot(&path)?;
    }
    Ok(())
}

fn prepare_target_access(target: &ResolvedTarget) -> CleanupAccessEntry {
    let path = target.path.to_string_lossy().to_string();

    match prepare_path_access(&target.path) {
        Ok(()) => CleanupAccessEntry {
            id: target.id.clone(),
            name: target.name.clone(),
            path,
            success: true,
            error: None,
        },
        Err(error) => CleanupAccessEntry {
            id: target.id.clone(),
            name: target.name.clone(),
            path,
            success: false,
            error: Some(error),
        },
    }
}

#[cfg(target_os = "windows")]
fn prepare_path_access(path: &Path) -> Result<(), String> {
    let path = path.to_string_lossy().to_string();

    let _ = run_command("attrib", &["-R", path.as_str(), "/S", "/D"]);
    let takeown = run_command("takeown", &["/F", path.as_str(), "/A", "/R", "/D", "Y"]);
    let inheritance = run_command("icacls", &[path.as_str(), "/inheritance:e", "/T", "/C"]);
    let admins_grant = run_command(
        "icacls",
        &[path.as_str(), "/grant", "*S-1-5-32-544:F", "/T", "/C"],
    );
    let user_grant = current_user_sid().and_then(|sid| {
        run_command(
            "icacls",
            &[path.as_str(), "/grant", &format!("*{sid}:F"), "/T", "/C"],
        )
    });

    if admins_grant.is_ok() || user_grant.is_ok() {
        return Ok(());
    }

    Err(format!(
        "takeown: {}; inheritance: {}; administrators grant: {}; user grant: {}",
        result_error_text(takeown),
        result_error_text(inheritance),
        result_error_text(admins_grant),
        result_error_text(user_grant)
    ))
}

#[cfg(not(target_os = "windows"))]
fn prepare_path_access(_path: &Path) -> Result<(), String> {
    Err("access preparation is supported only on Windows".to_string())
}

#[cfg(target_os = "windows")]
fn current_user_sid() -> Result<String, String> {
    let format_flag = ['/', 'f', 'o'].iter().collect::<String>();
    let output = run_command("whoami", &["/user", format_flag.as_str(), "csv", "/nh"])?;
    output
        .split(',')
        .nth(1)
        .map(|value| value.trim().trim_matches('"').to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "failed to parse current user SID".to_string())
}

#[cfg(target_os = "windows")]
fn run_command(program: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .map_err(|error| format!("failed to start {program}: {error}"))?;

    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).trim().to_string());
    }

    Err(command_output_text(&output.stdout, &output.stderr))
}

#[cfg(target_os = "windows")]
fn result_error_text(result: Result<String, String>) -> String {
    match result {
        Ok(_) => "ok".to_string(),
        Err(error) => error,
    }
}

#[cfg(target_os = "windows")]
fn command_output_text(stdout: &[u8], stderr: &[u8]) -> String {
    let stderr = String::from_utf8_lossy(stderr).trim().to_string();
    if !stderr.is_empty() {
        return stderr;
    }

    String::from_utf8_lossy(stdout).trim().to_string()
}

#[cfg(target_os = "windows")]
fn schedule_single_path_delete_on_reboot(path: &Path) -> Result<(), String> {
    use std::os::windows::ffi::OsStrExt;

    use windows::Win32::Storage::FileSystem::{MOVEFILE_DELAY_UNTIL_REBOOT, MoveFileExW};
    use windows::core::PCWSTR;

    let wide_path: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    unsafe {
        MoveFileExW(
            PCWSTR(wide_path.as_ptr()),
            PCWSTR::null(),
            MOVEFILE_DELAY_UNTIL_REBOOT,
        )
    }
    .map_err(|error| error.to_string())
}

#[cfg(not(target_os = "windows"))]
fn schedule_single_path_delete_on_reboot(_path: &Path) -> Result<(), String> {
    Err("delete on reboot is supported only on Windows".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_env_path_requires_only_used_placeholders() {
        let expanded = expand_env_path_with("{TEMP}\\AMD", |key| match key {
            "TEMP" => Some("C:\\Temp".to_string()),
            _ => None,
        });

        assert_eq!(expanded.as_deref(), Some("C:\\Temp\\AMD"));

        let missing_used = expand_env_path_with("{LOCALAPPDATA}\\AMD", |key| match key {
            "TEMP" => Some("C:\\Temp".to_string()),
            _ => None,
        });

        assert!(missing_used.is_none());
    }

    #[test]
    fn driver_cache_targets_do_not_include_active_driver_roots() {
        for target in DRIVER_CACHE_TARGETS {
            let path = target.path.to_ascii_uppercase();
            assert!(!path.contains("DRIVERSTORE"), "{}", target.path);
            assert!(!path.contains("SYSTEM32"), "{}", target.path);
            assert!(!path.contains("PROGRAM FILES"), "{}", target.path);
        }
    }

    #[test]
    fn allowed_cleanup_path_rejects_traversal_and_marks_roots() {
        let root = env::temp_dir().join(format!("winsentials-cleanup-root-{}", std::process::id()));
        let child = root.join("child");
        let traversal = root.join("..").join("outside");
        let roots = vec![root.clone()];

        let root_match = allowed_cleanup_path(&root, &roots).expect("root should be allowed");
        assert!(root_match.1);

        let child_match = allowed_cleanup_path(&child, &roots).expect("child should be allowed");
        assert!(!child_match.1);

        assert!(allowed_cleanup_path(&traversal, &roots).is_none());
    }

    #[test]
    fn reboot_collection_can_preserve_target_root() {
        let root = env::temp_dir().join(format!(
            "winsentials-cleanup-collect-{}",
            std::process::id()
        ));
        let nested = root.join("nested");
        let file = root.join("cache.bin");
        let nested_file = nested.join("shader.bin");

        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&nested).expect("create nested test directory");
        fs::write(&file, b"cache").expect("write root file");
        fs::write(&nested_file, b"shader").expect("write nested file");

        let collected = collect_delete_on_reboot_paths(&root, false).expect("collect paths");

        assert!(!collected.iter().any(|path| path == &root));
        assert!(collected.iter().any(|path| path == &file));
        assert!(collected.iter().any(|path| path == &nested));
        assert!(collected.iter().any(|path| path == &nested_file));

        fs::remove_dir_all(&root).expect("remove test directory");
    }
}
