import type {
  CpuInfo,
  DeviceInventoryInfo,
  DiskInfo,
  DiskLiveInfo,
  GpuInfo,
  LiveCpuInfo,
  LiveGpuInfo,
  LiveHomeInfo,
  LiveRamInfo,
  LiveSystemInfo,
  MotherboardInfo,
  NetworkAdapterInfo,
  NetworkIfaceStats,
  RamInfo,
  StaticSystemInfo,
  WindowsInfo,
} from '@/entities/system-info/model/types'
import { invoke } from '@tauri-apps/api/core'

// ─── Backend snake_case shapes ───────────────────────────────────────────────

interface BackendWindowsInfo {
  product_name: string
  display_version: string
  build: number
  ubr: number
  hostname: string
  username: string
  architecture: string
  activation_status: string
}

interface BackendCpuInfo {
  model: string
  physical_cores: number
  logical_cores: number
  base_freq_mhz: number
  sockets: number
  virtualization: boolean
  l1_cache_kb: number | null
  l2_cache_kb: number | null
  l3_cache_kb: number | null
}

interface BackendGpuInfo {
  index: number
  name: string
  vendor: string
  is_integrated: boolean
  driver_version: string | null
  driver_date: string | null
  directx_version: string | null
  vram_total_mb: number
  vram_used_mb: number
  vram_shared_mb: number
  vram_reserved_mb: number
  temperature_c: number | null
  power_w: number | null
  util_3d: number
  util_copy: number
  util_encode: number
  util_decode: number
  util_high_priority_3d: number
  util_high_priority_compute: number
  processes: BackendGpuProcess[]
  pci_bus: number | null
  pci_device: number | null
  pci_function: number | null
}

interface BackendGpuProcess {
  pid: number
  name: string
  dedicated_mem_mb: number
}

interface BackendMotherboardInfo {
  manufacturer: string
  product: string
  bios_vendor: string
  bios_version: string
}

interface BackendDiskInfo {
  name: string
  model: string | null
  mount_point: string
  total_bytes: number
  available_bytes: number
  kind: string
  file_system: string
  volume_label: string | null
  is_system_disk: boolean
  has_pagefile: boolean
  type_label: string
}

interface BackendDiskLiveInfo {
  mount_point: string
  active_time_percent: number
  avg_response_ms: number
  read_bytes_per_sec: number
  write_bytes_per_sec: number
}

interface BackendRamInfo {
  total_bytes: number
  speed_mhz: number | null
  used_slots: number
  total_slots: number
  form_factor: string | null
}

interface BackendStaticSystemInfo {
  windows: BackendWindowsInfo
  cpu: BackendCpuInfo
  ram: BackendRamInfo
  network_adapters: BackendNetworkAdapterInfo[]
  gpus: BackendGpuInfo[]
  motherboard: BackendMotherboardInfo
  disks: BackendDiskInfo[]
}

interface BackendDeviceInventoryInfo {
  network_adapters: BackendNetworkAdapterInfo[]
  disks: BackendDiskInfo[]
}

interface BackendNetworkAdapterInfo {
  index: number
  name: string
  adapter_description: string
  dns_name: string | null
  connection_type: string
  ipv4_addresses: string[]
  ipv6_addresses: string[]
  is_wifi: boolean
  ssid: string | null
  signal_percent: number | null
}

interface BackendNetworkIfaceStats {
  name: string
  rx_bytes_per_sec: number
  tx_bytes_per_sec: number
}

interface BackendLiveSystemInfo {
  cpu_usage_percent: number
  cpu_per_core: number[]
  cpu_current_freq_mhz: number
  cpu_process_count: number
  cpu_thread_count: number
  cpu_handle_count: number
  cpu_uptime_secs: number
  ram_used_bytes: number
  ram_available_bytes: number
  ram_committed_bytes: number
  ram_commit_limit_bytes: number
  ram_cached_bytes: number
  ram_compressed_bytes: number
  ram_paged_pool_bytes: number
  ram_nonpaged_pool_bytes: number
  disks: BackendDiskLiveInfo[]
  network: BackendNetworkIfaceStats[]
  gpus: BackendLiveGpuInfo[]
}

interface BackendLiveCpuInfo {
  cpu_usage_percent: number
  cpu_per_core: number[]
  cpu_current_freq_mhz: number
  cpu_process_count: number
  cpu_thread_count: number
  cpu_handle_count: number
  cpu_uptime_secs: number
}

interface BackendLiveRamInfo {
  ram_used_bytes: number
  ram_available_bytes: number
  ram_committed_bytes: number
  ram_commit_limit_bytes: number
  ram_cached_bytes: number
  ram_compressed_bytes: number
  ram_paged_pool_bytes: number
  ram_nonpaged_pool_bytes: number
}

interface BackendLiveGpuInfo {
  index: number
  vram_total_mb: number
  vram_used_mb: number
  vram_shared_mb: number
  vram_reserved_mb: number
  temperature_c: number | null
  power_w: number | null
  util_3d: number
  util_copy: number
  util_encode: number
  util_decode: number
  util_high_priority_3d: number
  util_high_priority_compute: number
  processes: BackendGpuProcess[]
}

interface BackendLiveHomeInfo {
  cpu_usage_percent: number
  ram_used_bytes: number
  network: BackendNetworkIfaceStats[]
  gpus: BackendLiveGpuInfo[]
}

// ─── Mappers ─────────────────────────────────────────────────────────────────

function mapWindows(w: BackendWindowsInfo): WindowsInfo {
  return {
    productName: w.product_name,
    displayVersion: w.display_version,
    build: w.build,
    ubr: w.ubr,
    hostname: w.hostname,
    username: w.username,
    architecture: w.architecture,
    activationStatus: w.activation_status,
  }
}

function mapCpu(c: BackendCpuInfo): CpuInfo {
  return {
    model: c.model,
    physicalCores: c.physical_cores,
    logicalCores: c.logical_cores,
    baseFreqMhz: c.base_freq_mhz,
    sockets: c.sockets,
    virtualization: c.virtualization,
    l1CacheKb: c.l1_cache_kb,
    l2CacheKb: c.l2_cache_kb,
    l3CacheKb: c.l3_cache_kb,
  }
}

function mapGpu(g: BackendGpuInfo): GpuInfo {
  return {
    index: g.index,
    name: g.name,
    vendor: g.vendor,
    isIntegrated: g.is_integrated,
    driverVersion: g.driver_version,
    driverDate: g.driver_date,
    directxVersion: g.directx_version,
    vramTotalMb: g.vram_total_mb,
    vramUsedMb: g.vram_used_mb,
    vramSharedMb: g.vram_shared_mb,
    vramReservedMb: g.vram_reserved_mb,
    temperatureC: g.temperature_c,
    powerW: g.power_w,
    util3d: g.util_3d,
    utilCopy: g.util_copy,
    utilEncode: g.util_encode,
    utilDecode: g.util_decode,
    utilHighPriority3d: g.util_high_priority_3d,
    utilHighPriorityCompute: g.util_high_priority_compute,
    processes: g.processes.map(p => ({ pid: p.pid, name: p.name, dedicatedMemMb: p.dedicated_mem_mb })),
    pciBus: g.pci_bus,
    pciDevice: g.pci_device,
    pciFunction: g.pci_function,
  }
}

function mapMotherboard(m: BackendMotherboardInfo): MotherboardInfo {
  return {
    manufacturer: m.manufacturer,
    product: m.product,
    biosVendor: m.bios_vendor,
    biosVersion: m.bios_version,
  }
}

function mapDisk(d: BackendDiskInfo): DiskInfo {
  return {
    name: d.name,
    model: d.model,
    mountPoint: d.mount_point,
    totalBytes: d.total_bytes,
    availableBytes: d.available_bytes,
    kind: d.kind,
    fileSystem: d.file_system,
    volumeLabel: d.volume_label,
    isSystemDisk: d.is_system_disk,
    hasPagefile: d.has_pagefile,
    typeLabel: d.type_label,
  }
}

function mapDiskLive(d: BackendDiskLiveInfo): DiskLiveInfo {
  return {
    mountPoint: d.mount_point,
    activeTimePercent: d.active_time_percent,
    avgResponseMs: d.avg_response_ms,
    readBytesPerSec: d.read_bytes_per_sec,
    writeBytesPerSec: d.write_bytes_per_sec,
  }
}

function mapNetworkAdapter(n: BackendNetworkAdapterInfo): NetworkAdapterInfo {
  return {
    index: n.index,
    name: n.name,
    adapterDescription: n.adapter_description,
    dnsName: n.dns_name,
    connectionType: n.connection_type,
    ipv4Addresses: n.ipv4_addresses,
    ipv6Addresses: n.ipv6_addresses,
    isWifi: n.is_wifi,
    ssid: n.ssid,
    signalPercent: n.signal_percent,
  }
}

function mapRam(r: BackendRamInfo): RamInfo {
  return {
    totalBytes: r.total_bytes,
    speedMhz: r.speed_mhz,
    usedSlots: r.used_slots,
    totalSlots: r.total_slots,
    formFactor: r.form_factor,
  }
}

function mapStatic(raw: BackendStaticSystemInfo): StaticSystemInfo {
  return {
    windows: mapWindows(raw.windows),
    cpu: mapCpu(raw.cpu),
    ram: mapRam(raw.ram),
    networkAdapters: raw.network_adapters.map(mapNetworkAdapter),
    gpus: raw.gpus.map(mapGpu),
    motherboard: mapMotherboard(raw.motherboard),
    disks: raw.disks.map(mapDisk),
  }
}

function mapNetwork(n: BackendNetworkIfaceStats): NetworkIfaceStats {
  return {
    name: n.name,
    rxBytesPerSec: n.rx_bytes_per_sec,
    txBytesPerSec: n.tx_bytes_per_sec,
  }
}

function mapLive(raw: BackendLiveSystemInfo): LiveSystemInfo {
  return {
    cpuUsagePercent: raw.cpu_usage_percent,
    cpuPerCore: raw.cpu_per_core,
    cpuCurrentFreqMhz: raw.cpu_current_freq_mhz,
    cpuProcessCount: raw.cpu_process_count,
    cpuThreadCount: raw.cpu_thread_count,
    cpuHandleCount: raw.cpu_handle_count,
    cpuUptimeSecs: raw.cpu_uptime_secs,
    ramUsedBytes: raw.ram_used_bytes,
    ramAvailableBytes: raw.ram_available_bytes,
    ramCommittedBytes: raw.ram_committed_bytes,
    ramCommitLimitBytes: raw.ram_commit_limit_bytes,
    ramCachedBytes: raw.ram_cached_bytes,
    ramCompressedBytes: raw.ram_compressed_bytes,
    ramPagedPoolBytes: raw.ram_paged_pool_bytes,
    ramNonpagedPoolBytes: raw.ram_nonpaged_pool_bytes,
    disks: raw.disks.map(mapDiskLive),
    network: raw.network.map(mapNetwork),
    gpus: raw.gpus.map(mapLiveGpu),
  }
}

function mapDeviceInventory(raw: BackendDeviceInventoryInfo): DeviceInventoryInfo {
  return {
    networkAdapters: raw.network_adapters.map(mapNetworkAdapter),
    disks: raw.disks.map(mapDisk),
  }
}

function mapLiveCpu(raw: BackendLiveCpuInfo): LiveCpuInfo {
  return {
    cpuUsagePercent: raw.cpu_usage_percent,
    cpuPerCore: raw.cpu_per_core,
    cpuCurrentFreqMhz: raw.cpu_current_freq_mhz,
    cpuProcessCount: raw.cpu_process_count,
    cpuThreadCount: raw.cpu_thread_count,
    cpuHandleCount: raw.cpu_handle_count,
    cpuUptimeSecs: raw.cpu_uptime_secs,
  }
}

function mapLiveRam(raw: BackendLiveRamInfo): LiveRamInfo {
  return {
    ramUsedBytes: raw.ram_used_bytes,
    ramAvailableBytes: raw.ram_available_bytes,
    ramCommittedBytes: raw.ram_committed_bytes,
    ramCommitLimitBytes: raw.ram_commit_limit_bytes,
    ramCachedBytes: raw.ram_cached_bytes,
    ramCompressedBytes: raw.ram_compressed_bytes,
    ramPagedPoolBytes: raw.ram_paged_pool_bytes,
    ramNonpagedPoolBytes: raw.ram_nonpaged_pool_bytes,
  }
}

function mapLiveGpu(g: BackendLiveGpuInfo): LiveGpuInfo {
  return {
    index: g.index,
    vramTotalMb: g.vram_total_mb,
    vramUsedMb: g.vram_used_mb,
    vramSharedMb: g.vram_shared_mb,
    vramReservedMb: g.vram_reserved_mb,
    temperatureC: g.temperature_c,
    powerW: g.power_w,
    util3d: g.util_3d,
    utilCopy: g.util_copy,
    utilEncode: g.util_encode,
    utilDecode: g.util_decode,
    utilHighPriority3d: g.util_high_priority_3d,
    utilHighPriorityCompute: g.util_high_priority_compute,
    processes: g.processes.map(p => ({ pid: p.pid, name: p.name, dedicatedMemMb: p.dedicated_mem_mb })),
  }
}

function mapLiveHome(raw: BackendLiveHomeInfo): LiveHomeInfo {
  return {
    cpuUsagePercent: raw.cpu_usage_percent,
    ramUsedBytes: raw.ram_used_bytes,
    network: raw.network.map(mapNetwork),
    gpus: raw.gpus.map(mapLiveGpu),
  }
}

// ─── Public API ───────────────────────────────────────────────────────────────

export async function getStaticSystemInfo(): Promise<StaticSystemInfo> {
  const raw = await invoke<BackendStaticSystemInfo>('get_static_system_info')
  return mapStatic(raw)
}

export async function getDeviceInventoryInfo(): Promise<DeviceInventoryInfo> {
  const raw = await invoke<BackendDeviceInventoryInfo>('get_device_inventory_info')
  return mapDeviceInventory(raw)
}

export async function getLiveSystemInfo(): Promise<LiveSystemInfo> {
  const raw = await invoke<BackendLiveSystemInfo>('get_live_system_info')
  return mapLive(raw)
}

export async function getLiveHomeInfo(): Promise<LiveHomeInfo> {
  const raw = await invoke<BackendLiveHomeInfo>('get_live_home_info')
  return mapLiveHome(raw)
}

export async function getLiveCpuInfo(): Promise<LiveCpuInfo> {
  const raw = await invoke<BackendLiveCpuInfo>('get_live_cpu_info')
  return mapLiveCpu(raw)
}

export async function getLiveRamInfo(): Promise<LiveRamInfo> {
  const raw = await invoke<BackendLiveRamInfo>('get_live_ram_info')
  return mapLiveRam(raw)
}

export async function getLiveDiskInfo(): Promise<DiskLiveInfo[]> {
  const raw = await invoke<BackendDiskLiveInfo[]>('get_live_disk_info')
  return raw.map(mapDiskLive)
}

export async function getLiveNetworkInfo(): Promise<NetworkIfaceStats[]> {
  const raw = await invoke<BackendNetworkIfaceStats[]>('get_live_network_info')
  return raw.map(mapNetwork)
}

export async function getLiveGpuInfo(): Promise<LiveGpuInfo[]> {
  const raw = await invoke<BackendLiveGpuInfo[]>('get_live_gpu_info')
  return raw.map(mapLiveGpu)
}
