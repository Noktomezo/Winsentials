use gfxinfo::active_gpu;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::OnceLock;
use sysinfo::{CpuRefreshKind, Disks, MemoryRefreshKind, RefreshKind, System};
use wmi::WMIConnection;

use crate::wmi_queries::{
  Win32_PhysicalMemory, Win32_PhysicalMemoryArray, Win32_VideoController,
  get_wmi_connection,
};

static STATIC_INFO: OnceLock<StaticSystemInfo> = OnceLock::new();
static SYSTEM_STATE: RwLock<SystemState> = RwLock::new(SystemState::new());

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsInfo {
  pub name: String,
  pub version: String,
  pub build: String,
  pub arch: String,
  pub display_version: String,
  pub hostname: String,
  pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticCpuInfo {
  pub name: String,
  pub cores: usize,
  pub logical_cores: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicCpuInfo {
  pub usage: f32,
  pub frequency: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticGpuInfo {
  pub name: String,
  pub memory_total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicGpuInfo {
  pub usage: f32,
  pub memory_used: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticRamInfo {
  pub total: u64,
  pub slots_used: usize,
  pub slots_total: usize,
  pub speed: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicRamInfo {
  pub used: u64,
  pub usage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticDiskInfo {
  pub name: String,
  pub mount_point: String,
  pub label: String,
  pub total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicDiskInfo {
  pub mount_point: String,
  pub available: u64,
  pub usage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
  pub name: String,
  pub usage: f32,
  pub frequency: u64,
  pub cores: usize,
  pub logical_cores: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
  pub name: String,
  pub usage: f32,
  pub memory_total: u64,
  pub memory_used: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RamInfo {
  pub total: u64,
  pub used: u64,
  pub usage: f32,
  pub slots_used: usize,
  pub slots_total: usize,
  pub speed: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
  pub name: String,
  pub mount_point: String,
  pub label: String,
  pub total: u64,
  pub available: u64,
  pub usage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticSystemInfo {
  pub os: OsInfo,
  pub cpu: StaticCpuInfo,
  pub gpu: Option<StaticGpuInfo>,
  pub ram: StaticRamInfo,
  pub disks: Vec<StaticDiskInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicSystemInfo {
  pub cpu: DynamicCpuInfo,
  pub gpu: Option<DynamicGpuInfo>,
  pub ram: DynamicRamInfo,
  pub disks: Vec<DynamicDiskInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
  pub os: OsInfo,
  pub cpu: CpuInfo,
  pub gpu: Option<GpuInfo>,
  pub ram: RamInfo,
  pub disks: Vec<DiskInfo>,
}

struct SystemState {
  sys: Option<System>,
  initialized: bool,
}

impl SystemState {
  const fn new() -> Self {
    Self {
      sys: None,
      initialized: false,
    }
  }
}

#[tauri::command]
pub fn get_static_system_info() -> StaticSystemInfo {
  STATIC_INFO
    .get_or_init(|| {
      let wmi = get_wmi_connection();

      StaticSystemInfo {
        os: get_os_info(),
        cpu: get_static_cpu_info(),
        gpu: get_static_gpu_info(wmi.as_ref()),
        ram: get_static_ram_info(wmi.as_ref()),
        disks: get_static_disk_info(),
      }
    })
    .clone()
}

#[tauri::command]
pub fn get_dynamic_system_info() -> DynamicSystemInfo {
  let mut state = SYSTEM_STATE.write();

  if !state.initialized {
    state.sys = Some(System::new_with_specifics(
      RefreshKind::nothing()
        .with_cpu(CpuRefreshKind::everything())
        .with_memory(MemoryRefreshKind::everything()),
    ));
    state.initialized = true;
  }

  if let Some(ref mut sys) = state.sys {
    sys.refresh_cpu_all();
    sys.refresh_memory_specifics(MemoryRefreshKind::everything());
  }

  let disks = Disks::new_with_refreshed_list();

  DynamicSystemInfo {
    cpu: get_dynamic_cpu_info(&state.sys),
    gpu: get_dynamic_gpu_info(),
    ram: get_dynamic_ram_info(&state.sys),
    disks: get_dynamic_disk_info(&disks),
  }
}

#[tauri::command]
pub fn get_system_info() -> SystemInfo {
  let static_info = get_static_system_info();
  let dynamic_info = get_dynamic_system_info();

  SystemInfo {
    os: static_info.os,
    cpu: CpuInfo {
      name: static_info.cpu.name,
      usage: dynamic_info.cpu.usage,
      frequency: dynamic_info.cpu.frequency,
      cores: static_info.cpu.cores,
      logical_cores: static_info.cpu.logical_cores,
    },
    gpu: static_info.gpu.zip(dynamic_info.gpu).map(|(s, d)| GpuInfo {
      name: s.name,
      usage: d.usage,
      memory_total: s.memory_total,
      memory_used: d.memory_used,
    }),
    ram: RamInfo {
      total: static_info.ram.total,
      used: dynamic_info.ram.used,
      usage: dynamic_info.ram.usage,
      slots_used: static_info.ram.slots_used,
      slots_total: static_info.ram.slots_total,
      speed: static_info.ram.speed,
    },
    disks: static_info
      .disks
      .into_iter()
      .zip(dynamic_info.disks)
      .map(|(s, d)| DiskInfo {
        name: s.name,
        mount_point: s.mount_point,
        label: s.label,
        total: s.total,
        available: d.available,
        usage: d.usage,
      })
      .collect(),
  }
}

fn get_os_info() -> OsInfo {
  let name = System::name().unwrap_or_else(|| "Unknown".to_string());
  let version = System::os_version().unwrap_or_else(|| "Unknown".to_string());
  let build = System::kernel_version().unwrap_or_else(|| "Unknown".to_string());
  let arch = System::cpu_arch();
  let hostname = System::host_name().unwrap_or_else(|| "Unknown".to_string());
  let username = env::var("USERNAME").unwrap_or_else(|_| "Unknown".to_string());
  let display_version = get_display_version().unwrap_or_default();

  OsInfo {
    name,
    version,
    build,
    arch,
    display_version,
    hostname,
    username,
  }
}

fn get_display_version() -> Option<String> {
  use winreg::RegKey;
  use winreg::enums::*;

  let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
  let key = hklm
    .open_subkey(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion")
    .ok()?;
  key.get_value("DisplayVersion").ok()
}

fn get_static_cpu_info() -> StaticCpuInfo {
  let sys = System::new_with_specifics(
    RefreshKind::nothing().with_cpu(CpuRefreshKind::nothing().with_frequency()),
  );

  let cpu = sys.cpus().first();
  let name = cpu
    .map(|c| c.brand().to_string())
    .unwrap_or_else(|| "Unknown".to_string());
  let cores = System::physical_core_count().unwrap_or(0);
  let logical_cores = sys.cpus().len();

  StaticCpuInfo {
    name,
    cores,
    logical_cores,
  }
}

fn get_dynamic_cpu_info(sys: &Option<System>) -> DynamicCpuInfo {
  let (usage, frequency) = sys
    .as_ref()
    .map(|s| {
      let usage = s.global_cpu_usage();
      let frequency = s.cpus().first().map(|c| c.frequency()).unwrap_or(0);
      (usage, frequency)
    })
    .unwrap_or((0.0, 0));

  DynamicCpuInfo { usage, frequency }
}

fn get_static_gpu_info(wmi: Option<&WMIConnection>) -> Option<StaticGpuInfo> {
  if let Ok(gpu) = active_gpu() {
    let info = gpu.info();
    return Some(StaticGpuInfo {
      name: gpu.model().to_string(),
      memory_total: info.total_vram(),
    });
  }

  wmi.and_then(|conn| {
    let results: Vec<Win32_VideoController> = conn.query().ok()?;
    results.into_iter().next().map(|gpu| StaticGpuInfo {
      name: gpu.Name,
      memory_total: gpu.AdapterRAM.unwrap_or(0),
    })
  })
}

fn get_dynamic_gpu_info() -> Option<DynamicGpuInfo> {
  if let Ok(gpu) = active_gpu() {
    let info = gpu.info();
    return Some(DynamicGpuInfo {
      usage: info.load_pct() as f32,
      memory_used: info.used_vram(),
    });
  }
  None
}

fn get_static_ram_info(wmi: Option<&WMIConnection>) -> StaticRamInfo {
  let sys = System::new_all();
  let total = sys.total_memory();

  let (slots_used, slots_total, speed) = wmi
    .map(|conn| {
      let memory_modules: Vec<Win32_PhysicalMemory> =
        conn.query().unwrap_or_default();
      let memory_arrays: Vec<Win32_PhysicalMemoryArray> =
        conn.query().unwrap_or_default();

      let slots_used = memory_modules.len();
      let slots_total = memory_arrays
        .first()
        .map(|arr| arr.MemoryDevices as usize)
        .unwrap_or(slots_used);

      let speed = memory_modules
        .first()
        .and_then(|m| m.ConfiguredClockSpeed)
        .unwrap_or(0);

      (slots_used, slots_total, speed)
    })
    .unwrap_or((0, 0, 0));

  StaticRamInfo {
    total,
    slots_used,
    slots_total,
    speed,
  }
}

fn get_dynamic_ram_info(sys: &Option<System>) -> DynamicRamInfo {
  let (used, usage) = sys
    .as_ref()
    .map(|s| {
      let total = s.total_memory();
      let used = s.used_memory();
      let usage = if total > 0 {
        (used as f64 / total as f64 * 100.0) as f32
      } else {
        0.0
      };
      (used, usage)
    })
    .unwrap_or((0, 0.0));

  DynamicRamInfo { used, usage }
}

fn get_static_disk_info() -> Vec<StaticDiskInfo> {
  let disks = Disks::new_with_refreshed_list();

  let mut disk_list: Vec<StaticDiskInfo> = disks
    .iter()
    .filter(|d| {
      let path = d.mount_point();
      path
        .to_str()
        .map(|s| {
          s.len() == 3 && s.chars().nth(1) == Some(':') && s.ends_with('\\')
        })
        .unwrap_or(false)
    })
    .map(|disk| {
      let total = disk.total_space();
      let mount_point =
        disk.mount_point().to_str().unwrap_or("Unknown").to_string();
      let mount_point_clean = mount_point.trim_end_matches('\\').to_string();

      StaticDiskInfo {
        name: mount_point_clean.clone(),
        mount_point: mount_point_clean,
        label: disk.name().to_string_lossy().to_string(),
        total,
      }
    })
    .collect();

  disk_list.reverse();
  disk_list
}

fn get_dynamic_disk_info(disks: &Disks) -> Vec<DynamicDiskInfo> {
  disks
    .iter()
    .filter(|d| {
      let path = d.mount_point();
      path
        .to_str()
        .map(|s| {
          s.len() == 3 && s.chars().nth(1) == Some(':') && s.ends_with('\\')
        })
        .unwrap_or(false)
    })
    .map(|disk| {
      let total = disk.total_space();
      let available = disk.available_space();
      let usage = if total > 0 {
        ((total - available) as f64 / total as f64 * 100.0) as f32
      } else {
        0.0
      };

      let mount_point =
        disk.mount_point().to_str().unwrap_or("Unknown").to_string();
      let mount_point_clean = mount_point.trim_end_matches('\\').to_string();

      DynamicDiskInfo {
        mount_point: mount_point_clean,
        available,
        usage,
      }
    })
    .collect()
}
