use std::collections::HashMap;
#[cfg(target_os = "windows")]
use std::ptr;

use sysinfo::{DiskKind, Disks};

use crate::system_info::types::{DiskInfo, DiskLiveInfo};

#[derive(Debug, Clone)]
struct DiskMetadataInfo {
    is_system_disk: bool,
    has_pagefile: bool,
    type_label: String,
}

#[cfg(target_os = "windows")]
fn query_disk_models() -> HashMap<String, String> {
    use wmi::WMIConnection;

    fn wmi_string(row: &mut HashMap<String, wmi::Variant>, key: &str) -> Option<String> {
        match row.remove(key) {
            Some(wmi::Variant::String(value)) => {
                let trimmed = value.trim().to_string();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed)
                }
            }
            _ => None,
        }
    }

    fn escape_wmi_value(value: &str) -> String {
        value.replace('\'', "''").replace('\\', r"\\")
    }

    let Ok(wmi_con) = WMIConnection::new() else {
        return HashMap::new();
    };

    let Ok(drives) = wmi_con
        .raw_query::<HashMap<String, wmi::Variant>>("SELECT DeviceID, Model FROM Win32_DiskDrive")
    else {
        return HashMap::new();
    };

    let mut models_by_mount = HashMap::new();

    for mut drive in drives {
        let Some(device_id) = wmi_string(&mut drive, "DeviceID") else {
            continue;
        };
        let Some(model) = wmi_string(&mut drive, "Model") else {
            continue;
        };

        let partitions_query = format!(
            "ASSOCIATORS OF {{Win32_DiskDrive.DeviceID='{}'}} WHERE AssocClass = Win32_DiskDriveToDiskPartition",
            escape_wmi_value(&device_id),
        );

        let Ok(partitions) = wmi_con.raw_query::<HashMap<String, wmi::Variant>>(&partitions_query)
        else {
            continue;
        };

        for mut partition in partitions {
            let Some(partition_id) = wmi_string(&mut partition, "DeviceID") else {
                continue;
            };

            let logical_disks_query = format!(
                "ASSOCIATORS OF {{Win32_DiskPartition.DeviceID='{}'}} WHERE AssocClass = Win32_LogicalDiskToPartition",
                escape_wmi_value(&partition_id),
            );

            let Ok(logical_disks) =
                wmi_con.raw_query::<HashMap<String, wmi::Variant>>(&logical_disks_query)
            else {
                continue;
            };

            for mut logical_disk in logical_disks {
                let Some(logical_disk_id) = wmi_string(&mut logical_disk, "DeviceID") else {
                    continue;
                };
                let mount_point = format!("{}\\", logical_disk_id.trim_end_matches('\\'));
                models_by_mount.insert(mount_point, model.clone());
            }
        }
    }

    models_by_mount
}

#[cfg(not(target_os = "windows"))]
fn query_disk_models() -> HashMap<String, String> {
    HashMap::new()
}

#[cfg(target_os = "windows")]
pub fn get_volume_label(mount_point: &str) -> Option<String> {
    use windows::Win32::Storage::FileSystem::GetVolumeInformationW;
    let path: Vec<u16> = mount_point
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let mut name_buf = vec![0u16; 256];
    unsafe {
        GetVolumeInformationW(
            windows::core::PCWSTR(path.as_ptr()),
            Some(&mut name_buf),
            None,
            None,
            None,
            None,
        )
        .ok()?;
    }
    let end = name_buf
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(name_buf.len());
    let label = String::from_utf16_lossy(&name_buf[..end]);
    if label.is_empty() { None } else { Some(label) }
}

#[cfg(not(target_os = "windows"))]
pub fn get_volume_label(_mount_point: &str) -> Option<String> {
    None
}

#[cfg(target_os = "windows")]
fn get_system_drive_root() -> Option<String> {
    std::env::var("SystemDrive")
        .ok()
        .filter(|value| value.len() >= 2)
        .map(|value| format!("{}\\", &value[..2]))
}

#[cfg(not(target_os = "windows"))]
fn get_system_drive_root() -> Option<String> {
    None
}

#[cfg(target_os = "windows")]
fn normalize_bus_label(bus_type: u32) -> Option<&'static str> {
    match bus_type {
        17 => Some("NVMe"),
        11 | 3 => Some("SATA"),
        8 => Some("RAID"),
        7 => Some("USB"),
        _ => None,
    }
}

fn default_disk_type_label(kind: &str) -> String {
    match kind {
        "SSD" => "SSD".to_string(),
        "HDD" => "HDD".to_string(),
        _ => "Unknown".to_string(),
    }
}

#[cfg(target_os = "windows")]
fn query_disk_type_label(mount_point: &str, fallback_kind: &str) -> String {
    use std::ffi::c_void;
    use windows::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
    use windows::Win32::Storage::FileSystem::{
        CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
    };
    use windows::Win32::System::IO::DeviceIoControl;
    use windows::Win32::System::Ioctl::{
        DEVICE_SEEK_PENALTY_DESCRIPTOR, IOCTL_STORAGE_QUERY_PROPERTY, PropertyStandardQuery,
        STORAGE_DESCRIPTOR_HEADER, STORAGE_DEVICE_DESCRIPTOR, STORAGE_PROPERTY_QUERY,
        StorageDeviceProperty, StorageDeviceSeekPenaltyProperty,
    };

    let volume = mount_point.trim_end_matches('\\');
    if volume.len() < 2 {
        return default_disk_type_label(fallback_kind);
    }

    let device_path = format!(r"\\.\{}", volume);
    let wide: Vec<u16> = device_path
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let handle = match CreateFileW(
            windows::core::PCWSTR(wide.as_ptr()),
            0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            None,
        ) {
            Ok(handle) => handle,
            Err(_) => return default_disk_type_label(fallback_kind),
        };

        if handle == INVALID_HANDLE_VALUE {
            return default_disk_type_label(fallback_kind);
        }

        let mut bytes_returned = 0u32;
        let mut query = STORAGE_PROPERTY_QUERY {
            PropertyId: StorageDeviceProperty,
            QueryType: PropertyStandardQuery,
            AdditionalParameters: [0],
        };
        let mut header = STORAGE_DESCRIPTOR_HEADER::default();
        let header_ok = DeviceIoControl(
            handle,
            IOCTL_STORAGE_QUERY_PROPERTY,
            Some(&mut query as *mut _ as *mut c_void),
            std::mem::size_of::<STORAGE_PROPERTY_QUERY>() as u32,
            Some(&mut header as *mut _ as *mut c_void),
            std::mem::size_of::<STORAGE_DESCRIPTOR_HEADER>() as u32,
            Some(&mut bytes_returned),
            None,
        )
        .is_ok();

        let mut type_label = default_disk_type_label(fallback_kind);

        if header_ok && header.Size as usize >= std::mem::size_of::<STORAGE_DEVICE_DESCRIPTOR>() {
            let mut buffer = vec![0u8; header.Size as usize];
            if DeviceIoControl(
                handle,
                IOCTL_STORAGE_QUERY_PROPERTY,
                Some(&mut query as *mut _ as *mut c_void),
                std::mem::size_of::<STORAGE_PROPERTY_QUERY>() as u32,
                Some(buffer.as_mut_ptr() as *mut c_void),
                buffer.len() as u32,
                Some(&mut bytes_returned),
                None,
            )
            .is_ok()
                && bytes_returned as usize >= std::mem::size_of::<STORAGE_DEVICE_DESCRIPTOR>()
            {
                let descriptor =
                    ptr::read_unaligned(buffer.as_ptr() as *const STORAGE_DEVICE_DESCRIPTOR);
                let bus_label = normalize_bus_label(descriptor.BusType.0 as u32);

                let mut seek_query = STORAGE_PROPERTY_QUERY {
                    PropertyId: StorageDeviceSeekPenaltyProperty,
                    QueryType: PropertyStandardQuery,
                    AdditionalParameters: [0],
                };
                let mut seek = DEVICE_SEEK_PENALTY_DESCRIPTOR::default();
                let has_seek_penalty = if DeviceIoControl(
                    handle,
                    IOCTL_STORAGE_QUERY_PROPERTY,
                    Some(&mut seek_query as *mut _ as *mut c_void),
                    std::mem::size_of::<STORAGE_PROPERTY_QUERY>() as u32,
                    Some(&mut seek as *mut _ as *mut c_void),
                    std::mem::size_of::<DEVICE_SEEK_PENALTY_DESCRIPTOR>() as u32,
                    Some(&mut bytes_returned),
                    None,
                )
                .is_ok()
                {
                    Some(seek.IncursSeekPenalty)
                } else {
                    None
                };

                type_label = match (fallback_kind, bus_label, has_seek_penalty) {
                    ("SSD", Some(bus), _) => format!("SSD ({bus})"),
                    ("HDD", Some(bus), _) => format!("HDD ({bus})"),
                    ("unknown", Some(bus), Some(true)) => format!("HDD ({bus})"),
                    ("unknown", Some(bus), Some(false)) => format!("SSD ({bus})"),
                    ("unknown", Some(_), None) => default_disk_type_label("unknown"),
                    (kind, _, _) => default_disk_type_label(kind),
                };
            }
        }

        let _ = CloseHandle(handle);
        type_label
    }
}

#[cfg(not(target_os = "windows"))]
fn query_disk_type_label(_mount_point: &str, fallback_kind: &str) -> String {
    default_disk_type_label(fallback_kind)
}

#[cfg(target_os = "windows")]
fn disk_has_pagefile(mount_point: &str) -> bool {
    std::path::Path::new(mount_point)
        .join("pagefile.sys")
        .exists()
}

#[cfg(not(target_os = "windows"))]
fn disk_has_pagefile(_mount_point: &str) -> bool {
    false
}

#[cfg(target_os = "windows")]
pub fn pdh_open_disk_query() -> Option<(isize, isize, isize, isize, isize)> {
    use windows::Win32::System::Performance::*;

    unsafe {
        let mut query = PDH_HQUERY(std::ptr::null_mut());
        if PdhOpenQueryW(windows::core::PCWSTR(std::ptr::null()), 0, &mut query) != 0 {
            return None;
        }

        let paths = [
            windows::core::w!(r"\LogicalDisk(*)\% Disk Time"),
            windows::core::w!(r"\LogicalDisk(*)\Avg. Disk sec/Transfer"),
            windows::core::w!(r"\LogicalDisk(*)\Disk Read Bytes/sec"),
            windows::core::w!(r"\LogicalDisk(*)\Disk Write Bytes/sec"),
        ];
        let mut counters = [PDH_HCOUNTER(std::ptr::null_mut()); 4];

        for (index, path) in paths.into_iter().enumerate() {
            if PdhAddEnglishCounterW(query, path, 0, &mut counters[index]) != 0 {
                let _ = PdhCloseQuery(query);
                return None;
            }
        }

        let _ = PdhCollectQueryData(query);
        Some((
            query.0 as isize,
            counters[0].0 as isize,
            counters[1].0 as isize,
            counters[2].0 as isize,
            counters[3].0 as isize,
        ))
    }
}

#[cfg(not(target_os = "windows"))]
pub fn pdh_open_disk_query() -> Option<(isize, isize, isize, isize, isize)> {
    None
}

#[cfg(target_os = "windows")]
fn pdh_collect_disk_counter_array(counter_raw: isize) -> HashMap<String, f64> {
    use windows::Win32::System::Performance::*;
    const PDH_CSTATUS_NEW_DATA: u32 = 0x00000001;
    const PDH_CSTATUS_VALID_DATA: u32 = 0x00000000;
    const PDH_MORE_DATA: u32 = 0x800007D2;

    unsafe {
        let counter = PDH_HCOUNTER(counter_raw as *mut _);
        let mut buf_size = 0u32;
        let mut item_count = 0u32;
        let result = PdhGetFormattedCounterArrayW(
            counter,
            PDH_FMT_DOUBLE,
            &mut buf_size,
            &mut item_count,
            None,
        );
        if result != PDH_MORE_DATA && result != 0 {
            return HashMap::new();
        }
        if buf_size == 0 {
            return HashMap::new();
        }

        let item_capacity =
            (buf_size as usize).div_ceil(std::mem::size_of::<PDH_FMT_COUNTERVALUE_ITEM_W>());
        let mut buffer: Vec<PDH_FMT_COUNTERVALUE_ITEM_W> = Vec::with_capacity(item_capacity);
        let result = PdhGetFormattedCounterArrayW(
            counter,
            PDH_FMT_DOUBLE,
            &mut buf_size,
            &mut item_count,
            Some(buffer.as_mut_ptr()),
        );
        if result != 0 {
            return HashMap::new();
        }
        buffer.set_len(item_count as usize);
        let buffer_start = buffer.as_ptr().cast::<u8>();
        let buffer_end = buffer_start.add(buf_size as usize);

        buffer
            .iter()
            .filter_map(|item| {
                if item.FmtValue.CStatus != PDH_CSTATUS_VALID_DATA
                    && item.FmtValue.CStatus != PDH_CSTATUS_NEW_DATA
                {
                    return None;
                }
                let ptr = item.szName.0;
                if ptr.is_null() {
                    return None;
                }

                let ptr_bytes = ptr.cast_const().cast::<u8>();
                if ptr_bytes < buffer_start || ptr_bytes >= buffer_end {
                    return None;
                }

                let original_max_u16_len = (buf_size as usize).div_ceil(std::mem::size_of::<u16>());
                let remaining_bytes = buffer_end.offset_from(ptr_bytes) as usize;
                let remaining_u16_len = remaining_bytes / std::mem::size_of::<u16>();
                let max_u16_len = original_max_u16_len.min(remaining_u16_len);
                let len = (0usize..max_u16_len)
                    .take_while(|&j| *ptr.add(j) != 0)
                    .count();
                let instance = String::from_utf16_lossy(std::slice::from_raw_parts(ptr, len));
                if instance == "_Total" || !instance.ends_with(':') {
                    return None;
                }

                Some((instance, item.FmtValue.Anonymous.doubleValue.max(0.0)))
            })
            .collect()
    }
}

#[cfg(not(target_os = "windows"))]
fn pdh_collect_disk_counter_array(_counter_raw: isize) -> HashMap<String, f64> {
    HashMap::new()
}

#[cfg(target_os = "windows")]
pub fn gather_disk_live(
    query_raw: isize,
    active_counter_raw: isize,
    response_counter_raw: isize,
    read_counter_raw: isize,
    write_counter_raw: isize,
) -> HashMap<String, DiskLiveInfo> {
    use windows::Win32::System::Performance::{PDH_HQUERY, PdhCollectQueryData};

    if query_raw == 0 {
        return HashMap::new();
    }

    unsafe {
        let query = PDH_HQUERY(query_raw as *mut _);
        if PdhCollectQueryData(query) != 0 {
            return HashMap::new();
        }
    }

    let active = pdh_collect_disk_counter_array(active_counter_raw);
    let response = pdh_collect_disk_counter_array(response_counter_raw);
    let read = pdh_collect_disk_counter_array(read_counter_raw);
    let write = pdh_collect_disk_counter_array(write_counter_raw);

    let mut mounts = HashMap::new();
    for mount in active
        .keys()
        .chain(response.keys())
        .chain(read.keys())
        .chain(write.keys())
    {
        let mount_point = format!("{}\\", mount);
        mounts.insert(
            mount_point.clone(),
            DiskLiveInfo {
                mount_point,
                active_time_percent: active
                    .get(mount)
                    .copied()
                    .unwrap_or(0.0)
                    .round()
                    .clamp(0.0, 100.0) as u32,
                avg_response_ms: response.get(mount).copied().unwrap_or(0.0) * 1000.0,
                read_bytes_per_sec: read.get(mount).copied().unwrap_or(0.0).round() as u64,
                write_bytes_per_sec: write.get(mount).copied().unwrap_or(0.0).round() as u64,
            },
        );
    }

    mounts
}

#[cfg(not(target_os = "windows"))]
pub fn gather_disk_live(
    _query_raw: isize,
    _active_counter_raw: isize,
    _response_counter_raw: isize,
    _read_counter_raw: isize,
    _write_counter_raw: isize,
) -> HashMap<String, DiskLiveInfo> {
    HashMap::new()
}

pub fn gather_disks() -> Vec<DiskInfo> {
    let disks = Disks::new_with_refreshed_list();
    let disk_models = query_disk_models();
    let system_drive_root = get_system_drive_root();
    let mut result: Vec<DiskInfo> = disks
        .iter()
        .filter(|d| d.total_space() > 0)
        .map(|d| {
            let mount_point = d.mount_point().to_string_lossy().into_owned();
            let volume_label = get_volume_label(&mount_point);
            let kind = match d.kind() {
                DiskKind::SSD => "SSD".to_string(),
                DiskKind::HDD => "HDD".to_string(),
                DiskKind::Unknown(_) => "unknown".to_string(),
            };
            let metadata = DiskMetadataInfo {
                is_system_disk: system_drive_root
                    .as_deref()
                    .is_some_and(|value| value.eq_ignore_ascii_case(mount_point.as_str())),
                has_pagefile: disk_has_pagefile(&mount_point),
                type_label: query_disk_type_label(&mount_point, &kind),
            };
            DiskInfo {
                name: d.name().to_string_lossy().into_owned(),
                model: disk_models.get(&mount_point).cloned(),
                mount_point,
                total_bytes: d.total_space(),
                available_bytes: d.available_space(),
                kind,
                file_system: d.file_system().to_string_lossy().into_owned(),
                volume_label,
                is_system_disk: metadata.is_system_disk,
                has_pagefile: metadata.has_pagefile,
                type_label: metadata.type_label,
            }
        })
        .collect();
    result.sort_by(|a, b| a.mount_point.cmp(&b.mount_point));
    result
}
