use std::collections::HashMap;

use rayon::prelude::*;

use crate::system_info::types::{GpuInfo, LiveGpuMetrics};

#[cfg(target_os = "windows")]
pub(crate) struct DxgiAdapterSnapshot {
    name: String,
    luid_low: u32,
    luid_high: i32,
    raw_dedicated: u64,
    shared_system: u64,
    directx_version: Option<String>,
}

#[cfg(not(target_os = "windows"))]
pub(crate) struct DxgiAdapterSnapshot {
    name: String,
    luid_low: u32,
    luid_high: i32,
    raw_dedicated: u64,
    shared_system: u64,
    directx_version: Option<String>,
}

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
fn read_pci_location(
    device_info_set: windows::Win32::Devices::DeviceAndDriverInstallation::HDEVINFO,
    device_info: &windows::Win32::Devices::DeviceAndDriverInstallation::SP_DEVINFO_DATA,
) -> (Option<u32>, Option<u32>, Option<u32>) {
    use std::mem::size_of;

    use windows::Win32::Devices::DeviceAndDriverInstallation::{
        SETUP_DI_REGISTRY_PROPERTY, SPDRP_ADDRESS, SPDRP_BUSNUMBER,
        SetupDiGetDeviceRegistryPropertyW,
    };

    fn read_u32_property(
        device_info_set: windows::Win32::Devices::DeviceAndDriverInstallation::HDEVINFO,
        device_info: &windows::Win32::Devices::DeviceAndDriverInstallation::SP_DEVINFO_DATA,
        property: SETUP_DI_REGISTRY_PROPERTY,
    ) -> Option<u32> {
        let mut property_type = 0u32;
        let mut buffer = [0u8; size_of::<u32>()];

        unsafe {
            SetupDiGetDeviceRegistryPropertyW(
                device_info_set,
                device_info,
                property,
                Some(&mut property_type),
                Some(&mut buffer),
                None,
            )
            .ok()?;
        }

        Some(u32::from_le_bytes(buffer))
    }

    let bus = read_u32_property(device_info_set, device_info, SPDRP_BUSNUMBER);
    let address = read_u32_property(device_info_set, device_info, SPDRP_ADDRESS);
    let device = address.map(|value| (value >> 16) & 0xFFFF);
    let function = address.map(|value| value & 0xFFFF);

    (bus, device, function)
}

#[cfg(target_os = "windows")]
fn directx_version_for_adapter(
    adapter: &windows::Win32::Graphics::Dxgi::IDXGIAdapter1,
) -> Option<String> {
    use windows::Win32::Foundation::HMODULE;
    use windows::Win32::Graphics::Direct3D::{
        D3D_DRIVER_TYPE_UNKNOWN, D3D_FEATURE_LEVEL, D3D_FEATURE_LEVEL_9_1, D3D_FEATURE_LEVEL_9_2,
        D3D_FEATURE_LEVEL_9_3, D3D_FEATURE_LEVEL_10_0, D3D_FEATURE_LEVEL_10_1,
        D3D_FEATURE_LEVEL_11_0, D3D_FEATURE_LEVEL_11_1, D3D_FEATURE_LEVEL_12_0,
        D3D_FEATURE_LEVEL_12_1, D3D_FEATURE_LEVEL_12_2,
    };
    use windows::Win32::Graphics::Direct3D11::{
        D3D11_CREATE_DEVICE_FLAG, D3D11_SDK_VERSION, D3D11CreateDevice,
    };
    use windows::Win32::Graphics::Direct3D12::{D3D12CreateDevice, ID3D12Device};
    use windows::Win32::Graphics::Dxgi::IDXGIAdapter;
    use windows::core::Interface;

    let adapter = adapter.cast::<IDXGIAdapter>().ok()?;

    for feature_level in [
        D3D_FEATURE_LEVEL_12_2,
        D3D_FEATURE_LEVEL_12_1,
        D3D_FEATURE_LEVEL_12_0,
    ] {
        let mut device: Option<ID3D12Device> = None;
        if unsafe { D3D12CreateDevice(&adapter, feature_level, &mut device) }.is_ok() {
            return Some("12.x".to_string());
        }
    }

    fn probe_d3d11_feature_level(
        adapter: &IDXGIAdapter,
        feature_levels: &[D3D_FEATURE_LEVEL],
    ) -> Option<D3D_FEATURE_LEVEL> {
        let mut selected = D3D_FEATURE_LEVEL(0);
        unsafe {
            D3D11CreateDevice(
                adapter,
                D3D_DRIVER_TYPE_UNKNOWN,
                HMODULE::default(),
                D3D11_CREATE_DEVICE_FLAG(0),
                Some(feature_levels),
                D3D11_SDK_VERSION,
                None,
                Some(&mut selected),
                None,
            )
            .ok()?;
        }
        Some(selected)
    }

    let feature_level = probe_d3d11_feature_level(
        &adapter,
        &[
            D3D_FEATURE_LEVEL_11_1,
            D3D_FEATURE_LEVEL_11_0,
            D3D_FEATURE_LEVEL_10_1,
            D3D_FEATURE_LEVEL_10_0,
            D3D_FEATURE_LEVEL_9_3,
            D3D_FEATURE_LEVEL_9_2,
            D3D_FEATURE_LEVEL_9_1,
        ],
    )
    .or_else(|| {
        probe_d3d11_feature_level(
            &adapter,
            &[
                D3D_FEATURE_LEVEL_11_0,
                D3D_FEATURE_LEVEL_10_1,
                D3D_FEATURE_LEVEL_10_0,
                D3D_FEATURE_LEVEL_9_3,
                D3D_FEATURE_LEVEL_9_2,
                D3D_FEATURE_LEVEL_9_1,
            ],
        )
    })?;

    let version = if feature_level == D3D_FEATURE_LEVEL_11_1 {
        "11.1"
    } else if feature_level == D3D_FEATURE_LEVEL_11_0 {
        "11.0"
    } else if feature_level == D3D_FEATURE_LEVEL_10_1 {
        "10.1"
    } else if feature_level == D3D_FEATURE_LEVEL_10_0 {
        "10.0"
    } else if feature_level == D3D_FEATURE_LEVEL_9_3 {
        "9.3"
    } else if feature_level == D3D_FEATURE_LEVEL_9_2 {
        "9.2"
    } else {
        "9.1"
    };

    Some(version.to_string())
}

#[cfg(target_os = "windows")]
#[derive(Debug, Clone, Copy)]
pub enum PdhGpuUsageError {
    CollectQueryData(u32),
    GetFormattedCounterArray(u32),
}

#[cfg(target_os = "windows")]
impl PdhGpuUsageError {
    pub fn status_code(self) -> u32 {
        match self {
            Self::CollectQueryData(code) | Self::GetFormattedCounterArray(code) => code,
        }
    }
}

#[cfg(target_os = "windows")]
#[allow(clippy::type_complexity)]
fn gather_gpu_driver_info(
    luid_low: u32,
    luid_high: i32,
) -> Option<(
    Option<String>,
    Option<String>,
    Option<u32>,
    Option<u32>,
    Option<u32>,
)> {
    use std::mem::size_of;
    use std::ptr::read_unaligned;

    use windows::Win32::Devices::DeviceAndDriverInstallation::{
        CM_Get_Device_ID_Size, CM_Get_Device_IDW, CR_SUCCESS, DIGCF_PRESENT, GUID_DEVCLASS_DISPLAY,
        SP_DEVINFO_DATA, SetupDiDestroyDeviceInfoList, SetupDiEnumDeviceInfo, SetupDiGetClassDevsW,
        SetupDiGetDevicePropertyW,
    };
    use windows::Win32::Devices::Properties::{DEVPROP_TYPE_UINT64, DEVPROPTYPE};
    use windows::Win32::Foundation::{DEVPROPKEY, LUID};
    use windows::core::{GUID, PCWSTR};
    use winreg::RegKey as WinRegKey;
    use winreg::enums::HKEY_LOCAL_MACHINE;

    const DEVPKEY_DISPLAY_ADAPTER_LUID: DEVPROPKEY = DEVPROPKEY {
        fmtid: GUID::from_u128(0x60b193cb_5276_4d0f_96fc_f173abad3ec6),
        pid: 2,
    };

    let (target_device_instance_id, pci_bus, pci_device, pci_function) = unsafe {
        let device_info_set = SetupDiGetClassDevsW(
            Some(&GUID_DEVCLASS_DISPLAY),
            PCWSTR::null(),
            None,
            DIGCF_PRESENT,
        )
        .ok()?;

        let mut match_data = None;
        let mut index = 0;

        loop {
            let mut device_info = SP_DEVINFO_DATA {
                cbSize: size_of::<SP_DEVINFO_DATA>() as u32,
                ..Default::default()
            };

            if SetupDiEnumDeviceInfo(device_info_set, index, &mut device_info).is_err() {
                break;
            }
            index += 1;

            let mut property_type = DEVPROPTYPE::default();
            let mut luid_buffer = [0u8; size_of::<LUID>()];
            if SetupDiGetDevicePropertyW(
                device_info_set,
                &device_info,
                &DEVPKEY_DISPLAY_ADAPTER_LUID,
                &mut property_type,
                Some(&mut luid_buffer),
                None,
                0,
            )
            .is_err()
                || property_type != DEVPROP_TYPE_UINT64
            {
                continue;
            }

            let adapter_luid = read_unaligned(luid_buffer.as_ptr() as *const LUID);
            if adapter_luid.LowPart != luid_low || adapter_luid.HighPart != luid_high {
                continue;
            }

            let (pci_bus, pci_device, pci_function) =
                read_pci_location(device_info_set, &device_info);

            let mut required_len = 0;
            if CM_Get_Device_ID_Size(&mut required_len, device_info.DevInst, 0) != CR_SUCCESS {
                match_data = Some((None, pci_bus, pci_device, pci_function));
                break;
            }

            let mut device_id = vec![0u16; required_len as usize + 1];
            if CM_Get_Device_IDW(device_info.DevInst, &mut device_id, 0) == CR_SUCCESS {
                let end = device_id
                    .iter()
                    .position(|value| *value == 0)
                    .unwrap_or(device_id.len());
                match_data = Some((
                    Some(String::from_utf16_lossy(&device_id[..end])),
                    pci_bus,
                    pci_device,
                    pci_function,
                ));
            } else {
                match_data = Some((None, pci_bus, pci_device, pci_function));
            }
            break;
        }

        let _ = SetupDiDestroyDeviceInfoList(device_info_set);
        match_data
    }?;

    let hklm = WinRegKey::predef(HKEY_LOCAL_MACHINE);
    let Some(target_device_instance_id) = target_device_instance_id else {
        return Some((None, None, pci_bus, pci_device, pci_function));
    };
    let class_key = match hklm.open_subkey(
        r"SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}",
    ) {
        Ok(key) => key,
        Err(_) => return Some((None, None, pci_bus, pci_device, pci_function)),
    };

    for key_name in class_key.enum_keys().flatten() {
        let subkey = match class_key.open_subkey(&key_name) {
            Ok(k) => k,
            Err(_) => continue,
        };

        if !matches!(
            subkey.get_value::<String, _>("DeviceInstanceID"),
            Ok(value) if value.eq_ignore_ascii_case(&target_device_instance_id)
        ) {
            continue;
        }
        let driver_version: Option<String> = subkey.get_value("DriverVersion").ok();
        let driver_date: Option<String> = subkey
            .get_value::<String, _>("DriverDate")
            .ok()
            .and_then(|s| normalise_driver_date(&s));
        return Some((
            driver_version,
            driver_date,
            pci_bus,
            pci_device,
            pci_function,
        ));
    }

    Some((None, None, pci_bus, pci_device, pci_function))
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

    if raw_dedicated_bytes >= 2 * GB {
        return false;
    }
    if intel_integrated || generic_amd_integrated {
        return true;
    }
    if shared_system_bytes > 0 && raw_dedicated_bytes <= 512 * MB {
        return true;
    }

    false
}

#[cfg(not(target_os = "windows"))]
pub fn enumerate_dxgi_adapters() -> Vec<DxgiAdapterSnapshot> {
    vec![]
}

#[cfg(target_os = "windows")]
pub fn enumerate_dxgi_adapters() -> Vec<DxgiAdapterSnapshot> {
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
            if desc.Flags & (DXGI_ADAPTER_FLAG_SOFTWARE.0 as u32) != 0 {
                continue;
            }

            let null_pos = desc.Description.iter().position(|&c| c == 0).unwrap_or(128);
            let name = String::from_utf16_lossy(&desc.Description[..null_pos]);
            let raw_dedicated = desc.DedicatedVideoMemory as u64;
            let shared_system = desc.SharedSystemMemory as u64;
            let directx_version = directx_version_for_adapter(&adapter);
            result.push(DxgiAdapterSnapshot {
                name,
                luid_low,
                luid_high,
                raw_dedicated,
                shared_system,
                directx_version,
            });
        }
        result
    }
}

#[cfg(target_os = "windows")]
pub fn build_static_gpus(adapters: Vec<DxgiAdapterSnapshot>) -> Vec<GpuInfo> {
    adapters
        .into_par_iter()
        .enumerate()
        .map(|(index, adapter)| {
            let DxgiAdapterSnapshot {
                name,
                luid_low,
                luid_high,
                raw_dedicated,
                shared_system,
                directx_version,
            } = adapter;

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
                gather_gpu_driver_info(luid_low, luid_high)
                    .unwrap_or((None, None, None, None, None));
            GpuInfo {
                index,
                name,
                vendor,
                is_integrated,
                driver_version,
                driver_date,
                directx_version,
                // Preserve legacy `vram_total_mb` meaning as dedicated VRAM only.
                vram_total_mb: raw_dedicated / 1_048_576,
                dedicated_vram_mb: raw_dedicated / 1_048_576,
                shared_system_mb: shared_system / 1_048_576,
                pci_bus,
                pci_device,
                pci_function,
                luid_low,
                luid_high,
                ..Default::default()
            }
        })
        .collect()
}

#[cfg(not(target_os = "windows"))]
pub fn build_static_gpus(_adapters: Vec<DxgiAdapterSnapshot>) -> Vec<GpuInfo> {
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
pub fn pdh_open_gpu_query() -> Option<(isize, isize, isize, isize)> {
    use windows::Win32::System::Performance::*;
    unsafe {
        let mut query = PDH_HQUERY(std::ptr::null_mut());
        if PdhOpenQueryW(windows::core::PCWSTR(std::ptr::null()), 0, &mut query) != 0 {
            return None;
        }
        let engine_path = windows::core::w!(r"\GPU Engine(*)\Utilization Percentage");
        let dedicated_path = windows::core::w!(r"\GPU Adapter Memory(*)\Dedicated Usage");
        let shared_path = windows::core::w!(r"\GPU Adapter Memory(*)\Shared Usage");

        let mut engine_counter = PDH_HCOUNTER(std::ptr::null_mut());
        if PdhAddEnglishCounterW(query, engine_path, 0, &mut engine_counter) != 0 {
            let _ = PdhCloseQuery(query);
            return None;
        }
        let mut dedicated_counter = PDH_HCOUNTER(std::ptr::null_mut());
        if PdhAddEnglishCounterW(query, dedicated_path, 0, &mut dedicated_counter) != 0 {
            dedicated_counter = PDH_HCOUNTER(std::ptr::null_mut());
        }
        let mut shared_counter = PDH_HCOUNTER(std::ptr::null_mut());
        if PdhAddEnglishCounterW(query, shared_path, 0, &mut shared_counter) != 0 {
            shared_counter = PDH_HCOUNTER(std::ptr::null_mut());
        }
        let _ = PdhCollectQueryData(query);
        Some((
            query.0 as isize,
            engine_counter.0 as isize,
            dedicated_counter.0 as isize,
            shared_counter.0 as isize,
        ))
    }
}

#[cfg(target_os = "windows")]
pub fn pdh_close_gpu_query(handles: (isize, isize, isize, isize)) {
    use windows::Win32::System::Performance::{PDH_HQUERY, PdhCloseQuery};

    unsafe {
        let _ = PdhCloseQuery(PDH_HQUERY(handles.0 as *mut _));
    }
}

#[cfg(not(target_os = "windows"))]
pub fn pdh_close_gpu_query(_handles: (isize, isize, isize, isize)) {}

#[cfg(not(target_os = "windows"))]
pub fn pdh_open_gpu_query() -> Option<(isize, isize, isize, isize)> {
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
fn pdh_luid_prefix_from_name(name: &str) -> Option<String> {
    let start = name.find("luid_")?;
    let rel = name[start..].find("_phys_")?;
    Some(name[start..start + rel].to_ascii_lowercase())
}

#[cfg(target_os = "windows")]
type GpuUsageByLuid = HashMap<String, HashMap<String, f32>>;

#[cfg(target_os = "windows")]
type GpuMemoryByLuid = HashMap<String, u64>;

#[cfg(target_os = "windows")]
fn pdh_collect_gpu_sample(query_raw: isize) -> Result<(), PdhGpuUsageError> {
    use windows::Win32::System::Performance::*;

    unsafe {
        let query = PDH_HQUERY(query_raw as *mut _);
        let collect_status = PdhCollectQueryData(query);
        if collect_status != 0 {
            return Err(PdhGpuUsageError::CollectQueryData(collect_status));
        }
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn pdh_collect_gpu_usage(counter_raw: isize) -> Result<GpuUsageByLuid, PdhGpuUsageError> {
    use windows::Win32::System::Performance::*;

    const PDH_CSTATUS_NEW_DATA: u32 = 0x00000001;
    const PDH_CSTATUS_VALID_DATA: u32 = 0x00000000;
    const PDH_MORE_DATA: u32 = 0x800007D2;

    if counter_raw == 0 {
        return Ok(HashMap::new());
    }

    unsafe {
        let counter = PDH_HCOUNTER(counter_raw as *mut _);
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
            return Err(PdhGpuUsageError::GetFormattedCounterArray(r));
        }
        if buf_size == 0 {
            return Ok(HashMap::new());
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
                return Err(PdhGpuUsageError::GetFormattedCounterArray(r2));
            }
            buf.set_len(item_count as usize);
            typed_buf = buf;
            break;
        }
        let mut result: GpuUsageByLuid = HashMap::new();
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
            let Some(luid_prefix) = pdh_luid_prefix_from_name(&name) else {
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
        Ok(result)
    }
}

#[cfg(target_os = "windows")]
fn pdh_collect_gpu_memory(counter_raw: isize) -> Result<GpuMemoryByLuid, PdhGpuUsageError> {
    use windows::Win32::System::Performance::*;

    const PDH_CSTATUS_NEW_DATA: u32 = 0x00000001;
    const PDH_CSTATUS_VALID_DATA: u32 = 0x00000000;
    const PDH_MORE_DATA: u32 = 0x800007D2;

    if counter_raw == 0 {
        return Ok(HashMap::new());
    }

    unsafe {
        let counter = PDH_HCOUNTER(counter_raw as *mut _);
        let mut buf_size: u32 = 0;
        let mut item_count: u32 = 0;
        let r = PdhGetFormattedCounterArrayW(
            counter,
            PDH_FMT_LARGE,
            &mut buf_size,
            &mut item_count,
            None,
        );
        if r != PDH_MORE_DATA && r != 0 {
            return Err(PdhGpuUsageError::GetFormattedCounterArray(r));
        }
        if buf_size == 0 {
            return Ok(HashMap::new());
        }

        let typed_buf: Vec<PDH_FMT_COUNTERVALUE_ITEM_W>;
        let mut attempts = 0u32;
        loop {
            let item_capacity =
                (buf_size as usize).div_ceil(std::mem::size_of::<PDH_FMT_COUNTERVALUE_ITEM_W>());
            let mut buf: Vec<PDH_FMT_COUNTERVALUE_ITEM_W> = Vec::with_capacity(item_capacity);
            let r2 = PdhGetFormattedCounterArrayW(
                counter,
                PDH_FMT_LARGE,
                &mut buf_size,
                &mut item_count,
                Some(buf.as_mut_ptr()),
            );
            if r2 == PDH_MORE_DATA && attempts < 3 {
                attempts += 1;
                continue;
            }
            if r2 != 0 {
                return Err(PdhGpuUsageError::GetFormattedCounterArray(r2));
            }
            buf.set_len(item_count as usize);
            typed_buf = buf;
            break;
        }

        let mut result: GpuMemoryByLuid = HashMap::new();
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
            let Some(luid_prefix) = pdh_luid_prefix_from_name(&name) else {
                continue;
            };
            let value = item.FmtValue.Anonymous.largeValue.max(0) as u64;
            *result.entry(luid_prefix).or_insert(0) += value;
        }

        Ok(result)
    }
}

#[cfg(target_os = "windows")]
pub fn gather_gpu_live(
    gpus: &[GpuInfo],
    pdh_query: isize,
    pdh_engine_counter: isize,
    pdh_dedicated_counter: isize,
    pdh_shared_counter: isize,
) -> Result<Vec<LiveGpuMetrics>, PdhGpuUsageError> {
    let pdh_open = pdh_query != 0;
    let (usage_by_luid, dedicated_by_luid, shared_by_luid): (
        GpuUsageByLuid,
        GpuMemoryByLuid,
        GpuMemoryByLuid,
    ) = if pdh_open {
        pdh_collect_gpu_sample(pdh_query)?;
        let usage_by_luid = pdh_collect_gpu_usage(pdh_engine_counter)?;
        let dedicated_by_luid = match pdh_collect_gpu_memory(pdh_dedicated_counter) {
            Ok(memory) => memory,
            Err(error) => {
                log::warn!(
                    "gpu dedicated-memory PDH query failed with status code {}",
                    error.status_code()
                );
                HashMap::new()
            }
        };
        let shared_by_luid = match pdh_collect_gpu_memory(pdh_shared_counter) {
            Ok(memory) => memory,
            Err(error) => {
                log::warn!(
                    "gpu shared-memory PDH query failed with status code {}",
                    error.status_code()
                );
                HashMap::new()
            }
        };

        (usage_by_luid, dedicated_by_luid, shared_by_luid)
    } else {
        (HashMap::new(), HashMap::new(), HashMap::new())
    };

    let temperatures: Vec<Option<u32>> = gpus
        .par_iter()
        .map(|gpu| get_gpu_temp_d3dkmt(gpu.luid_low, gpu.luid_high).map(|t| t.round() as u32))
        .collect();

    Ok(gpus
        .iter()
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
            let dedicated_used_mb =
                dedicated_by_luid.get(&luid_prefix).copied().unwrap_or(0) / 1_048_576;
            let shared_used_mb = shared_by_luid.get(&luid_prefix).copied().unwrap_or(0) / 1_048_576;

            LiveGpuMetrics {
                index: gpu.index,
                vram_total_mb: gpu.vram_total_mb,
                dedicated_vram_mb: gpu.dedicated_vram_mb,
                shared_system_mb: gpu.shared_system_mb,
                vram_used_mb: dedicated_used_mb,
                vram_shared_mb: shared_used_mb,
                vram_reserved_mb: 0,
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
        .collect())
}

#[cfg(not(target_os = "windows"))]
pub fn gather_gpu_live(
    _gpus: &[GpuInfo],
    _pdh_query: isize,
    _pdh_engine_counter: isize,
    _pdh_dedicated_counter: isize,
    _pdh_shared_counter: isize,
) -> Vec<LiveGpuMetrics> {
    vec![]
}
