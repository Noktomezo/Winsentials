use std::collections::HashMap;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use sysinfo::{CpuRefreshKind, DiskKind, Disks, MemoryRefreshKind, Networks, RefreshKind, System};

use crate::error::AppError;
use crate::registry::{Hive, RegKey};

// ─── Static structs ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticSystemInfo {
    pub windows: WindowsInfo,
    pub cpu: CpuInfo,
    pub ram: RamInfo,
    pub gpus: Vec<GpuInfo>,
    pub motherboard: MotherboardInfo,
    pub disks: Vec<DiskInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RamInfo {
    pub total_bytes: u64,
    pub speed_mhz: Option<u32>,
    pub used_slots: u32,
    pub total_slots: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowsInfo {
    pub product_name: String,
    pub display_version: String,
    pub build: u32,
    pub ubr: u32,
    pub hostname: String,
    pub username: String,
    pub architecture: String,
    pub activation_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub model: String,
    pub physical_cores: u32,
    pub logical_cores: u32,
    pub base_freq_mhz: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub model: String,
    pub vendor: String,
    pub vram_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotherboardInfo {
    pub manufacturer: String,
    pub product: String,
    pub bios_vendor: String,
    pub bios_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub kind: String,
    pub file_system: String,
}

// ─── Live structs ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveSystemInfo {
    pub cpu_usage_percent: f32,
    pub cpu_per_core: Vec<f32>,
    pub ram_used_bytes: u64,
    pub network: Vec<NetworkIfaceStats>,
    pub gpu_live: Vec<GpuLiveStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkIfaceStats {
    pub name: String,
    pub rx_bytes_per_sec: u64,
    pub tx_bytes_per_sec: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuLiveStats {
    pub index: u32,
    pub usage_percent: Option<f32>,
    pub temperature_celsius: Option<f32>,
}

// ─── Managed state ────────────────────────────────────────────────────────────

pub struct SystemInfoState {
    pub system: Mutex<System>,
    pub networks: Mutex<Networks>,
    pub prev_net: Mutex<Option<HashMap<String, (u64, u64)>>>,
    pub static_cache: Mutex<Option<StaticSystemInfo>>,
    #[cfg(target_os = "windows")]
    pub nvml: Mutex<Option<nvml_wrapper::Nvml>>,
}

impl Default for SystemInfoState {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemInfoState {
    pub fn new() -> Self {
        let system = System::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything()),
        );
        let networks = Networks::new_with_refreshed_list();
        #[cfg(target_os = "windows")]
        let nvml = Mutex::new(nvml_wrapper::Nvml::init().ok());
        Self {
            system: Mutex::new(system),
            networks: Mutex::new(networks),
            prev_net: Mutex::new(None),
            static_cache: Mutex::new(None),
            #[cfg(target_os = "windows")]
            nvml,
        }
    }
}

// ─── Registry keys ────────────────────────────────────────────────────────────

const WIN_VERSION_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SOFTWARE\Microsoft\Windows NT\CurrentVersion",
};

// ─── Static info gathering ────────────────────────────────────────────────────

fn gather_windows_info() -> Result<WindowsInfo, AppError> {
    let product_name = WIN_VERSION_KEY
        .get_string("ProductName")
        .unwrap_or_else(|_| "Windows".to_string());
    let display_version = WIN_VERSION_KEY
        .get_string("DisplayVersion")
        .unwrap_or_else(|_| "Unknown".to_string());
    let build_str = WIN_VERSION_KEY.get_string("CurrentBuild")?;
    let build = build_str
        .parse::<u32>()
        .map_err(|_| AppError::message(format!("invalid build: {build_str}")))?;
    let ubr = WIN_VERSION_KEY.get_dword("UBR").unwrap_or(0);

    // ProductName says "Windows 10 Pro" even on Windows 11 on many machines.
    // Build >= 22000 unambiguously identifies Windows 11.
    let product_name = if build >= 22000 {
        product_name.replace("Windows 10", "Windows 11")
    } else {
        product_name
    };

    let hostname = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "Unknown".to_string());
    let username = std::env::var("USERNAME").unwrap_or_else(|_| "Unknown".to_string());
    let architecture = std::env::consts::ARCH.to_string();

    Ok(WindowsInfo {
        product_name,
        display_version,
        build,
        ubr,
        hostname,
        username,
        architecture,
        activation_status: "unknown".to_string(), // filled in by gather_static_info after WMI
    })
}

fn gather_cpu_info(system: &System) -> CpuInfo {
    let cpus = system.cpus();
    let model = cpus
        .first()
        .map(|c| c.brand().to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    let logical_cores = cpus.len() as u32;
    let physical_cores = system
        .physical_core_count()
        .unwrap_or(logical_cores as usize) as u32;
    let base_freq_mhz = cpus.first().map(|c| c.frequency()).unwrap_or(0);

    CpuInfo {
        model,
        physical_cores,
        logical_cores,
        base_freq_mhz,
    }
}

fn gather_disks() -> Vec<DiskInfo> {
    let disks = Disks::new_with_refreshed_list();
    let mut result: Vec<DiskInfo> = disks
        .iter()
        .filter(|d| d.total_space() > 0)
        .map(|d| DiskInfo {
            name: d.name().to_string_lossy().into_owned(),
            mount_point: d.mount_point().to_string_lossy().into_owned(),
            total_bytes: d.total_space(),
            available_bytes: d.available_space(),
            kind: match d.kind() {
                DiskKind::SSD => "SSD".to_string(),
                DiskKind::HDD => "HDD".to_string(),
                DiskKind::Unknown(_) => "unknown".to_string(),
            },
            file_system: d.file_system().to_string_lossy().into_owned(),
        })
        .collect();
    result.sort_by(|a, b| a.mount_point.cmp(&b.mount_point));
    result
}

#[cfg(target_os = "windows")]
fn gather_wmi_hardware() -> (Vec<GpuInfo>, MotherboardInfo, RamInfo, String) {
    use wmi::{COMLibrary, WMIConnection};

    let default_mb = MotherboardInfo {
        manufacturer: "unknown".to_string(),
        product: "unknown".to_string(),
        bios_vendor: "unknown".to_string(),
        bios_version: "unknown".to_string(),
    };
    let default_ram = RamInfo {
        total_bytes: 0,
        speed_mhz: None,
        used_slots: 0,
        total_slots: 0,
    };

    // Spawn a fresh thread so COMLibrary::new() (COINIT_MULTITHREADED) succeeds —
    // Tauri's WebView2 threads already have COM initialized in STA mode, which causes
    // RPC_E_CHANGED_MODE if we call CoInitializeEx from the command handler thread.
    let fallback_mb = default_mb.clone();
    let fallback_ram = default_ram.clone();
    std::thread::spawn(
        move || -> (Vec<GpuInfo>, MotherboardInfo, RamInfo, String) {
            let Ok(com) = COMLibrary::new() else {
                return (vec![], default_mb, default_ram, "Unknown".to_string());
            };
            let Ok(wmi_con) = WMIConnection::new(com) else {
                return (vec![], default_mb, default_ram, "Unknown".to_string());
            };

            // GPUs
            let gpus: Vec<GpuInfo> = wmi_con
                .raw_query::<HashMap<String, wmi::Variant>>(
                    "SELECT Name, AdapterCompatibility, AdapterRAM FROM Win32_VideoController",
                )
                .unwrap_or_default()
                .into_iter()
                .map(|mut row| {
                    let model = wmi_str(&mut row, "Name");
                    let vendor = wmi_str(&mut row, "AdapterCompatibility");
                    let vram_bytes = match row.remove("AdapterRAM") {
                        Some(wmi::Variant::UI4(v)) => Some(v as u64),
                        Some(wmi::Variant::I4(v)) => Some(v as u64),
                        _ => None,
                    };
                    GpuInfo {
                        model,
                        vendor,
                        vram_bytes,
                    }
                })
                .collect();

            // Motherboard
            let mut manufacturer = "Unknown".to_string();
            let mut product = "Unknown".to_string();
            if let Ok(rows) = wmi_con.raw_query::<HashMap<String, wmi::Variant>>(
                "SELECT Manufacturer, Product FROM Win32_BaseBoard",
            ) && let Some(mut row) = rows.into_iter().next()
            {
                manufacturer = wmi_str(&mut row, "Manufacturer");
                product = wmi_str(&mut row, "Product");
            }

            // BIOS
            let mut bios_vendor = "Unknown".to_string();
            let mut bios_version = "Unknown".to_string();
            if let Ok(rows) = wmi_con.raw_query::<HashMap<String, wmi::Variant>>(
                "SELECT Manufacturer, SMBIOSBIOSVersion FROM Win32_BIOS",
            ) && let Some(mut row) = rows.into_iter().next()
            {
                bios_vendor = wmi_str(&mut row, "Manufacturer");
                bios_version = wmi_str(&mut row, "SMBIOSBIOSVersion");
            }

            // RAM sticks — speed + used slot count
            let sticks: Vec<HashMap<String, wmi::Variant>> = wmi_con
                .raw_query("SELECT Speed, Capacity FROM Win32_PhysicalMemory")
                .unwrap_or_default();
            let used_slots = sticks.len() as u32;
            let speed_mhz = sticks.iter().find_map(|row| match row.get("Speed") {
                Some(wmi::Variant::UI4(v)) if *v > 0 => Some(*v),
                _ => None,
            });

            // Total RAM slots from array descriptor
            let mut total_slots: u32 = used_slots; // fallback: at least as many as installed
            if let Ok(rows) = wmi_con.raw_query::<HashMap<String, wmi::Variant>>(
                "SELECT MemoryDevices FROM Win32_PhysicalMemoryArray",
            ) && let Some(row) = rows.into_iter().next()
            {
                match row.get("MemoryDevices") {
                    Some(wmi::Variant::UI2(v)) => total_slots = *v as u32,
                    Some(wmi::Variant::UI4(v)) => total_slots = *v,
                    _ => {}
                }
            }

            // total_bytes: sum capacities from sticks (more accurate than sysinfo on some systems)
            let total_bytes: u64 = sticks
                .iter()
                .filter_map(|row| match row.get("Capacity") {
                    Some(wmi::Variant::UI8(v)) => Some(*v),
                    Some(wmi::Variant::UI4(v)) => Some(*v as u64),
                    _ => None,
                })
                .sum();

            // Windows activation status via SoftwareLicensingProduct
            let activation_status = {
                let rows = wmi_con
                    .raw_query::<HashMap<String, wmi::Variant>>(
                        "SELECT LicenseStatus FROM SoftwareLicensingProduct \
                     WHERE PartialProductKey IS NOT NULL \
                     AND ApplicationId='55c92734-d682-4d71-983e-d6ec3f16059f'",
                    )
                    .unwrap_or_default();
                let code = rows
                    .into_iter()
                    .next()
                    .and_then(|mut row| match row.remove("LicenseStatus") {
                        Some(wmi::Variant::UI4(v)) => Some(v),
                        _ => None,
                    })
                    .unwrap_or(0);
                match code {
                    1 => "activated".to_string(),
                    2 | 3 | 6 => "grace_period".to_string(),
                    _ => "not_activated".to_string(),
                }
            };

            (
                gpus,
                MotherboardInfo {
                    manufacturer,
                    product,
                    bios_vendor,
                    bios_version,
                },
                RamInfo {
                    total_bytes,
                    speed_mhz,
                    used_slots,
                    total_slots,
                },
                activation_status,
            )
        },
    )
    .join()
    .unwrap_or_else(|_| (vec![], fallback_mb, fallback_ram, "Unknown".to_string()))
}

#[cfg(not(target_os = "windows"))]
fn gather_wmi_hardware() -> (Vec<GpuInfo>, MotherboardInfo, RamInfo, String) {
    (
        vec![],
        MotherboardInfo {
            manufacturer: "unknown".to_string(),
            product: "unknown".to_string(),
            bios_vendor: "unknown".to_string(),
            bios_version: "unknown".to_string(),
        },
        RamInfo {
            total_bytes: 0,
            speed_mhz: None,
            used_slots: 0,
            total_slots: 0,
        },
        "unknown".to_string(),
    )
}

fn wmi_str(row: &mut HashMap<String, wmi::Variant>, key: &str) -> String {
    match row.remove(key) {
        Some(wmi::Variant::String(s)) => s.trim().to_string(),
        _ => "unknown".to_string(),
    }
}

pub fn gather_static_info(system: &System) -> Result<StaticSystemInfo, AppError> {
    let mut windows = gather_windows_info()?;
    let cpu = gather_cpu_info(system);
    let (gpus, motherboard, mut ram, activation_status) = gather_wmi_hardware();
    windows.activation_status = activation_status;
    let disks = gather_disks();

    // Fall back to sysinfo total if WMI returned nothing (e.g. non-Windows build)
    if ram.total_bytes == 0 {
        ram.total_bytes = system.total_memory();
    }

    Ok(StaticSystemInfo {
        windows,
        cpu,
        ram,
        gpus,
        motherboard,
        disks,
    })
}

// ─── GPU live (NVML, NVIDIA only) ─────────────────────────────────────────────

#[cfg(target_os = "windows")]
fn gather_gpu_live(nvml_lock: &Mutex<Option<nvml_wrapper::Nvml>>) -> Vec<GpuLiveStats> {
    use nvml_wrapper::enum_wrappers::device::TemperatureSensor;
    let guard = nvml_lock.lock().unwrap();
    let Some(nvml) = guard.as_ref() else {
        return vec![];
    };
    let count = nvml.device_count().unwrap_or(0);
    (0..count)
        .filter_map(|i| {
            let dev = nvml.device_by_index(i).ok()?;
            let usage = dev.utilization_rates().ok().map(|u| u.gpu as f32);
            let temp = dev
                .temperature(TemperatureSensor::Gpu)
                .ok()
                .map(|t| t as f32);
            Some(GpuLiveStats {
                index: i,
                usage_percent: usage,
                temperature_celsius: temp,
            })
        })
        .collect()
}

#[cfg(not(target_os = "windows"))]
fn gather_gpu_live() -> Vec<GpuLiveStats> {
    vec![]
}

// ─── Live info gathering ──────────────────────────────────────────────────────

pub fn gather_live_info(
    system: &mut System,
    networks: &mut Networks,
    prev_net: &mut Option<HashMap<String, (u64, u64)>>,
    #[cfg(target_os = "windows")] nvml: &Mutex<Option<nvml_wrapper::Nvml>>,
) -> LiveSystemInfo {
    system.refresh_cpu_usage();
    system.refresh_memory();

    let cpus = system.cpus();
    let cpu_per_core: Vec<f32> = cpus.iter().map(|c| c.cpu_usage()).collect();
    let cpu_usage_percent =
        cpu_per_core.iter().copied().sum::<f32>() / cpu_per_core.len().max(1) as f32;

    let ram_used_bytes = system.used_memory();

    networks.refresh(false);
    let current_net: HashMap<String, (u64, u64)> = networks
        .iter()
        .map(|(name, data)| {
            (
                name.clone(),
                (data.total_received(), data.total_transmitted()),
            )
        })
        .collect();

    // Keep only real adapters (have ever sent/received bytes, not loopback).
    // On the first call prev_net is None — deltas will be 0 B/s, which is correct.
    let network: Vec<NetworkIfaceStats> = current_net
        .iter()
        .filter(|(name, _)| {
            !name.starts_with("Loopback")
                && networks
                    .get(name.as_str())
                    .map(|d| d.total_received() > 0 || d.total_transmitted() > 0)
                    .unwrap_or(false)
        })
        .map(|(name, &(rx, tx))| {
            let (prev_rx, prev_tx) = prev_net
                .as_ref()
                .and_then(|p| p.get(name))
                .copied()
                .unwrap_or((rx, tx));
            NetworkIfaceStats {
                name: name.clone(),
                rx_bytes_per_sec: rx.saturating_sub(prev_rx),
                tx_bytes_per_sec: tx.saturating_sub(prev_tx),
            }
        })
        .collect();

    *prev_net = Some(current_net);

    // GPU live stats (NVIDIA only via NVML)
    #[cfg(target_os = "windows")]
    let gpu_live = gather_gpu_live(nvml);
    #[cfg(not(target_os = "windows"))]
    let gpu_live = gather_gpu_live();

    LiveSystemInfo {
        cpu_usage_percent,
        cpu_per_core,
        ram_used_bytes,
        network,
        gpu_live,
    }
}
