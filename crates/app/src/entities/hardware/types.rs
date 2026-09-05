use std::collections::HashMap;
use std::time::Instant;
use gpui::SharedString;

pub const CPU_HISTORY_SAMPLES: usize = 30;

#[derive(Clone, Debug)]
pub struct CpuInfo {
    pub name: SharedString,
    pub usage_percent: u32,
}

#[derive(Clone, Debug)]
pub struct CpuDetailData {
    pub model: SharedString,
    pub processes: u32,
    pub threads: u32,
    pub handles: u32,
    pub uptime: SharedString,
    pub base_clock_ghz: f32,
    pub current_clock_ghz: f32,
    pub sockets: u32,
    pub cores: u32,
    pub logical_processors: u32,
    pub virtualization: bool,
    pub l1_cache_kb: u32,
    pub l2_cache_mb: u32,
    pub l3_cache_mb: u32,
    pub previous_core_utilization: Vec<f32>,
    pub core_utilization: Vec<f32>,
    pub history_15s: Vec<f32>,
    pub sample_instant: Instant,
}

#[derive(Clone, Debug)]
pub struct RamInfo {
    pub slots: SharedString,
    pub used_gb: f32,
    pub total_gb: f32,
    pub available_gb: f32,
    pub committed_gb: f32,
    pub commit_limit_gb: f32,
    pub cached_mb: f32,
    pub paged_pool_mb: f32,
    pub non_paged_pool_mb: f32,
    pub speed_mhz: u32,
    pub form_factor: SharedString,
    pub hardware_reserved_mb: f32,
    pub history_15s: Vec<f32>,
    pub sample_instant: Instant,
}

#[derive(Clone, Debug)]
pub struct DiskInfo {
    pub id: usize,
    pub letter: SharedString,
    pub custom_name: Option<SharedString>,
    pub used_gb: u64,
    pub total_gb: u64,
    pub file_system: SharedString,
    pub kind: DiskKind,
    pub is_removable: bool,
    pub is_system: bool,
    pub active_percent: f32,
    pub read_mb_s: f32,
    pub write_mb_s: f32,
    pub average_response_ms: f32,
    pub active_history_15s: Vec<f32>,
    pub transfer_history_15s: Vec<f32>,
    pub previous_transfer_scale: f32,
    pub transfer_scale: f32,
    pub sample_instant: Instant,
}

#[must_use]
pub fn transfer_scale(history: &[f32]) -> f32 {
    let peak = history.iter().copied().fold(0.0_f32, f32::max);
    (peak / 10.0).ceil().mul_add(10.0, 0.0).max(50.0)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DiskKind {
    NvmeSsd,
    Ssd,
    Hdd,
    Unknown,
}

#[derive(Clone, Debug)]
pub struct NetworkInfo {
    pub id: u32,
    pub interface_name: SharedString,
    pub adapter_name: SharedString,
    pub kind: NetworkKind,
    pub link_speed_mbps: u64,
    pub rx_speed: SharedString,
    pub tx_speed: SharedString,
    pub rx_history_15s: Vec<f32>,
    pub tx_history_15s: Vec<f32>,
    pub previous_throughput_scale: f32,
    pub throughput_scale: f32,
    pub sample_instant: Instant,
}

#[must_use]
pub fn throughput_scale(rx: &[f32], tx: &[f32]) -> f32 {
    let peak = rx.iter().chain(tx).copied().fold(0.0_f32, f32::max);
    if peak <= 1.0 {
        1.0
    } else {
        (peak * 1.15).log2().ceil().exp2()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NetworkKind {
    Ethernet,
    Wifi,
}

pub const DISCRETE_ENGINES: [&str; 14] = [
    "3D",
    "Copy",
    "Video Encode",
    "Video Decode",
    "Overlay",
    "Copy 1",
    "Security",
    "OFA_0",
    "VR",
    "Copy 2",
    "Copy 3",
    "Copy 4",
    "Copy 5",
    "Security_1",
];

pub const INTEGRATED_ENGINES: [&str; 11] = [
    "3D",
    "Copy",
    "High Priority Compute",
    "High Priority 3D",
    "Compute 0",
    "Compute 1",
    "Timer 0",
    "Security 1",
    "Video JPEG 0",
    "Video Decode 1",
    "Video Codec 0",
];

#[derive(Clone, Debug)]
pub struct GpuInfo {
    pub id: usize,
    pub name: SharedString,
    pub usage_percent: u32,
    pub temperature_c: u32,
    pub is_discrete: bool,
    pub dedicated_used_mb: f32,
    pub dedicated_total_mb: f32,
    pub shared_used_mb: f32,
    pub shared_total_mb: f32,
    pub memory_used_mb: f32,
    pub memory_total_mb: f32,
    pub driver_version: SharedString,
    pub driver_date: SharedString,
    pub directx_version: SharedString,
    pub pci_location: SharedString,
    pub hardware_reserved_mb: u32,
    pub available_engines: Vec<&'static str>,
    pub engine_utilizations: HashMap<&'static str, f32>,
    pub engine_histories_15s: HashMap<&'static str, Vec<f32>>,
    pub dedicated_history_15s: Vec<f32>,
    pub shared_history_15s: Vec<f32>,
    pub sample_instant: Instant,
}

#[derive(Clone, Debug)]
pub struct TelemetryData {
    pub cpu: CpuInfo,
    pub cpu_detail: CpuDetailData,
    pub ram: RamInfo,
    pub disks: Vec<DiskInfo>,
    pub networks: Vec<NetworkInfo>,
    pub gpus: Vec<GpuInfo>,
}