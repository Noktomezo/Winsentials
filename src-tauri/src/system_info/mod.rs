mod live_info;
mod network_filter;
mod state;
mod static_info;
mod types;
mod windows;

#[cfg(target_os = "windows")]
pub use live_info::PdhHandles;
pub use live_info::gather_live_info;
pub use network_filter::{is_hidden_virtual_adapter_label, is_hidden_virtual_adapter_name};
pub use state::{PreviousNetSnapshot, SystemInfoState};
pub use static_info::gather_static_info;
pub use types::{
    CpuInfo, DeviceInventoryInfo, DiskInfo, DiskLiveInfo, GpuInfo, GpuProcess, LiveCpuInfo,
    LiveGpuMetrics, LiveHomeInfo, LiveRamInfo, LiveSystemInfo, MotherboardInfo, NetworkAdapterInfo,
    NetworkIfaceStats, RamInfo, StaticSystemInfo, WindowsInfo,
};
pub use windows::cpu::pdh_open_cpu_perf_query;
pub use windows::disk::gather_disks;
pub use windows::disk::pdh_open_disk_query;
pub use windows::gpu::pdh_close_gpu_query;
pub use windows::gpu::pdh_open_gpu_query;
pub use windows::network::gather_network_adapters;
