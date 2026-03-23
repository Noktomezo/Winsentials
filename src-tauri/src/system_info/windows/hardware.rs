use std::collections::HashMap;

use crate::system_info::types::{MotherboardInfo, RamInfo};

pub struct CpuExtra {
    pub sockets: u32,
    pub virtualization: bool,
    pub l1_cache_kb: Option<u32>,
    pub l2_cache_kb: Option<u32>,
    pub l3_cache_kb: Option<u32>,
}

impl Default for CpuExtra {
    fn default() -> Self {
        Self {
            sockets: 1,
            virtualization: false,
            l1_cache_kb: None,
            l2_cache_kb: None,
            l3_cache_kb: None,
        }
    }
}

#[cfg(target_os = "windows")]
pub fn gather_wmi_hardware() -> (MotherboardInfo, RamInfo, String, CpuExtra) {
    use wmi::WMIConnection;

    let default_mb = MotherboardInfo {
        manufacturer: "unknown".to_string(),
        product: "unknown".to_string(),
        bios_vendor: "unknown".to_string(),
        bios_version: "unknown".to_string(),
    };
    let default_ram = RamInfo {
        total_bytes: 0,
        speed_mhz: None,
        used_slots: 0,
        total_slots: 0,
        form_factor: None,
    };

    let fallback_mb = default_mb.clone();
    let fallback_ram = default_ram.clone();
    std::thread::spawn(move || -> (MotherboardInfo, RamInfo, String, CpuExtra) {
        let Ok(wmi_con) = WMIConnection::new() else {
            return (
                default_mb,
                default_ram,
                "unknown".to_string(),
                CpuExtra::default(),
            );
        };

        let mut manufacturer = "Unknown".to_string();
        let mut product = "Unknown".to_string();
        if let Ok(rows) = wmi_con.raw_query::<HashMap<String, wmi::Variant>>(
            "SELECT Manufacturer, Product FROM Win32_BaseBoard",
        ) && let Some(mut row) = rows.into_iter().next()
        {
            manufacturer = wmi_str(&mut row, "Manufacturer");
            product = wmi_str(&mut row, "Product");
        }

        let mut bios_vendor = "Unknown".to_string();
        let mut bios_version = "Unknown".to_string();
        if let Ok(rows) = wmi_con.raw_query::<HashMap<String, wmi::Variant>>(
            "SELECT Manufacturer, SMBIOSBIOSVersion FROM Win32_BIOS",
        ) && let Some(mut row) = rows.into_iter().next()
        {
            bios_vendor = wmi_str(&mut row, "Manufacturer");
            bios_version = wmi_str(&mut row, "SMBIOSBIOSVersion");
        }

        let sticks: Vec<HashMap<String, wmi::Variant>> = wmi_con
            .raw_query("SELECT Speed, Capacity, FormFactor FROM Win32_PhysicalMemory")
            .unwrap_or_default();
        let used_slots = sticks.len() as u32;
        let speed_mhz = sticks.iter().find_map(|row| match row.get("Speed") {
            Some(wmi::Variant::UI4(v)) if *v > 0 => Some(*v),
            _ => None,
        });
        let form_factor = sticks.iter().find_map(|row| match row.get("FormFactor") {
            Some(wmi::Variant::UI2(v)) => Some(
                match v {
                    8 => "DIMM",
                    12 => "SODIMM",
                    13 => "TSOP",
                    9 => "RIMM",
                    _ => "Unknown",
                }
                .to_string(),
            ),
            _ => None,
        });

        let mut total_slots: u32 = used_slots;
        if let Ok(rows) = wmi_con.raw_query::<HashMap<String, wmi::Variant>>(
            "SELECT MemoryDevices FROM Win32_PhysicalMemoryArray",
        ) && let Some(row) = rows.into_iter().next()
        {
            match row.get("MemoryDevices") {
                Some(wmi::Variant::UI2(v)) => total_slots = *v as u32,
                Some(wmi::Variant::UI4(v)) => total_slots = *v,
                _ => {}
            }
        }

        let total_bytes: u64 = sticks
            .iter()
            .filter_map(|row| match row.get("Capacity") {
                Some(wmi::Variant::UI8(v)) => Some(*v),
                Some(wmi::Variant::UI4(v)) => Some(*v as u64),
                _ => None,
            })
            .sum();

        let activation_status = {
            let rows = wmi_con
                .raw_query::<HashMap<String, wmi::Variant>>(
                    "SELECT LicenseStatus FROM SoftwareLicensingProduct \
                     WHERE PartialProductKey IS NOT NULL \
                     AND ApplicationId='55c92734-d682-4d71-983e-d6ec3f16059f'",
                )
                .unwrap_or_default();
            let code = rows
                .into_iter()
                .next()
                .and_then(|mut row| match row.remove("LicenseStatus") {
                    Some(wmi::Variant::UI4(v)) => Some(v),
                    _ => None,
                })
                .unwrap_or(0);
            match code {
                1 => "activated".to_string(),
                2 | 3 | 6 => "grace_period".to_string(),
                _ => "not_activated".to_string(),
            }
        };

        let proc_rows: Vec<HashMap<String, wmi::Variant>> = wmi_con
            .raw_query(
                "SELECT VirtualizationFirmwareEnabled, L2CacheSize, L3CacheSize \
                     FROM Win32_Processor",
            )
            .unwrap_or_default();
        let cpu_extra = {
            let sockets = proc_rows.len().max(1) as u32;
            let virtualization = proc_rows.iter().any(|r| {
                matches!(
                    r.get("VirtualizationFirmwareEnabled"),
                    Some(wmi::Variant::Bool(true))
                )
            });
            let l2_cache_kb = proc_rows.first().and_then(|r| match r.get("L2CacheSize") {
                Some(wmi::Variant::UI4(v)) if *v > 0 => Some(*v),
                _ => None,
            });
            let l3_cache_kb = proc_rows.first().and_then(|r| match r.get("L3CacheSize") {
                Some(wmi::Variant::UI4(v)) if *v > 0 => Some(*v),
                _ => None,
            });
            let l1_rows: Vec<HashMap<String, wmi::Variant>> = wmi_con
                .raw_query("SELECT InstalledSize FROM Win32_CacheMemory WHERE Level=3")
                .unwrap_or_default();
            let l1_sum: u32 = l1_rows
                .iter()
                .filter_map(|r| match r.get("InstalledSize") {
                    Some(wmi::Variant::UI4(v)) => Some(*v),
                    _ => None,
                })
                .sum();
            CpuExtra {
                sockets,
                virtualization,
                l1_cache_kb: if l1_sum > 0 { Some(l1_sum) } else { None },
                l2_cache_kb,
                l3_cache_kb,
            }
        };

        (
            MotherboardInfo {
                manufacturer,
                product,
                bios_vendor,
                bios_version,
            },
            RamInfo {
                total_bytes,
                speed_mhz,
                used_slots,
                total_slots,
                form_factor,
            },
            activation_status,
            cpu_extra,
        )
    })
    .join()
    .unwrap_or_else(|_| {
        (
            fallback_mb,
            fallback_ram,
            "unknown".to_string(),
            CpuExtra::default(),
        )
    })
}

#[cfg(not(target_os = "windows"))]
pub fn gather_wmi_hardware() -> (MotherboardInfo, RamInfo, String, CpuExtra) {
    (
        MotherboardInfo {
            manufacturer: "unknown".to_string(),
            product: "unknown".to_string(),
            bios_vendor: "unknown".to_string(),
            bios_version: "unknown".to_string(),
        },
        RamInfo {
            total_bytes: 0,
            speed_mhz: None,
            used_slots: 0,
            total_slots: 0,
            form_factor: None,
        },
        "unknown".to_string(),
        CpuExtra::default(),
    )
}

#[cfg(target_os = "windows")]
fn wmi_str(row: &mut HashMap<String, wmi::Variant>, key: &str) -> String {
    match row.remove(key) {
        Some(wmi::Variant::String(s)) => s.trim().to_string(),
        _ => "unknown".to_string(),
    }
}
