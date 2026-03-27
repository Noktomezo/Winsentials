use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticSystemInfo {
    pub windows: WindowsInfo,
    pub cpu: CpuInfo,
    pub ram: RamInfo,
    pub network_adapters: Vec<NetworkAdapterInfo>,
    pub gpus: Vec<GpuInfo>,
    pub motherboard: MotherboardInfo,
    pub disks: Vec<DiskInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInventoryInfo {
    pub network_adapters: Vec<NetworkAdapterInfo>,
    pub disks: Vec<DiskInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RamInfo {
    pub total_bytes: u64,
    pub speed_mhz: Option<u32>,
    pub used_slots: u32,
    pub total_slots: u32,
    pub form_factor: Option<String>,
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
    pub sockets: u32,
    pub virtualization: bool,
    pub l1_cache_kb: Option<u32>,
    pub l2_cache_kb: Option<u32>,
    pub l3_cache_kb: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GpuProcess {
    pub pid: u32,
    pub name: String,
    pub dedicated_mem_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GpuInfo {
    pub index: usize,
    pub name: String,
    pub vendor: String,
    pub is_integrated: bool,
    pub driver_version: Option<String>,
    pub driver_date: Option<String>,
    pub directx_version: Option<String>,
    pub vram_total_mb: u64,
    pub vram_used_mb: u64,
    pub vram_shared_mb: u64,
    pub vram_reserved_mb: u64,
    pub temperature_c: Option<u32>,
    pub power_w: Option<u32>,
    pub util_3d: u32,
    pub util_copy: u32,
    pub util_encode: u32,
    pub util_decode: u32,
    pub util_high_priority_3d: u32,
    pub util_high_priority_compute: u32,
    pub processes: Vec<GpuProcess>,
    pub pci_bus: Option<u32>,
    pub pci_device: Option<u32>,
    pub pci_function: Option<u32>,
    #[serde(skip)]
    pub luid_low: u32,
    #[serde(skip)]
    pub luid_high: i32,
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
    pub model: Option<String>,
    pub mount_point: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub kind: String,
    pub file_system: String,
    pub volume_label: Option<String>,
    pub is_system_disk: bool,
    pub has_pagefile: bool,
    pub type_label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveSystemInfo {
    pub cpu_usage_percent: f32,
    pub cpu_per_core: Vec<f32>,
    pub cpu_current_freq_mhz: u64,
    pub cpu_process_count: u32,
    pub cpu_thread_count: u32,
    pub cpu_handle_count: u32,
    pub cpu_uptime_secs: u64,
    pub ram_used_bytes: u64,
    pub ram_available_bytes: u64,
    pub ram_committed_bytes: u64,
    pub ram_commit_limit_bytes: u64,
    pub ram_cached_bytes: u64,
    pub ram_compressed_bytes: u64,
    pub ram_paged_pool_bytes: u64,
    pub ram_nonpaged_pool_bytes: u64,
    pub disks: Vec<DiskLiveInfo>,
    pub network: Vec<NetworkIfaceStats>,
    pub gpus: Vec<LiveGpuMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveCpuInfo {
    pub cpu_usage_percent: f32,
    pub cpu_per_core: Vec<f32>,
    pub cpu_current_freq_mhz: u64,
    pub cpu_process_count: u32,
    pub cpu_thread_count: u32,
    pub cpu_handle_count: u32,
    pub cpu_uptime_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveRamInfo {
    pub ram_used_bytes: u64,
    pub ram_available_bytes: u64,
    pub ram_committed_bytes: u64,
    pub ram_commit_limit_bytes: u64,
    pub ram_cached_bytes: u64,
    pub ram_compressed_bytes: u64,
    pub ram_paged_pool_bytes: u64,
    pub ram_nonpaged_pool_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveGpuMetrics {
    pub index: usize,
    pub vram_total_mb: u64,
    pub vram_used_mb: u64,
    pub vram_shared_mb: u64,
    pub vram_reserved_mb: u64,
    pub temperature_c: Option<u32>,
    pub power_w: Option<u32>,
    pub util_3d: u32,
    pub util_copy: u32,
    pub util_encode: u32,
    pub util_decode: u32,
    pub util_high_priority_3d: u32,
    pub util_high_priority_compute: u32,
    pub processes: Vec<GpuProcess>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveHomeInfo {
    pub cpu_usage_percent: f32,
    pub ram_used_bytes: u64,
    pub network: Vec<NetworkIfaceStats>,
    pub gpus: Vec<LiveGpuMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskLiveInfo {
    pub mount_point: String,
    pub active_time_percent: u32,
    pub avg_response_ms: f64,
    pub read_bytes_per_sec: u64,
    pub write_bytes_per_sec: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkIfaceStats {
    pub name: String,
    pub rx_bytes_per_sec: u64,
    pub tx_bytes_per_sec: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkAdapterInfo {
    pub index: usize,
    pub name: String,
    pub adapter_description: String,
    pub dns_name: Option<String>,
    pub connection_type: String,
    pub ipv4_addresses: Vec<String>,
    pub ipv6_addresses: Vec<String>,
    pub is_wifi: bool,
    pub ssid: Option<String>,
    pub signal_percent: Option<u32>,
}

impl LiveSystemInfo {
    pub fn to_live_cpu_info(&self) -> LiveCpuInfo {
        LiveCpuInfo {
            cpu_usage_percent: self.cpu_usage_percent,
            cpu_per_core: self.cpu_per_core.clone(),
            cpu_current_freq_mhz: self.cpu_current_freq_mhz,
            cpu_process_count: self.cpu_process_count,
            cpu_thread_count: self.cpu_thread_count,
            cpu_handle_count: self.cpu_handle_count,
            cpu_uptime_secs: self.cpu_uptime_secs,
        }
    }

    pub fn to_live_ram_info(&self) -> LiveRamInfo {
        LiveRamInfo {
            ram_used_bytes: self.ram_used_bytes,
            ram_available_bytes: self.ram_available_bytes,
            ram_committed_bytes: self.ram_committed_bytes,
            ram_commit_limit_bytes: self.ram_commit_limit_bytes,
            ram_cached_bytes: self.ram_cached_bytes,
            ram_compressed_bytes: self.ram_compressed_bytes,
            ram_paged_pool_bytes: self.ram_paged_pool_bytes,
            ram_nonpaged_pool_bytes: self.ram_nonpaged_pool_bytes,
        }
    }

    pub fn to_live_gpu_info(&self) -> Vec<LiveGpuMetrics> {
        self.gpus.clone()
    }

    pub fn to_live_home_info(&self) -> LiveHomeInfo {
        LiveHomeInfo {
            cpu_usage_percent: self.cpu_usage_percent,
            ram_used_bytes: self.ram_used_bytes,
            network: self.network.clone(),
            gpus: self.to_live_gpu_info(),
        }
    }
}

impl From<&GpuInfo> for LiveGpuMetrics {
    fn from(value: &GpuInfo) -> Self {
        Self {
            index: value.index,
            vram_total_mb: value.vram_total_mb,
            vram_used_mb: value.vram_used_mb,
            vram_shared_mb: value.vram_shared_mb,
            vram_reserved_mb: value.vram_reserved_mb,
            temperature_c: value.temperature_c,
            power_w: value.power_w,
            util_3d: value.util_3d,
            util_copy: value.util_copy,
            util_encode: value.util_encode,
            util_decode: value.util_decode,
            util_high_priority_3d: value.util_high_priority_3d,
            util_high_priority_compute: value.util_high_priority_compute,
            processes: value.processes.clone(),
        }
    }
}
