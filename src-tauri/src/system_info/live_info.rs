use std::collections::HashMap;
use std::time::Instant;

use sysinfo::{CpuRefreshKind, Networks, System};

use crate::system_info::state::PreviousNetSnapshot;
use crate::system_info::types::{DiskLiveInfo, GpuInfo, LiveSystemInfo, NetworkIfaceStats};
use crate::system_info::windows::cpu::{get_perf_info, pdh_collect_cpu_perf_pct};
use crate::system_info::windows::disk::gather_disk_live;
use crate::system_info::windows::gpu::gather_gpu_live;
use crate::system_info::windows::ram::get_ram_perf;

#[allow(clippy::too_many_arguments)]
pub fn gather_live_info(
    system: &mut System,
    networks: &mut Networks,
    prev_net: &mut Option<PreviousNetSnapshot>,
    gpus: &[GpuInfo],
    #[cfg(target_os = "windows")] pdh_query: isize,
    #[cfg(target_os = "windows")] pdh_counter: isize,
    #[cfg(target_os = "windows")] disk_pdh_query: isize,
    #[cfg(target_os = "windows")] disk_active_counter: isize,
    #[cfg(target_os = "windows")] disk_response_counter: isize,
    #[cfg(target_os = "windows")] disk_read_counter: isize,
    #[cfg(target_os = "windows")] disk_write_counter: isize,
    #[cfg(target_os = "windows")] cpu_pdh_query: isize,
    #[cfg(target_os = "windows")] cpu_pdh_counter: isize,
    #[cfg(target_os = "windows")] base_freq_mhz: u64,
) -> LiveSystemInfo {
    system.refresh_cpu_specifics(CpuRefreshKind::everything());
    system.refresh_memory();

    let cpus = system.cpus();
    let cpu_per_core: Vec<f32> = cpus.iter().map(|c| c.cpu_usage()).collect();
    let cpu_usage_percent =
        cpu_per_core.iter().copied().sum::<f32>() / cpu_per_core.len().max(1) as f32;
    #[cfg(target_os = "windows")]
    let cpu_current_freq_mhz = pdh_collect_cpu_perf_pct(cpu_pdh_query, cpu_pdh_counter)
        .map(|pct| (base_freq_mhz as f64 * pct / 100.0).round() as u64)
        .or_else(|| cpus.first().map(|c| c.frequency()))
        .unwrap_or(0);
    #[cfg(not(target_os = "windows"))]
    let cpu_current_freq_mhz = cpus.first().map(|c| c.frequency()).unwrap_or(0);
    let cpu_uptime_secs = System::uptime();
    let (cpu_process_count, cpu_thread_count, cpu_handle_count) = get_perf_info();

    let ram_used_bytes = system.used_memory();
    let ram_available_bytes = system.available_memory();
    let (
        ram_committed_bytes,
        ram_commit_limit_bytes,
        ram_cached_bytes,
        ram_compressed_bytes,
        ram_paged_pool_bytes,
        ram_nonpaged_pool_bytes,
    ) = get_ram_perf();

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
    let now = Instant::now();

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
            let previous = prev_net
                .as_ref()
                .and_then(|snapshot| snapshot.totals.get(name).copied());
            let elapsed_secs = prev_net
                .as_ref()
                .map(|snapshot| now.duration_since(snapshot.captured_at).as_secs_f64())
                .filter(|elapsed| *elapsed > 0.0)
                .unwrap_or(1.0);
            let (prev_rx, prev_tx) = previous.unwrap_or((rx, tx));
            NetworkIfaceStats {
                name: name.clone(),
                rx_bytes_per_sec: (rx.saturating_sub(prev_rx) as f64 / elapsed_secs).round() as u64,
                tx_bytes_per_sec: (tx.saturating_sub(prev_tx) as f64 / elapsed_secs).round() as u64,
            }
        })
        .collect();

    #[cfg(target_os = "windows")]
    let disks: Vec<DiskLiveInfo> = gather_disk_live(
        disk_pdh_query,
        disk_active_counter,
        disk_response_counter,
        disk_read_counter,
        disk_write_counter,
    )
    .into_values()
    .collect();
    #[cfg(not(target_os = "windows"))]
    let disks: Vec<DiskLiveInfo> = gather_disk_live(0, 0, 0, 0, 0).into_values().collect();

    *prev_net = Some(PreviousNetSnapshot {
        captured_at: now,
        totals: current_net,
    });

    #[cfg(target_os = "windows")]
    let gpus_live = gather_gpu_live(gpus, pdh_query, pdh_counter);
    #[cfg(not(target_os = "windows"))]
    let gpus_live = gather_gpu_live(gpus, 0, 0);

    LiveSystemInfo {
        cpu_usage_percent,
        cpu_per_core,
        cpu_current_freq_mhz,
        cpu_process_count,
        cpu_thread_count,
        cpu_handle_count,
        cpu_uptime_secs,
        ram_used_bytes,
        ram_available_bytes,
        ram_committed_bytes,
        ram_commit_limit_bytes,
        ram_cached_bytes,
        ram_compressed_bytes,
        ram_paged_pool_bytes,
        ram_nonpaged_pool_bytes,
        disks,
        network,
        gpus: gpus_live,
    }
}
