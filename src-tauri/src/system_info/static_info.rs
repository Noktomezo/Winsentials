use sysinfo::System;

use crate::error::AppError;
use crate::system_info::types::{GpuInfo, StaticSystemInfo};
use crate::system_info::windows::disk::gather_disks;
use crate::system_info::windows::gpu::{build_static_gpus, enumerate_dxgi_adapters};
use crate::system_info::windows::hardware::gather_wmi_hardware;
use crate::system_info::windows::network::gather_network_adapters;
use crate::system_info::windows::windows_info::{gather_cpu_info, gather_windows_info};

pub fn gather_static_info(system: &System) -> Result<StaticSystemInfo, AppError> {
    let cpu = gather_cpu_info(system);

    let ((windows_res, disks), (wmi_res, dxgi_adapters)) = rayon::join(
        || (gather_windows_info(), gather_disks()),
        || rayon::join(gather_wmi_hardware, enumerate_dxgi_adapters),
    );

    let mut windows = windows_res?;
    let (motherboard, mut ram, activation_status, cpu_extra) = wmi_res;
    windows.activation_status = activation_status;
    let network_adapters = gather_network_adapters();

    #[cfg(target_os = "windows")]
    let gpus: Vec<GpuInfo> = build_static_gpus(dxgi_adapters);
    #[cfg(not(target_os = "windows"))]
    let gpus: Vec<GpuInfo> = {
        let _ = dxgi_adapters;
        vec![]
    };

    let mut cpu = cpu;
    cpu.sockets = cpu_extra.sockets;
    cpu.virtualization = cpu_extra.virtualization;
    cpu.l1_cache_kb = cpu_extra.l1_cache_kb;
    cpu.l2_cache_kb = cpu_extra.l2_cache_kb;
    cpu.l3_cache_kb = cpu_extra.l3_cache_kb;

    if ram.total_bytes == 0 {
        ram.total_bytes = system.total_memory();
    }

    Ok(StaticSystemInfo {
        windows,
        cpu,
        ram,
        network_adapters,
        gpus,
        motherboard,
        disks,
    })
}
