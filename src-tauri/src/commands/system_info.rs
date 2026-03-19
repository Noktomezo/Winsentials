use crate::error::AppError;
use crate::system_info::{
    DiskLiveInfo, LiveCpuInfo, LiveGpuMetrics, LiveHomeInfo, LiveRamInfo, LiveSystemInfo,
    NetworkIfaceStats, StaticSystemInfo, SystemInfoState, gather_static_info,
};
use tauri::State;

#[tauri::command]
pub fn get_static_system_info(state: State<SystemInfoState>) -> Result<StaticSystemInfo, AppError> {
    let mut cache = state
        .static_cache
        .lock()
        .map_err(|_| AppError::message("static_cache lock poisoned"))?;
    if let Some(info) = cache.as_ref() {
        return Ok(info.clone());
    }
    let system = state
        .system
        .lock()
        .map_err(|_| AppError::message("system lock poisoned"))?;
    let info = gather_static_info(&system)?;
    *cache = Some(info.clone());
    Ok(info)
}

#[tauri::command]
pub fn get_live_system_info(state: State<SystemInfoState>) -> Result<LiveSystemInfo, AppError> {
    get_live_snapshot(&state)
}

fn get_live_snapshot(state: &State<SystemInfoState>) -> Result<LiveSystemInfo, AppError> {
    state
        .live_cache
        .lock()
        .map_err(|_| AppError::message("live_cache lock poisoned"))?
        .clone()
        .ok_or_else(|| AppError::message("live info not yet available"))
}

#[tauri::command]
pub fn get_live_home_info(state: State<SystemInfoState>) -> Result<LiveHomeInfo, AppError> {
    Ok(get_live_snapshot(&state)?.to_live_home_info())
}

#[tauri::command]
pub fn get_live_cpu_info(state: State<SystemInfoState>) -> Result<LiveCpuInfo, AppError> {
    Ok(get_live_snapshot(&state)?.to_live_cpu_info())
}

#[tauri::command]
pub fn get_live_ram_info(state: State<SystemInfoState>) -> Result<LiveRamInfo, AppError> {
    Ok(get_live_snapshot(&state)?.to_live_ram_info())
}

#[tauri::command]
pub fn get_live_disk_info(state: State<SystemInfoState>) -> Result<Vec<DiskLiveInfo>, AppError> {
    Ok(get_live_snapshot(&state)?.disks)
}

#[tauri::command]
pub fn get_live_network_info(
    state: State<SystemInfoState>,
) -> Result<Vec<NetworkIfaceStats>, AppError> {
    Ok(get_live_snapshot(&state)?.network)
}

#[tauri::command]
pub fn get_live_gpu_info(state: State<SystemInfoState>) -> Result<Vec<LiveGpuMetrics>, AppError> {
    Ok(get_live_snapshot(&state)?.to_live_gpu_info())
}
