use sysinfo::System;

use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::system_info::types::{CpuInfo, WindowsInfo};

const WIN_VERSION_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SOFTWARE\Microsoft\Windows NT\CurrentVersion",
};

#[cfg(target_os = "windows")]
pub fn gather_windows_info() -> Result<WindowsInfo, AppError> {
    let hostname = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "Unknown".to_string());
    let username = std::env::var("USERNAME").unwrap_or_else(|_| "Unknown".to_string());
    let architecture = std::env::consts::ARCH.to_string();

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

    let product_name = if build >= 22000 {
        product_name.replace("Windows 10", "Windows 11")
    } else {
        product_name
    };

    Ok(WindowsInfo {
        product_name,
        display_version,
        build,
        ubr,
        hostname,
        username,
        architecture,
        activation_status: "unknown".to_string(),
    })
}

#[cfg(not(target_os = "windows"))]
pub fn gather_windows_info() -> Result<WindowsInfo, AppError> {
    Ok(WindowsInfo {
        product_name: "Windows".to_string(),
        display_version: "Unknown".to_string(),
        build: 0,
        ubr: 0,
        hostname: std::env::var("HOSTNAME").unwrap_or_else(|_| "Unknown".to_string()),
        username: std::env::var("USER").unwrap_or_else(|_| "Unknown".to_string()),
        architecture: std::env::consts::ARCH.to_string(),
        activation_status: "unknown".to_string(),
    })
}

pub fn gather_cpu_info(system: &System) -> CpuInfo {
    let cpus = system.cpus();
    let model = cpus
        .first()
        .map(|c| c.brand().to_string())
        .unwrap_or_else(|| "Unknown".to_string());
    let logical_cores = cpus.len() as u32;
    let physical_cores = System::physical_core_count().unwrap_or(logical_cores as usize) as u32;
    let base_freq_mhz = cpus.first().map(|c| c.frequency()).unwrap_or(0);

    CpuInfo {
        model,
        physical_cores,
        logical_cores,
        base_freq_mhz,
        sockets: 1,
        virtualization: false,
        l1_cache_kb: None,
        l2_cache_kb: None,
        l3_cache_kb: None,
    }
}
