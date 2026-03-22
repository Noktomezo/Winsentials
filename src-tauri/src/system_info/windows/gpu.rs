use std::collections::HashMap;

use rayon::prelude::*;

use crate::system_info::types::{GpuInfo, LiveGpuMetrics};

#[cfg(target_os = "windows")]
fn normalise_driver_date(s: &str) -> Option<String> {
    let parts: Vec<&str> = s.split('-').collect();
    if parts.len() == 3 {
        let month: u32 = parts[0].parse().ok()?;
        let day: u32 = parts[1].parse().ok()?;
        let year: u32 = parts[2].parse().ok()?;
        Some(format!("{:04}-{:02}-{:02}", year, month, day))
    } else {
        None
    }
}

#[cfg(target_os = "windows")]
fn parse_pci_location(location: &str) -> (Option<u32>, Option<u32>, Option<u32>) {
    let lower = location.to_ascii_lowercase();
    (
        extract_pci_number(&lower, "bus"),
        extract_pci_number(&lower, "device"),
        extract_pci_number(&lower, "function"),
    )
}

#[cfg(target_os = "windows")]
fn extract_pci_number(s: &str, keyword: &str) -> Option<u32> {
    let pos = s.find(keyword)?;
    let rest = &s[pos + keyword.len()..];
    let start = rest.find(|c: char| c.is_ascii_digit())?;
    rest[start..]
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse()
        .ok()
}

#[cfg(target_os = "windows")]
fn get_pci_location(device_instance_id: &str) -> Option<(Option<u32>, Option<u32>, Option<u32>)> {
    use winreg::RegKey as WinRegKey;
    use winreg::enums::HKEY_LOCAL_MACHINE;
    let hklm = WinRegKey::predef(HKEY_LOCAL_MACHINE);
    let path = format!(r"SYSTEM\CurrentControlSet\Enum\{}", device_instance_id);
    let key = hklm.open_subkey(path).ok()?;
    let location: String = key.get_value("LocationInformation").ok()?;
    Some(parse_pci_location(&location))
}

#[cfg(target_os = "windows")]
#[allow(clippy::type_complexity)]
fn gather_gpu_driver_info(
    model: &str,
    _luid_low: u32,
    _luid_high: i32,
) -> Option<(
    Option<String>,
    Option<String>,
    Option<u32>,
    Option<u32>,
    Option<u32>,
)> {
    use winreg::RegKey as WinRegKey;
    use winreg::enums::HKEY_LOCAL_MACHINE;
    let hklm = WinRegKey::predef(HKEY_LOCAL_MACHINE);
    let class_key = hklm
        .open_subkey(
            r"SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}",
        )
        .ok()?;

    let model_lower = model.to_ascii_lowercase();
    let mut candidates = Vec::new();

    for key_name in class_key.enum_keys().flatten() {
        let subkey = match class_key.open_subkey(&key_name) {
            Ok(k) => k,
            Err(_) => continue,
        };
        let driver_desc: String = match subkey.get_value("DriverDesc") {
            Ok(v) => v,
            Err(_) => continue,
        };
        let desc_lower = driver_desc.to_ascii_lowercase();
        if desc_lower != model_lower
            && !desc_lower.contains(&model_lower)
            && !model_lower.contains(&desc_lower)
        {
            continue;
        }

        let device_instance_id = subkey.get_value::<String, _>("DeviceInstanceID").ok();
        let driver_version: Option<String> = subkey.get_value("DriverVersion").ok();
        let driver_date: Option<String> = subkey
            .get_value::<String, _>("DriverDate")
            .ok()
            .and_then(|s| normalise_driver_date(&s));
        let (pci_bus, pci_device, pci_function) = device_instance_id
            .as_deref()
            .and_then(get_pci_location)
            .unwrap_or((None, None, None));

        candidates.push((
            desc_lower == model_lower,
            pci_bus.is_some() || pci_device.is_some() || pci_function.is_some(),
            driver_version,
            driver_date,
            pci_bus,
            pci_device,
            pci_function,
        ));
    }

    candidates
        .into_iter()
        .max_by_key(|(is_exact, has_pci, ..)| (*is_exact, *has_pci))
        .map(
            |(_, _, driver_version, driver_date, pci_bus, pci_device, pci_function)| {
                (
                    driver_version,
                    driver_date,
                    pci_bus,
                    pci_device,
                    pci_function,
                )
            },
        )
}

#[cfg(target_os = "windows")]
fn directx_from_build(build: u32) -> &'static str {
    if build >= 10240 {
        "12.0"
    } else if build >= 9600 {
        "11.1"
    } else {
        "11.0"
    }
}

#[cfg(target_os = "windows")]
fn classify_integrated_gpu(
    name: &str,
    vendor: &str,
    raw_dedicated_bytes: u64,
    shared_system_bytes: u64,
) -> bool {
    const MB: u64 = 1_048_576;
    const GB: u64 = 1_024 * MB;

    let name_upper = name.to_ascii_uppercase();
    let generic_amd_integrated = matches!(
        name_upper.as_str(),
        "AMD RADEON(TM) GRAPHICS" | "AMD RADEON GRAPHICS"
    );
    let intel_integrated = vendor == "Intel" && name_upper.contains("GRAPHICS");

    if intel_integrated || generic_amd_integrated {
        return true;
    }
    if raw_dedicated_bytes >= 2 * GB {
        return false;
    }
    if shared_system_bytes > 0 && raw_dedicated_bytes <= 512 * MB {
        return true;
    }

    false
}

#[cfg(not(target_os = "windows"))]
pub fn enumerate_dxgi_adapters() -> Vec<(String, u32, i32, u64, u64, u64)> {
    vec![]
}

#[cfg(target_os = "windows")]
pub fn enumerate_dxgi_adapters() -> Vec<(String, u32, i32, u64, u64, u64)> {
    use windows::Wdk::Graphics::Direct3D::*;
    use windows::Win32::Foundation::LUID;
    use windows::Win32::Graphics::Dxgi::*;
    use windows::core::Interface;
    unsafe {
        let mut enum2 = D3DKMT_ENUMADAPTERS2 {
            NumAdapters: 0,
            pAdapters: std::ptr::null_mut(),
        };
        let _ = D3DKMTEnumAdapters2(&mut enum2);
        let count = enum2.NumAdapters as usize;
        if count == 0 {
            return vec![];
        }

        let mut kmt_adapters: Vec<D3DKMT_ADAPTERINFO> = vec![std::mem::zeroed(); count];
        enum2.pAdapters = kmt_adapters.as_mut_ptr();
        if D3DKMTEnumAdapters2(&mut enum2).0 < 0 {
            return vec![];
        }
        let actual = enum2.NumAdapters as usize;

        let factory: IDXGIFactory1 = match CreateDXGIFactory1() {
            Ok(f) => f,
            Err(_) => return vec![],
        };
        let factory4: IDXGIFactory4 = match factory.cast() {
            Ok(f) => f,
            Err(_) => return vec![],
        };

        let mut result = Vec::new();
        for info in &kmt_adapters[..actual] {
            let luid_low = info.AdapterLuid.LowPart;
            let luid_high = info.AdapterLuid.HighPart;

            let adapter: IDXGIAdapter1 = match factory4.EnumAdapterByLuid(LUID {
                LowPart: luid_low,
                HighPart: luid_high,
            }) {
                Ok(a) => a,
                Err(_) => continue,
            };
            let desc = match adapter.GetDesc1() {
                Ok(d) => d,
                Err(_) => continue,
            };
            if desc.Flags & 2 != 0 {
                continue;
            }

            let null_pos = desc.Description.iter().position(|&c| c == 0).unwrap_or(128);
            let name = String::from_utf16_lossy(&desc.Description[..null_pos]);
            let raw_dedicated = desc.DedicatedVideoMemory as u64;
            let shared_system = desc.SharedSystemMemory as u64;
            let vram = if raw_dedicated > 0 {
                raw_dedicated
            } else {
                shared_system
            };
            result.push((
                name,
                luid_low,
                luid_high,
                vram,
                raw_dedicated,
                shared_system,
            ));
        }
        result
    }
}

#[cfg(target_os = "windows")]
pub fn build_static_gpus(
    build: u32,
    adapters: Vec<(String, u32, i32, u64, u64, u64)>,
) -> Vec<GpuInfo> {
    adapters
        .into_par_iter()
        .enumerate()
        .map(
            |(index, (name, luid_low, luid_high, vram, raw_dedicated, shared_system))| {
                let upper = name.to_ascii_uppercase();
                let vendor = if upper.contains("NVIDIA") {
                    "NVIDIA".to_string()
                } else if upper.contains("AMD") || upper.contains("RADEON") {
                    "AMD".to_string()
                } else if upper.contains("INTEL") {
                    "Intel".to_string()
                } else {
                    "Unknown".to_string()
                };
                let is_integrated =
                    classify_integrated_gpu(&name, &vendor, raw_dedicated, shared_system);
                let (driver_version, driver_date, pci_bus, pci_device, pci_function) =
                    gather_gpu_driver_info(&name, luid_low, luid_high)
                        .unwrap_or((None, None, None, None, None));
                let directx_version = Some(directx_from_build(build).to_string());
                GpuInfo {
                    index,
                    name,
                    vendor,
                    is_integrated,
                    driver_version,
                    driver_date,
                    directx_version,
                    vram_total_mb: vram / 1_048_576,
                    pci_bus,
                    pci_device,
                    pci_function,
                    luid_low,
                    luid_high,
                    ..Default::default()
                }
            },
        )
        .collect()
}

#[cfg(not(target_os = "windows"))]
pub fn build_static_gpus(
    _build: u32,
    _adapters: Vec<(String, u32, i32, u64, u64, u64)>,
) -> Vec<GpuInfo> {
    vec![]
}

#[cfg(target_os = "windows")]
#[repr(C)]
struct D3dkmtAdapterPerfdata {
    adapter_luid: [u8; 8],
    memory_frequency: u64,
    max_memory_frequency: u64,
    max_memory_frequency_state: u64,
    memory_bandwidth: u64,
    pcie_bandwidth: u64,
    fan_rpm: u32,
    power: u32,
    temperature: u32,
    power_state_override: u8,
    _pad: [u8; 3],
}

#[cfg(target_os = "windows")]
fn get_gpu_temp_d3dkmt(luid_low: u32, luid_high: i32) -> Option<f32> {
    use windows::Wdk::Graphics::Direct3D::*;
    use windows::Win32::Foundation::LUID;
    unsafe {
        let mut open_info = D3DKMT_OPENADAPTERFROMLUID {
            AdapterLuid: LUID {
                LowPart: luid_low,
                HighPart: luid_high,
            },
            hAdapter: 0,
        };
        if D3DKMTOpenAdapterFromLuid(&mut open_info).0 < 0 {
            return None;
        }
        let handle = open_info.hAdapter;

        let mut perf: D3dkmtAdapterPerfdata = std::mem::zeroed();
        let mut query = D3DKMT_QUERYADAPTERINFO {
            hAdapter: handle,
            Type: KMTQAITYPE_ADAPTERPERFDATA,
            pPrivateDriverData: &mut perf as *mut _ as *mut _,
            PrivateDriverDataSize: std::mem::size_of::<D3dkmtAdapterPerfdata>() as u32,
        };
        let ok = D3DKMTQueryAdapterInfo(&mut query).0 >= 0;
        let _ = D3DKMTCloseAdapter(&D3DKMT_CLOSEADAPTER { hAdapter: handle });

        if !ok || perf.temperature == 0 {
            return None;
        }
        Some(perf.temperature as f32 / 10.0)
    }
}

#[cfg(target_os = "windows")]
fn luid_to_pdh_prefix(luid_low: u32, luid_high: i32) -> String {
    format!("luid_0x{:08x}_0x{:08x}", luid_high as u32, luid_low).to_ascii_lowercase()
}

#[cfg(target_os = "windows")]
pub fn pdh_open_gpu_query() -> Option<(isize, isize)> {
    use windows::Win32::System::Performance::*;
    unsafe {
        let mut query = PDH_HQUERY(std::ptr::null_mut());
        if PdhOpenQueryW(windows::core::PCWSTR(std::ptr::null()), 0, &mut query) != 0 {
            return None;
        }
        let path = windows::core::w!(r"\GPU Engine(*)\Utilization Percentage");
        let mut counter = PDH_HCOUNTER(std::ptr::null_mut());
        if PdhAddEnglishCounterW(query, path, 0, &mut counter) != 0 {
            let _ = PdhCloseQuery(query);
            return None;
        }
        let _ = PdhCollectQueryData(query);
        Some((query.0 as isize, counter.0 as isize))
    }
}

#[cfg(not(target_os = "windows"))]
pub fn pdh_open_gpu_query() -> Option<(isize, isize)> {
    None
}

#[cfg(target_os = "windows")]
fn engine_type_from_name(instance: &str) -> Option<&'static str> {
    let pos = instance.find("engtype_")?;
    let engine = &instance[pos + "engtype_".len()..];

    if engine.starts_with("HighPriorityCompute") {
        Some("HighPriorityCompute")
    } else if engine.starts_with("HighPriority3D") {
        Some("HighPriority3D")
    } else if engine.starts_with("VideoDecode") {
        Some("VideoDecode")
    } else if engine.starts_with("VideoEncode") {
        Some("VideoEncode")
    } else if engine.starts_with("Compute") {
        Some("Compute")
    } else if engine.starts_with("Copy") {
        Some("Copy")
    } else if engine.starts_with("3D") {
        Some("3D")
    } else {
        None
    }
}

#[cfg(target_os = "windows")]
fn pdh_collect_gpu_usage(
    query_raw: isize,
    counter_raw: isize,
) -> HashMap<String, HashMap<String, f32>> {
    use windows::Win32::System::Performance::*;
    const PDH_CSTATUS_NEW_DATA: u32 = 0x00000001;
    const PDH_CSTATUS_VALID_DATA: u32 = 0x00000000;
    const PDH_MORE_DATA: u32 = 0x800007D2;
    unsafe {
        let query = PDH_HQUERY(query_raw as *mut _);
        let counter = PDH_HCOUNTER(counter_raw as *mut _);
        if PdhCollectQueryData(query) != 0 {
            return HashMap::new();
        }
        let mut buf_size: u32 = 0;
        let mut item_count: u32 = 0;
        let r = PdhGetFormattedCounterArrayW(
            counter,
            PDH_FMT_DOUBLE,
            &mut buf_size,
            &mut item_count,
            None,
        );
        if r != PDH_MORE_DATA && r != 0 {
            return HashMap::new();
        }
        if buf_size == 0 {
            return HashMap::new();
        }
        let typed_buf: Vec<PDH_FMT_COUNTERVALUE_ITEM_W>;
        let mut attempts = 0u32;
        loop {
            let item_capacity =
                (buf_size as usize).div_ceil(std::mem::size_of::<PDH_FMT_COUNTERVALUE_ITEM_W>());
            let mut buf: Vec<PDH_FMT_COUNTERVALUE_ITEM_W> = Vec::with_capacity(item_capacity);
            let r2 = PdhGetFormattedCounterArrayW(
                counter,
                PDH_FMT_DOUBLE,
                &mut buf_size,
                &mut item_count,
                Some(buf.as_mut_ptr()),
            );
            if r2 == PDH_MORE_DATA && attempts < 3 {
                attempts += 1;
                continue;
            }
            if r2 != 0 {
                return HashMap::new();
            }
            buf.set_len(item_count as usize);
            typed_buf = buf;
            break;
        }
        let mut result: HashMap<String, HashMap<String, f32>> = HashMap::new();
        for item in &typed_buf {
            if item.FmtValue.CStatus != PDH_CSTATUS_VALID_DATA
                && item.FmtValue.CStatus != PDH_CSTATUS_NEW_DATA
            {
                continue;
            }
            let ptr = item.szName.0;
            if ptr.is_null() {
                continue;
            }
            let max_u16_len = (buf_size as usize).div_ceil(std::mem::size_of::<u16>());
            let len = (0usize..max_u16_len)
                .take_while(|&j| *ptr.add(j) != 0)
                .count();
            let name = String::from_utf16_lossy(std::slice::from_raw_parts(ptr, len));
            let value = (item.FmtValue.Anonymous.doubleValue as f32).max(0.0);
            let luid_prefix = if let Some(start) = name.find("luid_") {
                if let Some(rel) = name[start..].find("_phys_") {
                    name[start..start + rel].to_ascii_lowercase()
                } else {
                    continue;
                }
            } else {
                continue;
            };
            let engine = match engine_type_from_name(&name) {
                Some(e) => e,
                None => continue,
            };
            let engine_map = result.entry(luid_prefix).or_default();
            *engine_map.entry(engine.to_string()).or_insert(0.0_f32) += value;
        }
        for engine_map in result.values_mut() {
            for v in engine_map.values_mut() {
                *v = v.min(100.0);
            }
        }
        result
    }
}

#[cfg(target_os = "windows")]
pub fn gather_gpu_live(
    gpus: &[GpuInfo],
    pdh_query: isize,
    pdh_counter: isize,
) -> Vec<LiveGpuMetrics> {
    use windows::Win32::Foundation::LUID;
    use windows::Win32::Graphics::Dxgi::{
        CreateDXGIFactory1, DXGI_MEMORY_SEGMENT_GROUP_LOCAL, DXGI_MEMORY_SEGMENT_GROUP_NON_LOCAL,
        DXGI_QUERY_VIDEO_MEMORY_INFO, IDXGIAdapter1, IDXGIAdapter3, IDXGIFactory1, IDXGIFactory4,
    };
    use windows::core::Interface;

    let pdh_open = pdh_query != 0;
    let usage_by_luid: HashMap<String, HashMap<String, f32>> = if pdh_open {
        pdh_collect_gpu_usage(pdh_query, pdh_counter)
    } else {
        HashMap::new()
    };

    let temperatures: Vec<Option<u32>> = gpus
        .par_iter()
        .map(|gpu| get_gpu_temp_d3dkmt(gpu.luid_low, gpu.luid_high).map(|t| t.round() as u32))
        .collect();

    let factory4_opt: Option<IDXGIFactory4> = unsafe {
        CreateDXGIFactory1::<IDXGIFactory1>()
            .ok()
            .and_then(|f| f.cast::<IDXGIFactory4>().ok())
    };

    gpus.iter()
        .zip(temperatures)
        .map(|(gpu, temperature_c)| {
            let (low, high) = (gpu.luid_low, gpu.luid_high);
            let luid_prefix = luid_to_pdh_prefix(low, high);
            let engines = if pdh_open {
                usage_by_luid.get(&luid_prefix).cloned().unwrap_or_default()
            } else {
                HashMap::new()
            };
            let get_eng = |key: &str| engines.get(key).copied().unwrap_or(0.0) as u32;

            let (vram_used_mb, vram_reserved_mb, vram_shared_mb) = match &factory4_opt {
                Some(factory4) => unsafe {
                    let luid = LUID {
                        LowPart: low,
                        HighPart: high,
                    };
                    match factory4
                        .EnumAdapterByLuid::<IDXGIAdapter1>(luid)
                        .ok()
                        .and_then(|a1| a1.cast::<IDXGIAdapter3>().ok())
                    {
                        Some(adapter3) => {
                            let mut local: DXGI_QUERY_VIDEO_MEMORY_INFO = std::mem::zeroed();
                            let mut non_local: DXGI_QUERY_VIDEO_MEMORY_INFO = std::mem::zeroed();
                            let local_ok = adapter3
                                .QueryVideoMemoryInfo(
                                    0,
                                    DXGI_MEMORY_SEGMENT_GROUP_LOCAL,
                                    &mut local,
                                )
                                .is_ok();
                            let non_local_ok = adapter3
                                .QueryVideoMemoryInfo(
                                    0,
                                    DXGI_MEMORY_SEGMENT_GROUP_NON_LOCAL,
                                    &mut non_local,
                                )
                                .is_ok();
                            let used = if local_ok {
                                local.CurrentUsage / 1_048_576
                            } else {
                                0
                            };
                            let reserved = if local_ok {
                                local.CurrentReservation / 1_048_576
                            } else {
                                0
                            };
                            let shared = if non_local_ok {
                                non_local.CurrentUsage / 1_048_576
                            } else {
                                0
                            };
                            (used, reserved, shared)
                        }
                        None => (0, 0, 0),
                    }
                },
                None => (0, 0, 0),
            };

            LiveGpuMetrics {
                index: gpu.index,
                vram_total_mb: gpu.vram_total_mb,
                vram_used_mb,
                vram_shared_mb,
                vram_reserved_mb,
                temperature_c,
                power_w: None,
                util_3d: get_eng("3D"),
                util_copy: get_eng("Copy"),
                util_encode: get_eng("VideoEncode"),
                util_decode: get_eng("VideoDecode"),
                util_high_priority_3d: get_eng("HighPriority3D"),
                util_high_priority_compute: get_eng("HighPriorityCompute"),
                processes: vec![],
            }
        })
        .collect()
}

#[cfg(not(target_os = "windows"))]
pub fn gather_gpu_live(
    _gpus: &[GpuInfo],
    _pdh_query: isize,
    _pdh_counter: isize,
) -> Vec<LiveGpuMetrics> {
    vec![]
}
