use std::collections::HashMap;
use std::time::Instant;

use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
use windows_sys::Win32::Storage::FileSystem::{
    BusTypeNvme, CreateFileW, FILE_SHARE_READ, FILE_SHARE_WRITE, GetDiskFreeSpaceExW,
    GetDriveTypeW, GetLogicalDriveStringsW, GetVolumeInformationW, OPEN_EXISTING, STORAGE_BUS_TYPE,
};
use windows_sys::Win32::System::IO::DeviceIoControl;
use windows_sys::Win32::System::Ioctl::{
    DEVICE_SEEK_PENALTY_DESCRIPTOR, DISK_PERFORMANCE, IOCTL_DISK_PERFORMANCE,
    IOCTL_STORAGE_QUERY_PROPERTY, PropertyStandardQuery, STORAGE_DEVICE_DESCRIPTOR,
    STORAGE_PROPERTY_QUERY, StorageDeviceProperty, StorageDeviceSeekPenaltyProperty,
};

use super::types::{CPU_HISTORY_SAMPLES, DiskInfo, DiskKind, transfer_scale};

const DRIVE_REMOVABLE: u32 = 2;
const DRIVE_FIXED: u32 = 3;

#[derive(Clone, Copy, Debug)]
pub(crate) struct DiskPerformanceSnapshot {
    pub(crate) bytes_read: u64,
    pub(crate) bytes_written: u64,
    pub(crate) read_time: u64,
    pub(crate) write_time: u64,
    pub(crate) idle_time: u64,
    pub(crate) read_count: u32,
    pub(crate) write_count: u32,
    pub(crate) query_time: u64,
}

#[repr(C, align(8))]
struct StorageDescriptorBuffer([u8; 1024]);

fn query_disk_performance(letter: char) -> Option<DiskPerformanceSnapshot> {
    let path = format!(r"\\.\{letter}:")
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();

    #[allow(unsafe_code)]
    // SAFETY: `path` is a live, null-terminated UTF-16 volume path; all optional pointers are null.
    let handle = unsafe {
        CreateFileW(
            path.as_ptr(),
            0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            std::ptr::null(),
            OPEN_EXISTING,
            0,
            std::ptr::null_mut(),
        )
    };

    if handle == INVALID_HANDLE_VALUE {
        return None;
    }

    #[allow(unsafe_code)]
    // SAFETY: zeroes match a blank `DISK_PERFORMANCE` struct layout before ioctl populates it.
    let mut perf: DISK_PERFORMANCE = unsafe { std::mem::zeroed() };
    let mut bytes_returned = 0u32;

    #[allow(unsafe_code)]
    // SAFETY: `handle` is open to the physical drive; output buffer pointer and size match `perf`.
    let ok = unsafe {
        DeviceIoControl(
            handle,
            IOCTL_DISK_PERFORMANCE,
            std::ptr::null(),
            0,
            std::ptr::addr_of_mut!(perf).cast(),
            std::mem::size_of::<DISK_PERFORMANCE>() as u32,
            &raw mut bytes_returned,
            std::ptr::null_mut(),
        )
    };

    #[allow(unsafe_code)]
    // SAFETY: `handle` was returned by `CreateFileW` and is closed exactly once.
    unsafe {
        CloseHandle(handle);
    }

    if ok == 0 {
        return None;
    }

    Some(DiskPerformanceSnapshot {
        bytes_read: perf.BytesRead as u64,
        bytes_written: perf.BytesWritten as u64,
        read_time: perf.ReadTime as u64,
        write_time: perf.WriteTime as u64,
        idle_time: perf.IdleTime as u64,
        read_count: perf.ReadCount,
        write_count: perf.WriteCount,
        query_time: perf.QueryTime as u64,
    })
}

pub(crate) const fn classify_disk_kind(
    bus_type: Option<STORAGE_BUS_TYPE>,
    incurs_seek_penalty: Option<bool>,
) -> DiskKind {
    if let Some(bus) = bus_type {
        if bus == BusTypeNvme {
            return DiskKind::NvmeSsd;
        }
    }

    match incurs_seek_penalty {
        Some(false) => DiskKind::Ssd,
        Some(true) => DiskKind::Hdd,
        None => DiskKind::Unknown,
    }
}

fn query_disk_kind(letter: char) -> DiskKind {
    let path = format!(r"\\.\{letter}:")
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect::<Vec<_>>();

    #[allow(unsafe_code)]
    // SAFETY: `path` is a live, null-terminated UTF-16 volume path; all optional pointers are null.
    let handle = unsafe {
        CreateFileW(
            path.as_ptr(),
            0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            std::ptr::null(),
            OPEN_EXISTING,
            0,
            std::ptr::null_mut(),
        )
    };

    if handle == INVALID_HANDLE_VALUE {
        return DiskKind::Unknown;
    }

    let mut device_query = STORAGE_PROPERTY_QUERY {
        PropertyId: StorageDeviceProperty,
        QueryType: PropertyStandardQuery,
        AdditionalParameters: [0],
    };
    let mut descriptor_buffer = StorageDescriptorBuffer([0; 1024]);
    let mut bytes_returned = 0u32;

    #[allow(unsafe_code)]
    // SAFETY: buffer size matches `StorageDescriptorBuffer`; `device_query` points to a live query.
    let dev_ok = unsafe {
        DeviceIoControl(
            handle,
            IOCTL_STORAGE_QUERY_PROPERTY,
            std::ptr::addr_of_mut!(device_query).cast(),
            std::mem::size_of::<STORAGE_PROPERTY_QUERY>() as u32,
            descriptor_buffer.0.as_mut_ptr().cast(),
            descriptor_buffer.0.len() as u32,
            &raw mut bytes_returned,
            std::ptr::null_mut(),
        )
    };

    let bus_type = if dev_ok != 0
        && bytes_returned >= std::mem::size_of::<STORAGE_DEVICE_DESCRIPTOR>() as u32
    {
        #[allow(unsafe_code)]
        // SAFETY: `descriptor_buffer` holds a valid `STORAGE_DEVICE_DESCRIPTOR` prefix.
        let desc = unsafe { &*descriptor_buffer.0.as_ptr().cast::<STORAGE_DEVICE_DESCRIPTOR>() };
        Some(desc.BusType)
    } else {
        None
    };

    let mut seek_query = STORAGE_PROPERTY_QUERY {
        PropertyId: StorageDeviceSeekPenaltyProperty,
        QueryType: PropertyStandardQuery,
        AdditionalParameters: [0],
    };
    let mut seek_penalty = std::mem::MaybeUninit::<DEVICE_SEEK_PENALTY_DESCRIPTOR>::uninit();
    let seek_size = std::mem::size_of::<DEVICE_SEEK_PENALTY_DESCRIPTOR>() as u32;

    #[allow(unsafe_code)]
    // SAFETY: query points to `seek_query` and output buffer is sized for `DEVICE_SEEK_PENALTY_DESCRIPTOR`.
    let seek_ok = unsafe {
        DeviceIoControl(
            handle,
            IOCTL_STORAGE_QUERY_PROPERTY,
            std::ptr::addr_of_mut!(seek_query).cast(),
            std::mem::size_of::<STORAGE_PROPERTY_QUERY>() as u32,
            seek_penalty.as_mut_ptr().cast(),
            seek_size,
            &raw mut bytes_returned,
            std::ptr::null_mut(),
        )
    };
    let incurs_seek_penalty = if seek_ok != 0 && bytes_returned >= seek_size {
        #[allow(unsafe_code)]
        // SAFETY: a successful query initialized the full seek-penalty descriptor.
        Some(unsafe { seek_penalty.assume_init() }.IncursSeekPenalty != 0)
    } else {
        None
    };

    #[allow(unsafe_code)]
    // SAFETY: `handle` was returned by `CreateFileW` and is closed exactly once.
    unsafe {
        CloseHandle(handle);
    }

    classify_disk_kind(bus_type, incurs_seek_penalty)
}

#[allow(clippy::cast_precision_loss, clippy::similar_names)]
pub(crate) fn sample_disks(
    disk_kinds: &mut HashMap<char, DiskKind>,
    disk_snapshots: &mut HashMap<char, DiskPerformanceSnapshot>,
    disk_active_histories: &mut HashMap<char, Vec<f32>>,
    disk_transfer_histories: &mut HashMap<char, Vec<f32>>,
    last_disk_transfer_scales: &mut HashMap<char, f32>,
    elapsed_secs: f64,
) -> Vec<DiskInfo> {
    let mut disks = Vec::new();
    let mut buffer = [0u16; 512];
    let system_drive = std::env::var("SystemDrive")
        .ok()
        .and_then(|drive| drive.chars().next())
        .map(|letter| letter.to_ascii_uppercase());

    #[allow(unsafe_code)]
    let len = unsafe { GetLogicalDriveStringsW(buffer.len() as u32, buffer.as_mut_ptr()) };

    if len > 0 {
        let mut offset = 0;
        let mut disk_id = 0;

        while offset < len as usize && buffer[offset] != 0 {
            let mut end = offset;
            while end < buffer.len() && buffer[end] != 0 {
                end += 1;
            }

            let drive_root_slice = &buffer[offset..=end];
            #[allow(unsafe_code)]
            let drive_type = unsafe { GetDriveTypeW(drive_root_slice.as_ptr()) };

            if drive_type == DRIVE_FIXED || drive_type == DRIVE_REMOVABLE {
                let mut free_avail = 0u64;
                let mut total_bytes = 0u64;
                let mut total_free = 0u64;

                #[allow(unsafe_code)]
                let space_ok = unsafe {
                    GetDiskFreeSpaceExW(
                        drive_root_slice.as_ptr(),
                        &raw mut free_avail,
                        &raw mut total_bytes,
                        &raw mut total_free,
                    )
                };

                if space_ok != 0 && total_bytes > 0 {
                    let total_gb = total_bytes / (1024 * 1024 * 1024);
                    let used_gb = (total_bytes.saturating_sub(total_free)) / (1024 * 1024 * 1024);

                    let mut vol_name_buf = [0u16; 261];
                    let mut serial = 0u32;
                    let mut max_comp_len = 0u32;
                    let mut flags = 0u32;
                    let mut fs_name_buf = [0u16; 261];

                    #[allow(unsafe_code)]
                    let vol_ok = unsafe {
                        GetVolumeInformationW(
                            drive_root_slice.as_ptr(),
                            vol_name_buf.as_mut_ptr(),
                            vol_name_buf.len() as u32,
                            &raw mut serial,
                            &raw mut max_comp_len,
                            &raw mut flags,
                            fs_name_buf.as_mut_ptr(),
                            fs_name_buf.len() as u32,
                        )
                    };

                    let custom_name = if vol_ok != 0 {
                        let vol_len = vol_name_buf.iter().position(|&c| c == 0).unwrap_or(0);
                        let label = String::from_utf16_lossy(&vol_name_buf[..vol_len])
                            .trim()
                            .to_string();
                        if label.is_empty() {
                            None
                        } else {
                            Some(label.into())
                        }
                    } else {
                        None
                    };

                    let file_system = if vol_ok != 0 {
                        let fs_len = fs_name_buf.iter().position(|&c| c == 0).unwrap_or(0);
                        String::from_utf16_lossy(&fs_name_buf[..fs_len])
                            .trim()
                            .to_string()
                    } else {
                        String::new()
                    };

                    let drive_str = String::from_utf16_lossy(&buffer[offset..end]);
                    let letter = drive_str.chars().next().unwrap_or('C').to_ascii_uppercase();
                    let clean_letter = letter.to_string();

                    let kind = disk_kinds.get(&letter).copied().unwrap_or_else(|| {
                        let kind = query_disk_kind(letter);
                        if kind != DiskKind::Unknown {
                            disk_kinds.insert(letter, kind);
                        }
                        kind
                    });

                    let mut active_percent = 0.0;
                    let mut read_mb_s = 0.0;
                    let mut write_mb_s = 0.0;
                    let mut average_response_ms = 0.0;
                    if let Some(current) = query_disk_performance(letter) {
                        if let Some(previous) = disk_snapshots.insert(letter, current) {
                            let delta_query =
                                current.query_time.saturating_sub(previous.query_time);
                            let delta_idle =
                                current.idle_time.saturating_sub(previous.idle_time);
                            if delta_query > 0 {
                                active_percent = (100.0
                                    * (1.0 - delta_idle as f64 / delta_query as f64))
                                    .clamp(0.0, 100.0)
                                    as f32;
                            }

                            read_mb_s = (current.bytes_read.saturating_sub(previous.bytes_read)
                                as f64
                                / elapsed_secs
                                / (1024.0 * 1024.0))
                                as f32;
                            write_mb_s =
                                (current.bytes_written.saturating_sub(previous.bytes_written)
                                    as f64
                                    / elapsed_secs
                                    / (1024.0 * 1024.0))
                                    as f32;

                            let operations = current
                                .read_count
                                .saturating_sub(previous.read_count)
                                .saturating_add(
                                    current.write_count.saturating_sub(previous.write_count),
                                );
                            if operations > 0 {
                                let elapsed_io = current
                                    .read_time
                                    .saturating_sub(previous.read_time)
                                    .saturating_add(
                                        current.write_time.saturating_sub(previous.write_time),
                                    );
                                average_response_ms =
                                    (elapsed_io as f64 / f64::from(operations) / 10_000.0)
                                        as f32;
                            }
                        }
                    }

                    let active_history = disk_active_histories
                        .entry(letter)
                        .or_insert_with(|| vec![0.0; CPU_HISTORY_SAMPLES]);
                    active_history.rotate_left(1);
                    active_history[CPU_HISTORY_SAMPLES - 1] = active_percent;

                    let transfer_history = disk_transfer_histories
                        .entry(letter)
                        .or_insert_with(|| vec![0.0; CPU_HISTORY_SAMPLES]);
                    transfer_history.rotate_left(1);
                    transfer_history[CPU_HISTORY_SAMPLES - 1] = read_mb_s + write_mb_s;

                    let target_scale = transfer_scale(transfer_history);
                    let previous_transfer_scale = last_disk_transfer_scales
                        .insert(letter, target_scale)
                        .unwrap_or(target_scale);

                    disks.push(DiskInfo {
                        id: disk_id,
                        letter: clean_letter.into(),
                        custom_name,
                        used_gb,
                        total_gb,
                        file_system: file_system.into(),
                        kind,
                        is_removable: drive_type == DRIVE_REMOVABLE,
                        is_system: system_drive == Some(letter),
                        active_percent,
                        read_mb_s,
                        write_mb_s,
                        average_response_ms,
                        active_history_15s: active_history.clone(),
                        transfer_history_15s: transfer_history.clone(),
                        previous_transfer_scale,
                        transfer_scale: target_scale,
                        sample_instant: Instant::now(),
                    });
                    disk_id += 1;
                }
            }

            offset = end + 1;
        }
    }

    if disks.is_empty() {
        disks.push(DiskInfo {
            id: 0,
            letter: "C".into(),
            custom_name: None,
            used_gb: 342,
            total_gb: 1024,
            file_system: "NTFS".into(),
            kind: DiskKind::Unknown,
            is_removable: false,
            is_system: true,
            active_percent: 0.0,
            read_mb_s: 0.0,
            write_mb_s: 0.0,
            average_response_ms: 0.0,
            active_history_15s: vec![0.0; CPU_HISTORY_SAMPLES],
            transfer_history_15s: vec![0.0; CPU_HISTORY_SAMPLES],
            previous_transfer_scale: 50.0,
            transfer_scale: 50.0,
            sample_instant: Instant::now(),
        });
    }

    disks
}