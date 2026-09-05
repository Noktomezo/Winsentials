use windows_sys::Win32::NetworkManagement::IpHelper::IF_TYPE_ETHERNET_CSMACD;
use windows_sys::Win32::NetworkManagement::Ndis::{
    MediaConnectStateConnected, NET_IF_OPER_STATUS_UP,
};
use windows_sys::Win32::Storage::FileSystem::BusTypeNvme;

use super::cpu_ram::clean_cpu_name;
use super::disk::classify_disk_kind;
use super::gpu::clean_gpu_name;
use super::network::is_visible_network_interface;
use super::types::{CPU_HISTORY_SAMPLES, DiskKind, TelemetryData};

#[test]
fn test_clean_cpu_name() {
    assert_eq!(
        clean_cpu_name("AMD Ryzen 7 7800X3D 8-Core Processor"),
        "AMD Ryzen 7 7800X3D"
    );
    assert_eq!(
        clean_cpu_name("13th Gen Intel(R) Core(TM) i7-13700K"),
        "13th Gen Intel Core i7-13700K"
    );
}

#[test]
fn test_clean_gpu_name() {
    assert_eq!(
        clean_gpu_name("NVIDIA GeForce RTX 4080"),
        "NVIDIA GeForce RTX 4080"
    );
    assert_eq!(
        clean_gpu_name("AMD Radeon(TM) Graphics"),
        "AMD Radeon Graphics"
    );
}

#[test]
fn network_filter_keeps_hardware_and_rejects_ndis_filters() {
    let visible = |flags| {
        is_visible_network_interface(
            IF_TYPE_ETHERNET_CSMACD,
            NET_IF_OPER_STATUS_UP,
            MediaConnectStateConnected,
            6,
            flags,
        )
    };

    assert!(visible(0b0000_0101));
    assert!(!visible(0b0000_0010));
    assert!(!visible(0));
}

#[test]
fn disk_kind_uses_bus_and_seek_penalty() {
    use windows_sys::Win32::Storage::FileSystem::BusTypeSata;

    assert_eq!(
        classify_disk_kind(Some(BusTypeNvme), None),
        DiskKind::NvmeSsd
    );
    assert_eq!(
        classify_disk_kind(Some(BusTypeSata), Some(false)),
        DiskKind::Ssd
    );
    assert_eq!(
        classify_disk_kind(Some(BusTypeSata), Some(true)),
        DiskKind::Hdd
    );
    assert_eq!(classify_disk_kind(None, None), DiskKind::Unknown);
}

#[test]
fn test_telemetry_fetch() {
    let data = TelemetryData::fetch();
    assert!(!data.cpu.name.is_empty());
    assert!(data.ram.total_gb > 0.0);
    assert!(data.ram.available_gb >= 0.0);
    assert!(!data.disks.is_empty());
    assert_eq!(data.cpu_detail.history_15s.len(), CPU_HISTORY_SAMPLES);
    assert_eq!(data.ram.history_15s.len(), CPU_HISTORY_SAMPLES);
}