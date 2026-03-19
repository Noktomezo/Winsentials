use std::collections::HashMap;
use std::sync::{Mutex, OnceLock, RwLock};
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use sysinfo::{CpuRefreshKind, DiskKind, Disks, MemoryRefreshKind, Networks, RefreshKind, System};

use crate::error::AppError;
use crate::registry::{Hive, RegKey};

// ─── Static structs ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaticSystemInfo {
    pub windows: WindowsInfo,
    pub cpu: CpuInfo,
    pub ram: RamInfo,
    pub network_adapters: Vec<NetworkAdapterInfo>,
    pub gpus: Vec<GpuInfo>,
    pub motherboard: MotherboardInfo,
    pub disks: Vec<DiskInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RamInfo {
    pub total_bytes: u64,
    pub speed_mhz: Option<u32>,
    pub used_slots: u32,
    pub total_slots: u32,
    pub form_factor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowsInfo {
    pub product_name: String,
    pub display_version: String,
    pub build: u32,
    pub ubr: u32,
    pub hostname: String,
    pub username: String,
    pub architecture: String,
    pub activation_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub model: String,
    pub physical_cores: u32,
    pub logical_cores: u32,
    pub base_freq_mhz: u64,
    pub sockets: u32,
    pub virtualization: bool,
    pub l1_cache_kb: Option<u32>,
    pub l2_cache_kb: Option<u32>,
    pub l3_cache_kb: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GpuProcess {
    pub pid: u32,
    pub name: String,
    pub dedicated_mem_mb: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GpuInfo {
    pub index: usize,
    pub name: String,
    pub vendor: String,
    pub is_integrated: bool,
    pub driver_version: Option<String>,
    pub driver_date: Option<String>,
    pub directx_version: Option<String>,
    pub vram_total_mb: u64,
    pub vram_used_mb: u64,
    pub vram_shared_mb: u64,
    pub vram_reserved_mb: u64,
    pub temperature_c: Option<u32>,
    pub power_w: Option<u32>,
    pub util_3d: u32,
    pub util_copy: u32,
    pub util_encode: u32,
    pub util_decode: u32,
    pub util_high_priority_3d: u32,
    pub util_high_priority_compute: u32,
    pub processes: Vec<GpuProcess>,
    /// PCI bus / device / function from the Windows device registry.
    pub pci_bus: Option<u32>,
    pub pci_device: Option<u32>,
    pub pci_function: Option<u32>,
    /// DXGI adapter LUID — internal only, not sent to frontend
    #[serde(skip)]
    pub luid_low: u32,
    #[serde(skip)]
    pub luid_high: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotherboardInfo {
    pub manufacturer: String,
    pub product: String,
    pub bios_vendor: String,
    pub bios_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub kind: String,
    pub file_system: String,
    pub volume_label: Option<String>,
    pub is_system_disk: bool,
    pub has_pagefile: bool,
    pub type_label: String,
}

// ─── Live structs ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveSystemInfo {
    pub cpu_usage_percent: f32,
    pub cpu_per_core: Vec<f32>,
    pub cpu_current_freq_mhz: u64,
    pub cpu_process_count: u32,
    pub cpu_thread_count: u32,
    pub cpu_handle_count: u32,
    pub cpu_uptime_secs: u64,
    pub ram_used_bytes: u64,
    pub ram_available_bytes: u64,
    pub ram_committed_bytes: u64,
    pub ram_commit_limit_bytes: u64,
    pub ram_cached_bytes: u64,
    pub ram_compressed_bytes: u64,
    pub ram_paged_pool_bytes: u64,
    pub ram_nonpaged_pool_bytes: u64,
    pub disks: Vec<DiskLiveInfo>,
    pub network: Vec<NetworkIfaceStats>,
    pub gpus: Vec<GpuInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskLiveInfo {
    pub mount_point: String,
    pub active_time_percent: u32,
    pub avg_response_ms: f64,
    pub read_bytes_per_sec: u64,
    pub write_bytes_per_sec: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkIfaceStats {
    pub name: String,
    pub rx_bytes_per_sec: u64,
    pub tx_bytes_per_sec: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkAdapterInfo {
    pub index: usize,
    pub name: String,
    pub adapter_description: String,
    pub dns_name: Option<String>,
    pub connection_type: String,
    pub ipv4_addresses: Vec<String>,
    pub ipv6_addresses: Vec<String>,
    pub is_wifi: bool,
    pub ssid: Option<String>,
    pub signal_percent: Option<u32>,
}

// ─── Managed state ────────────────────────────────────────────────────────────

pub struct SystemInfoState {
    pub system: Mutex<System>,
    pub networks: Mutex<Networks>,
    pub prev_net: Mutex<Option<PreviousNetSnapshot>>,
    pub static_cache: Mutex<Option<StaticSystemInfo>>,
    pub live_cache: Mutex<Option<LiveSystemInfo>>,
}

#[derive(Debug, Clone)]
pub struct PreviousNetSnapshot {
    pub captured_at: Instant,
    pub totals: HashMap<String, (u64, u64)>,
}

impl Default for SystemInfoState {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemInfoState {
    pub fn new() -> Self {
        let system = System::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything()),
        );
        let networks = Networks::new_with_refreshed_list();
        Self {
            system: Mutex::new(system),
            networks: Mutex::new(networks),
            prev_net: Mutex::new(None),
            static_cache: Mutex::new(None),
            live_cache: Mutex::new(None),
        }
    }
}

// ─── Registry keys ────────────────────────────────────────────────────────────

const WIN_VERSION_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SOFTWARE\Microsoft\Windows NT\CurrentVersion",
};

// ─── Static info gathering ────────────────────────────────────────────────────

#[cfg(target_os = "windows")]
fn gather_windows_info() -> Result<WindowsInfo, AppError> {
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

    // ProductName says "Windows 10 Pro" even on Windows 11 on many machines.
    // Build >= 22000 unambiguously identifies Windows 11.
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
        activation_status: "unknown".to_string(), // filled in by gather_static_info after WMI
    })
}

#[cfg(not(target_os = "windows"))]
fn gather_windows_info() -> Result<WindowsInfo, AppError> {
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

fn gather_cpu_info(system: &System) -> CpuInfo {
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

/// Queries the volume label (user-visible name) for a mount point via `GetVolumeInformationW`.
/// Returns `None` on non-Windows or when the label is empty.
#[cfg(target_os = "windows")]
fn get_volume_label(mount_point: &str) -> Option<String> {
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
fn get_volume_label(_mount_point: &str) -> Option<String> {
    None
}

#[derive(Debug, Deserialize)]
struct RawDiskMetadataInfo {
    mount_point: String,
    is_system_disk: bool,
    has_pagefile: bool,
    type_label: String,
}

#[derive(Debug, Deserialize)]
struct RawDiskLiveInfo {
    mount_point: String,
    active_time_percent: u32,
    avg_response_ms: f64,
    read_bytes_per_sec: u64,
    write_bytes_per_sec: u64,
}

const DISK_LIVE_CACHE_INTERVAL: Duration = Duration::from_secs(3);
type DiskLiveCache = RwLock<Option<(Instant, HashMap<String, DiskLiveInfo>)>>;

fn disk_live_cache() -> &'static DiskLiveCache {
    static DISK_LIVE_CACHE: OnceLock<DiskLiveCache> = OnceLock::new();
    DISK_LIVE_CACHE.get_or_init(|| RwLock::new(None))
}

#[cfg(target_os = "windows")]
fn gather_disk_metadata() -> HashMap<String, RawDiskMetadataInfo> {
    let script = r#"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$systemDrive = (Get-CimInstance Win32_OperatingSystem -ErrorAction SilentlyContinue).SystemDrive
$result = @()

foreach ($volume in Get-CimInstance Win32_LogicalDisk -ErrorAction SilentlyContinue | Where-Object { $_.DriveType -eq 3 -and $_.DeviceID }) {
  $mount = "$($volume.DeviceID)\"
  $partition = Get-Partition -DriveLetter $volume.DeviceID.TrimEnd(':') -ErrorAction SilentlyContinue | Select-Object -First 1
  $disk = if ($partition) { Get-Disk -Number $partition.DiskNumber -ErrorAction SilentlyContinue | Select-Object -First 1 } else { $null }
  $physical = if ($disk) { Get-PhysicalDisk -ErrorAction SilentlyContinue | Where-Object { $_.DeviceId -eq $disk.Number } | Select-Object -First 1 } else { $null }
  $mediaType = if ($physical -and $physical.MediaType) { [string]$physical.MediaType } elseif ($disk -and $disk.MediaType) { [string]$disk.MediaType } else { '' }
  $rawBusType = if ($physical -and $physical.BusType) { [string]$physical.BusType } elseif ($disk -and $disk.BusType) { [string]$disk.BusType } else { '' }
  $busType = switch -Regex ($rawBusType) {
    '^NVMe$' { 'NVMe'; break }
    '^SATA$' { 'SATA'; break }
    '^RAID$' { 'RAID'; break }
    '^USB$' { 'USB'; break }
    default { if ($rawBusType) { $rawBusType.ToUpperInvariant() } else { '' } }
  }
  $typeLabel = if ($mediaType -match 'SSD') {
    if ($busType) { "SSD ($busType)" } else { 'SSD' }
  } elseif ($mediaType -match 'HDD') {
    if ($busType) { "HDD ($busType)" } else { 'HDD' }
  } else {
    if ($disk -and $disk.BusType -match 'NVMe|SATA|RAID|USB') {
      "SSD ($busType)"
    } elseif ($busType) {
      $busType
    } else {
      'Unknown'
    }
  }

  $result += [pscustomobject]@{
    mount_point = $mount
    is_system_disk = $volume.DeviceID -eq $systemDrive
    has_pagefile = Test-Path (Join-Path $mount 'pagefile.sys')
    type_label = $typeLabel
  }
}

@($result) | ConvertTo-Json -Depth 3 -Compress
"#;

    let Ok(output) = crate::shell::run_powershell(script) else {
        return HashMap::new();
    };
    let output = output.trim();
    if output.is_empty() {
        return HashMap::new();
    }
    let Ok(raw) = serde_json::from_str::<OneOrMany<RawDiskMetadataInfo>>(output) else {
        return HashMap::new();
    };

    raw.into_vec()
        .into_iter()
        .map(|entry| (entry.mount_point.clone(), entry))
        .collect()
}

#[cfg(not(target_os = "windows"))]
fn gather_disk_metadata() -> HashMap<String, RawDiskMetadataInfo> {
    HashMap::new()
}

#[cfg(target_os = "windows")]
fn refresh_disk_live() -> HashMap<String, DiskLiveInfo> {
    let script = r#"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$result = @()

foreach ($disk in Get-CimInstance Win32_PerfFormattedData_PerfDisk_LogicalDisk -ErrorAction SilentlyContinue | Where-Object { $_.Name -match '^[A-Z]:$' }) {
  $result += [pscustomobject]@{
    mount_point = "$($disk.Name)\"
    active_time_percent = [uint32][math]::Min([math]::Round($disk.PercentDiskTime), 100)
    avg_response_ms = [double]($disk.AvgDisksecPerTransfer * 1000)
    read_bytes_per_sec = [uint64]$disk.DiskReadBytesPersec
    write_bytes_per_sec = [uint64]$disk.DiskWriteBytesPersec
  }
}

@($result) | ConvertTo-Json -Depth 3 -Compress
"#;

    let Ok(output) = crate::shell::run_powershell(script) else {
        return HashMap::new();
    };
    let output = output.trim();
    if output.is_empty() {
        return HashMap::new();
    }
    let Ok(raw) = serde_json::from_str::<OneOrMany<RawDiskLiveInfo>>(output) else {
        return HashMap::new();
    };

    raw.into_vec()
        .into_iter()
        .map(|entry| {
            (
                entry.mount_point.clone(),
                DiskLiveInfo {
                    mount_point: entry.mount_point,
                    active_time_percent: entry.active_time_percent,
                    avg_response_ms: entry.avg_response_ms,
                    read_bytes_per_sec: entry.read_bytes_per_sec,
                    write_bytes_per_sec: entry.write_bytes_per_sec,
                },
            )
        })
        .collect()
}

#[cfg(target_os = "windows")]
fn gather_disk_live() -> HashMap<String, DiskLiveInfo> {
    let now = Instant::now();

    if let Ok(cache) = disk_live_cache().read()
        && let Some((cached_at, disks)) = cache.as_ref()
        && now.duration_since(*cached_at) < DISK_LIVE_CACHE_INTERVAL
    {
        return disks.clone();
    }

    let disks = refresh_disk_live();
    if let Ok(mut cache) = disk_live_cache().write() {
        *cache = Some((now, disks.clone()));
    }
    disks
}

#[cfg(not(target_os = "windows"))]
fn gather_disk_live() -> HashMap<String, DiskLiveInfo> {
    HashMap::new()
}

fn gather_disks() -> Vec<DiskInfo> {
    let disks = Disks::new_with_refreshed_list();
    let disk_metadata = gather_disk_metadata();
    let mut result: Vec<DiskInfo> = disks
        .iter()
        .filter(|d| d.total_space() > 0)
        .map(|d| {
            let mount_point = d.mount_point().to_string_lossy().into_owned();
            let volume_label = get_volume_label(&mount_point);
            let metadata = disk_metadata.get(&mount_point);
            DiskInfo {
                name: d.name().to_string_lossy().into_owned(),
                mount_point,
                total_bytes: d.total_space(),
                available_bytes: d.available_space(),
                kind: match d.kind() {
                    DiskKind::SSD => "SSD".to_string(),
                    DiskKind::HDD => "HDD".to_string(),
                    DiskKind::Unknown(_) => "unknown".to_string(),
                },
                file_system: d.file_system().to_string_lossy().into_owned(),
                volume_label,
                is_system_disk: metadata.map(|entry| entry.is_system_disk).unwrap_or(false),
                has_pagefile: metadata.map(|entry| entry.has_pagefile).unwrap_or(false),
                type_label: metadata
                    .map(|entry| entry.type_label.clone())
                    .unwrap_or_else(|| "Unknown".to_string()),
            }
        })
        .collect();
    result.sort_by(|a, b| a.mount_point.cmp(&b.mount_point));
    result
}

#[cfg(target_os = "windows")]
#[derive(Debug, Deserialize)]
struct RawNetworkAdapterInfo {
    name: String,
    adapter_description: String,
    dns_name: Option<String>,
    connection_type: String,
    #[serde(default)]
    ipv4_addresses: Vec<String>,
    #[serde(default)]
    ipv6_addresses: Vec<String>,
    is_wifi: bool,
    ssid: Option<String>,
    signal_percent: Option<u32>,
}

#[cfg(target_os = "windows")]
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}

#[cfg(target_os = "windows")]
impl<T> OneOrMany<T> {
    fn into_vec(self) -> Vec<T> {
        match self {
            Self::One(value) => vec![value],
            Self::Many(values) => values,
        }
    }
}

#[cfg(target_os = "windows")]
fn gather_network_adapters() -> Vec<NetworkAdapterInfo> {
    let script = r#"
[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$wifiByName = @{}
try {
  $wifiOutput = netsh wlan show interfaces 2>$null
  $current = @{}

  foreach ($line in $wifiOutput) {
    if ($line -match '^\s*$') {
      if ($current.ContainsKey('name')) {
        $wifiByName[$current.name] = [pscustomobject]$current
      }
      $current = @{}
      continue
    }

    if ($line -match '^\s*Name\s*:\s*(.+)$') {
      $current.name = $matches[1].Trim()
      continue
    }

    if ($line -match '^\s*SSID\s*:\s*(.+)$' -and $line -notmatch 'BSSID') {
      $current.ssid = $matches[1].Trim()
      continue
    }

    if ($line -match '^\s*Signal\s*:\s*(\d+)%$') {
      $current.signal_percent = [int]$matches[1]
      continue
    }

    if ($line -match '^\s*Radio type\s*:\s*(.+)$') {
      $current.connection_type = $matches[1].Trim()
      continue
    }
  }

  if ($current.ContainsKey('name')) {
    $wifiByName[$current.name] = [pscustomobject]$current
  }
} catch {}

$dnsByAlias = @{}
Get-DnsClient -ErrorAction SilentlyContinue | ForEach-Object {
  $dnsByAlias[$_.InterfaceAlias] = $_.ConnectionSpecificSuffix
}

$adapters = @(
  Get-NetAdapter -ErrorAction SilentlyContinue |
    Where-Object {
      $_.Status -ne 'Not Present' -and
      $_.PhysicalMediaType -ne 'BlueTooth' -and
      ($_.MediaType -eq '802.3' -or $_.MediaType -eq 'Native 802.11')
    } |
    Sort-Object InterfaceIndex
)

$result = foreach ($adapter in $adapters) {
  $ip = Get-NetIPConfiguration -InterfaceIndex $adapter.InterfaceIndex -ErrorAction SilentlyContinue
  $wifiInfo = $wifiByName[$adapter.Name]
  $ipv4Addresses = @($ip.IPv4Address | ForEach-Object { $_.IPv4Address } | Where-Object { $_ })
  $ipv6Addresses = @($ip.IPv6Address | ForEach-Object { $_.IPv6Address } | Where-Object { $_ })
  $hasDefaultGateway = @($ip.IPv4DefaultGateway).Count -gt 0 -or @($ip.IPv6DefaultGateway).Count -gt 0
  $hasRoutableIpv4 = @($ipv4Addresses | Where-Object { $_ -and $_ -notlike '169.254.*' }).Count -gt 0
  $isConnected = $adapter.Status -eq 'Up' -and ($adapter.MediaConnectionState -eq 'Connected' -or $hasDefaultGateway -or $hasRoutableIpv4)

  if (-not $isConnected) {
    continue
  }

  [pscustomobject]@{
    name = $adapter.Name
    adapter_description = $adapter.InterfaceDescription
    dns_name = $dnsByAlias[$adapter.Name]
    connection_type = if ($wifiInfo -and $wifiInfo.connection_type) {
      $wifiInfo.connection_type
    } elseif ($adapter.MediaType -eq 'Native 802.11') {
      'Wi-Fi'
    } else {
      'Ethernet'
    }
    ipv4_addresses = $ipv4Addresses
    ipv6_addresses = $ipv6Addresses
    is_wifi = $adapter.MediaType -eq 'Native 802.11'
    ssid = if ($wifiInfo) { $wifiInfo.ssid } else { $null }
    signal_percent = if ($wifiInfo) { $wifiInfo.signal_percent } else { $null }
  }
}

@($result) | ConvertTo-Json -Depth 4 -Compress
"#;

    let Ok(output) = crate::shell::run_powershell(script) else {
        return vec![];
    };

    let output = output.trim();
    if output.is_empty() {
        return vec![];
    }

    let Ok(raw) = serde_json::from_str::<OneOrMany<RawNetworkAdapterInfo>>(output) else {
        return vec![];
    };

    raw.into_vec()
        .into_iter()
        .enumerate()
        .map(|(index, adapter)| NetworkAdapterInfo {
            index,
            name: adapter.name,
            adapter_description: adapter.adapter_description,
            dns_name: adapter.dns_name.filter(|value| !value.trim().is_empty()),
            connection_type: adapter.connection_type,
            ipv4_addresses: adapter.ipv4_addresses,
            ipv6_addresses: adapter.ipv6_addresses,
            is_wifi: adapter.is_wifi,
            ssid: adapter.ssid.filter(|value| !value.trim().is_empty()),
            signal_percent: adapter.signal_percent,
        })
        .collect()
}

#[cfg(not(target_os = "windows"))]
fn gather_network_adapters() -> Vec<NetworkAdapterInfo> {
    vec![]
}

struct CpuExtra {
    sockets: u32,
    virtualization: bool,
    l1_cache_kb: Option<u32>,
    l2_cache_kb: Option<u32>,
    l3_cache_kb: Option<u32>,
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
fn gather_wmi_hardware() -> (MotherboardInfo, RamInfo, String, CpuExtra) {
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

    // Spawn a fresh thread so COMLibrary::new() (COINIT_MULTITHREADED) succeeds —
    // Tauri's WebView2 threads already have COM initialized in STA mode, which causes
    // RPC_E_CHANGED_MODE if we call CoInitializeEx from the command handler thread.
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

        // Motherboard
        let mut manufacturer = "Unknown".to_string();
        let mut product = "Unknown".to_string();
        if let Ok(rows) = wmi_con.raw_query::<HashMap<String, wmi::Variant>>(
            "SELECT Manufacturer, Product FROM Win32_BaseBoard",
        ) && let Some(mut row) = rows.into_iter().next()
        {
            manufacturer = wmi_str(&mut row, "Manufacturer");
            product = wmi_str(&mut row, "Product");
        }

        // BIOS
        let mut bios_vendor = "Unknown".to_string();
        let mut bios_version = "Unknown".to_string();
        if let Ok(rows) = wmi_con.raw_query::<HashMap<String, wmi::Variant>>(
            "SELECT Manufacturer, SMBIOSBIOSVersion FROM Win32_BIOS",
        ) && let Some(mut row) = rows.into_iter().next()
        {
            bios_vendor = wmi_str(&mut row, "Manufacturer");
            bios_version = wmi_str(&mut row, "SMBIOSBIOSVersion");
        }

        // RAM sticks — speed, capacity, form factor
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

        // Total RAM slots from array descriptor
        let mut total_slots: u32 = used_slots; // fallback: at least as many as installed
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

        // total_bytes: sum capacities from sticks (more accurate than sysinfo on some systems)
        let total_bytes: u64 = sticks
            .iter()
            .filter_map(|row| match row.get("Capacity") {
                Some(wmi::Variant::UI8(v)) => Some(*v),
                Some(wmi::Variant::UI4(v)) => Some(*v as u64),
                _ => None,
            })
            .sum();

        // Windows activation status via SoftwareLicensingProduct
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

        // Win32_Processor — one row per socket
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
fn gather_wmi_hardware() -> (MotherboardInfo, RamInfo, String, CpuExtra) {
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

/// Normalises "M-D-YYYY" or "MM-DD-YYYY" driver date to "YYYY-MM-DD".
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

/// Parses "PCI bus X, device Y, function Z" into (bus, device, function).
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

/// Looks up PCI bus/device/function via the Enum registry key for a device instance ID.
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

/// Returns `(driver_version, driver_date, pci_bus, pci_device, pci_function)` from the
/// GPU display class registry key matching the given adapter model name.
#[cfg(target_os = "windows")]
#[allow(clippy::type_complexity)] // 5-element tuple is the clearest representation here
fn gather_gpu_driver_info(
    model: &str,
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

        let driver_version: Option<String> = subkey.get_value("DriverVersion").ok();
        let driver_date: Option<String> = subkey
            .get_value::<String, _>("DriverDate")
            .ok()
            .and_then(|s| normalise_driver_date(&s));
        let (pci_bus, pci_device, pci_function) = subkey
            .get_value::<String, _>("DeviceInstanceID")
            .ok()
            .and_then(|id| get_pci_location(&id))
            .unwrap_or((None, None, None));

        return Some((
            driver_version,
            driver_date,
            pci_bus,
            pci_device,
            pci_function,
        ));
    }
    None
}

/// Derives the DirectX feature level version from the Windows build number.
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

pub fn gather_static_info(system: &System) -> Result<StaticSystemInfo, AppError> {
    use rayon::prelude::*;

    // Run the four independent collection branches in parallel.
    // • gather_windows_info  — registry reads
    // • gather_wmi_hardware  — WMI (already spawns its own thread internally for COM)
    // • gather_dxgi_gpus     — DXGI + registry driver scan (see closure below)
    // • gather_disks         — sysinfo + GetVolumeInformationW
    //
    // Measured sequential cost: ~280 ms.  After parallelisation the wall time
    // drops to roughly max(WMI ~150 ms, others ~30 ms) ≈ 150 ms.

    let cpu = gather_cpu_info(system);

    let ((windows_res, disks), (wmi_res, dxgi_adapters)) = rayon::join(
        || (gather_windows_info(), gather_disks()),
        || rayon::join(gather_wmi_hardware, enumerate_dxgi_adapters),
    );

    let mut windows = windows_res?;
    let (motherboard, mut ram, activation_status, cpu_extra) = wmi_res;
    windows.activation_status = activation_status;
    let network_adapters = gather_network_adapters();

    // GPU list: build GpuInfo per adapter in parallel (registry scan per GPU).
    #[cfg(target_os = "windows")]
    let gpus: Vec<GpuInfo> = {
        let build = windows.build;
        dxgi_adapters
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
                        gather_gpu_driver_info(&name).unwrap_or((None, None, None, None, None));
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
    };
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

    // Fall back to sysinfo total if WMI returned nothing (e.g. non-Windows build)
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

// ─── GPU live (DXGI + PDH + D3DKMT) ──────────────────────────────────────────

#[cfg(not(target_os = "windows"))]
fn enumerate_dxgi_adapters() -> Vec<(String, u32, i32, u64, u64, u64)> {
    vec![]
}

/// Returns `(name, luid_low, luid_high, vram_bytes, raw_dedicated_vram_bytes, shared_system_bytes)` for each adapter.
///
/// Uses `D3DKMTEnumAdapters2` (kernel-level) to enumerate ALL WDDM adapters —
/// this includes AMD iGPU adapters that `IDXGIFactory1::EnumAdapters1` may omit.
/// The returned LUIDs are identical to the LUIDs embedded in PDH counter instance names.
/// DXGI `IDXGIFactory4::EnumAdapterByLuid` is then used to get the display name and VRAM.
/// `raw_dedicated_vram_bytes` is the raw `DedicatedVideoMemory` (0 for iGPU).
#[cfg(target_os = "windows")]
fn enumerate_dxgi_adapters() -> Vec<(String, u32, i32, u64, u64, u64)> {
    use windows::Wdk::Graphics::Direct3D::*;
    use windows::Win32::Foundation::LUID;
    use windows::Win32::Graphics::Dxgi::*;
    use windows::core::Interface;
    unsafe {
        // First call: pAdapters = null → D3DKMTEnumAdapters2 fills NumAdapters with the count.
        let mut enum2 = D3DKMT_ENUMADAPTERS2 {
            NumAdapters: 0,
            pAdapters: std::ptr::null_mut(),
        };
        let _ = D3DKMTEnumAdapters2(&mut enum2);
        let count = enum2.NumAdapters as usize;
        if count == 0 {
            return vec![];
        }

        // Second call: supply allocated buffer.
        let mut kmt_adapters: Vec<D3DKMT_ADAPTERINFO> = vec![std::mem::zeroed(); count];
        enum2.pAdapters = kmt_adapters.as_mut_ptr();
        if D3DKMTEnumAdapters2(&mut enum2).0 < 0 {
            return vec![];
        }
        let actual = enum2.NumAdapters as usize;

        // IDXGIFactory4::EnumAdapterByLuid can resolve an adapter by LUID even if
        // IDXGIFactory1::EnumAdapters1 would have skipped it (e.g. AMD iGPU in hybrid setups).
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
                Err(_) => continue, // render-only or headless adapter with no DXGI interface
            };
            let desc = match adapter.GetDesc1() {
                Ok(d) => d,
                Err(_) => continue,
            };
            // Skip software (WARP) adapters — DXGI_ADAPTER_FLAG_SOFTWARE = 2
            if desc.Flags & 2 != 0 {
                continue;
            }

            let null_pos = desc.Description.iter().position(|&c| c == 0).unwrap_or(128);
            let name = String::from_utf16_lossy(&desc.Description[..null_pos]);
            // dGPU: DedicatedVideoMemory is the physical VRAM.
            // iGPU: DedicatedVideoMemory = 0; fall back to SharedSystemMemory (the system RAM
            // pool accessible to the GPU — what Task Manager shows for integrated adapters).
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

/// Manual definition of D3DKMT_ADAPTER_PERFDATA (KMTQAITYPE_ADAPTERPERFDATA = 70).
/// Temperature is in tenths of °C (e.g. 385 → 38.5 °C).
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

/// Queries GPU temperature via D3DKMT (WDDM kernel interface — works for all vendors).
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

/// Formats a DXGI adapter LUID as the prefix used in PDH counter instance names.
#[cfg(target_os = "windows")]
fn luid_to_pdh_prefix(luid_low: u32, luid_high: i32) -> String {
    format!("luid_0x{:08x}_0x{:08x}", luid_high as u32, luid_low).to_ascii_lowercase()
}

/// Opens a persistent PDH query for `\GPU Engine(*)\Utilization Percentage`.
/// Returns raw handle pair `(PDH_HQUERY.0, PDH_HCOUNTER.0)` as `isize`.
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
        let _ = PdhCollectQueryData(query); // priming collect
        Some((query.0 as isize, counter.0 as isize))
    }
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

/// Collects per-engine GPU utilization via PDH.
/// Returns `HashMap<luid_prefix, HashMap<engine_name, usage_percent>>`.
#[cfg(target_os = "windows")]
fn pdh_collect_gpu_usage(
    query_raw: isize,
    counter_raw: isize,
) -> HashMap<String, HashMap<String, f32>> {
    use windows::Win32::System::Performance::*;
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
        let raw_buf: Vec<u8>;
        let mut attempts = 0u32;
        loop {
            let mut buf: Vec<u8> = vec![0u8; buf_size as usize];
            let r2 = PdhGetFormattedCounterArrayW(
                counter,
                PDH_FMT_DOUBLE,
                &mut buf_size,
                &mut item_count,
                Some(buf.as_mut_ptr() as *mut PDH_FMT_COUNTERVALUE_ITEM_W),
            );
            if r2 == PDH_MORE_DATA && attempts < 3 {
                attempts += 1;
                continue;
            }
            if r2 != 0 {
                return HashMap::new();
            }
            raw_buf = buf;
            break;
        }
        let items = std::slice::from_raw_parts(
            raw_buf.as_ptr() as *const PDH_FMT_COUNTERVALUE_ITEM_W,
            item_count as usize,
        );
        let mut result: HashMap<String, HashMap<String, f32>> = HashMap::new();
        for item in items {
            let ptr = item.szName.0;
            if ptr.is_null() {
                continue;
            }
            let len = (0usize..).take_while(|&j| *ptr.add(j) != 0).count();
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
fn gather_gpu_live(gpus: &[GpuInfo], pdh_query: isize, pdh_counter: isize) -> Vec<GpuInfo> {
    use rayon::prelude::*;
    use windows::Win32::Foundation::LUID;
    use windows::Win32::Graphics::Dxgi::{
        CreateDXGIFactory1, DXGI_MEMORY_SEGMENT_GROUP_LOCAL, DXGI_MEMORY_SEGMENT_GROUP_NON_LOCAL,
        DXGI_QUERY_VIDEO_MEMORY_INFO, IDXGIAdapter1, IDXGIAdapter3, IDXGIFactory1, IDXGIFactory4,
    };
    use windows::core::Interface;

    // PDH: per-engine utilization for ALL GPUs, keyed by LUID prefix
    let pdh_open = pdh_query != 0;
    let usage_by_luid: HashMap<String, HashMap<String, f32>> = if pdh_open {
        pdh_collect_gpu_usage(pdh_query, pdh_counter)
    } else {
        HashMap::new()
    };

    // Collect D3DKMT temperatures in parallel — each call is an independent kernel
    // query with no shared state.  IDXGIFactory4 is !Send so we keep the DXGI memory
    // path sequential below.
    let temperatures: Vec<Option<u32>> = gpus
        .par_iter()
        .map(|gpu| get_gpu_temp_d3dkmt(gpu.luid_low, gpu.luid_high).map(|t| t.round() as u32))
        .collect();

    // Create DXGI factory once — used for memory queries on every GPU (not Send, stays sequential)
    let factory4_opt: Option<IDXGIFactory4> = unsafe {
        CreateDXGIFactory1::<IDXGIFactory1>()
            .ok()
            .and_then(|f| f.cast::<IDXGIFactory4>().ok())
    };

    gpus.iter()
        .zip(temperatures)
        .map(|(gpu, temperature_c)| {
            let (low, high) = (gpu.luid_low, gpu.luid_high);

            // PDH engine utilization — works for ALL GPUs (Intel, AMD, NVIDIA) via LUID
            let luid_prefix = luid_to_pdh_prefix(low, high);
            let engines: HashMap<String, f32> = if pdh_open {
                usage_by_luid.get(&luid_prefix).cloned().unwrap_or_default()
            } else {
                HashMap::new()
            };
            let get_eng = |key: &str| engines.get(key).copied().unwrap_or(0.0) as u32;

            // DXGI memory: LOCAL = dedicated, NON_LOCAL = shared
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
                            // Hardware-reserved = total reported by DXGI minus OS budget
                            let reserved = if local_ok && gpu.vram_total_mb > 0 {
                                gpu.vram_total_mb.saturating_sub(local.Budget / 1_048_576)
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

            GpuInfo {
                index: gpu.index,
                name: gpu.name.clone(),
                vendor: gpu.vendor.clone(),
                is_integrated: gpu.is_integrated,
                driver_version: gpu.driver_version.clone(),
                driver_date: gpu.driver_date.clone(),
                directx_version: gpu.directx_version.clone(),
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
                pci_bus: gpu.pci_bus,
                pci_device: gpu.pci_device,
                pci_function: gpu.pci_function,
                luid_low: low,
                luid_high: high,
            }
        })
        .collect()
}

#[cfg(not(target_os = "windows"))]
fn gather_gpu_live(_gpus: &[GpuInfo], _pdh_query: isize, _pdh_counter: isize) -> Vec<GpuInfo> {
    vec![]
}

// ─── Perf counters (process / thread / handle counts + RAM details) ──────────

#[cfg(target_os = "windows")]
fn get_perf_info() -> (u32, u32, u32) {
    use windows::Win32::System::ProcessStatus::{K32GetPerformanceInfo, PERFORMANCE_INFORMATION};
    let mut info: PERFORMANCE_INFORMATION = unsafe { std::mem::zeroed() };
    info.cb = std::mem::size_of::<PERFORMANCE_INFORMATION>() as u32;
    if unsafe { K32GetPerformanceInfo(&mut info, info.cb) } != false {
        (info.ProcessCount, info.ThreadCount, info.HandleCount)
    } else {
        (0, 0, 0)
    }
}

#[cfg(not(target_os = "windows"))]
fn get_perf_info() -> (u32, u32, u32) {
    (0, 0, 0)
}

/// Returns `(committed, commit_limit, cached, compressed, paged_pool, nonpaged_pool)` in bytes.
/// `committed`    = CommitTotal * PageSize — matches Task Manager "Committed" (RAM + pagefile used).
/// `commit_limit` = CommitLimit * PageSize — RAM + total pagefile size.
/// `cached`       = system file cache pages.
/// `compressed`   = best-effort Memory Compression working set.
/// `paged_pool`   = paged kernel pool.
/// `nonpaged_pool`= non-paged kernel pool.
#[cfg(target_os = "windows")]
fn get_ram_perf() -> (u64, u64, u64, u64, u64, u64) {
    use windows::Win32::System::ProcessStatus::{K32GetPerformanceInfo, PERFORMANCE_INFORMATION};

    let page = unsafe {
        let mut si = std::mem::zeroed::<windows::Win32::System::SystemInformation::SYSTEM_INFO>();
        windows::Win32::System::SystemInformation::GetSystemInfo(&mut si);
        si.dwPageSize as u64
    };

    // CommitTotal/CommitLimit from PERFORMANCE_INFORMATION match Task Manager "Committed" exactly.
    // ullTotalVirtual - ullAvailVirtual is the *virtual address space* committed — can exceed RAM.
    let (committed, commit_limit, cached, paged, nonpaged) = unsafe {
        let mut info = std::mem::zeroed::<PERFORMANCE_INFORMATION>();
        info.cb = std::mem::size_of::<PERFORMANCE_INFORMATION>() as u32;
        if K32GetPerformanceInfo(&mut info, info.cb) != false {
            (
                info.CommitTotal as u64 * page,
                info.CommitLimit as u64 * page,
                info.SystemCache as u64 * page,
                info.KernelPaged as u64 * page,
                info.KernelNonpaged as u64 * page,
            )
        } else {
            (0, 0, 0, 0, 0)
        }
    };

    let compressed = get_compressed_memory_bytes();

    (committed, commit_limit, cached, compressed, paged, nonpaged)
}

#[cfg(not(target_os = "windows"))]
fn get_ram_perf() -> (u64, u64, u64, u64, u64, u64) {
    (0, 0, 0, 0, 0, 0)
}

/// Returns a best-effort compressed memory value in bytes by reading the
/// `Memory Compression` process working set.
/// This avoids the previous brittle `MemCompression`-only lookup and the
/// incorrect `PrivateUsage` metric that often produced `0`.
#[cfg(target_os = "windows")]
fn get_compressed_memory_bytes() -> u64 {
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Diagnostics::ToolHelp::{
        CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW,
        TH32CS_SNAPPROCESS,
    };
    use windows::Win32::System::ProcessStatus::{
        K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS_EX,
    };
    use windows::Win32::System::Threading::{
        OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_VM_READ,
    };

    fn is_memory_compression_process(name: &[u16]) -> bool {
        let process_name = String::from_utf16_lossy(name).to_ascii_lowercase();
        matches!(
            process_name.as_str(),
            "memory compression" | "memcompression"
        )
    }

    unsafe {
        let snap = match CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) {
            Ok(h) => h,
            Err(_) => return 0,
        };

        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        if Process32FirstW(snap, &mut entry).is_err() {
            let _ = CloseHandle(snap);
            return 0;
        }

        let mut result: u64 = 0;

        loop {
            let name_len = entry
                .szExeFile
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(entry.szExeFile.len());
            let name = &entry.szExeFile[..name_len];

            if is_memory_compression_process(name) {
                let handle = OpenProcess(
                    PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
                    false,
                    entry.th32ProcessID,
                )
                .or_else(|_| {
                    OpenProcess(
                        PROCESS_QUERY_LIMITED_INFORMATION,
                        false,
                        entry.th32ProcessID,
                    )
                });

                if let Ok(handle) = handle {
                    let mut mem: PROCESS_MEMORY_COUNTERS_EX = std::mem::zeroed();
                    mem.cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32;
                    if K32GetProcessMemoryInfo(
                        handle,
                        &mut mem as *mut PROCESS_MEMORY_COUNTERS_EX
                            as *mut windows::Win32::System::ProcessStatus::PROCESS_MEMORY_COUNTERS,
                        mem.cb,
                    ) != false
                    {
                        result = mem.WorkingSetSize as u64;
                    }
                    let _ = CloseHandle(handle);
                }
                break;
            }

            if Process32NextW(snap, &mut entry).is_err() {
                break;
            }
        }

        let _ = CloseHandle(snap);
        result
    }
}

#[cfg(not(target_os = "windows"))]
fn get_compressed_memory_bytes() -> u64 {
    0
}

/// Opens a persistent PDH query for `\Processor Information(_Total)\% Processor Performance`.
/// Returns raw handle pair `(PDH_HQUERY.0, PDH_HCOUNTER.0)` as `isize`.
/// Performs a priming collection so the first real call returns valid data.
#[cfg(target_os = "windows")]
pub fn pdh_open_cpu_perf_query() -> Option<(isize, isize)> {
    use windows::Win32::System::Performance::*;
    unsafe {
        let mut query = PDH_HQUERY(std::ptr::null_mut());
        if PdhOpenQueryW(windows::core::PCWSTR(std::ptr::null()), 0, &mut query) != 0 {
            return None;
        }
        let path = windows::core::w!(r"\Processor Information(_Total)\% Processor Performance");
        let mut counter = PDH_HCOUNTER(std::ptr::null_mut());
        if PdhAddEnglishCounterW(query, path, 0, &mut counter) != 0 {
            let _ = PdhCloseQuery(query);
            return None;
        }
        let _ = PdhCollectQueryData(query); // priming collect
        Some((query.0 as isize, counter.0 as isize))
    }
}

/// Collects the `% Processor Performance` counter value (0–∞ %).
/// Values > 100 indicate boost above base frequency.
#[cfg(target_os = "windows")]
fn pdh_collect_cpu_perf_pct(query_raw: isize, counter_raw: isize) -> Option<f64> {
    use windows::Win32::System::Performance::*;
    unsafe {
        let query = PDH_HQUERY(query_raw as *mut _);
        let counter = PDH_HCOUNTER(counter_raw as *mut _);
        let _ = PdhCollectQueryData(query);
        let mut val: PDH_FMT_COUNTERVALUE = std::mem::zeroed();
        let r = PdhGetFormattedCounterValue(counter, PDH_FMT_DOUBLE, None, &mut val);
        if r != 0 {
            return None;
        }
        Some(val.Anonymous.doubleValue)
    }
}

// ─── Live info gathering ──────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)] // Keeps the background collector callsite explicit across cfg-gated inputs.
pub fn gather_live_info(
    system: &mut System,
    networks: &mut Networks,
    prev_net: &mut Option<PreviousNetSnapshot>,
    gpus: &[GpuInfo],
    #[cfg(target_os = "windows")] pdh_query: isize,
    #[cfg(target_os = "windows")] pdh_counter: isize,
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

    // Keep only real adapters (have ever sent/received bytes, not loopback).
    // Normalize deltas by actual elapsed wall time so bytes/sec stays correct even
    // when collection drifts from the target 1s cadence.
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

    let disks: Vec<DiskLiveInfo> = gather_disk_live().into_values().collect();

    *prev_net = Some(PreviousNetSnapshot {
        captured_at: now,
        totals: current_net,
    });

    // GPU live stats: PDH (per-engine, all GPUs) + D3DKMT (temperature) + DXGI (memory)
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
