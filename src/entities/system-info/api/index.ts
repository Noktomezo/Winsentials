import type {
  CpuInfo,
  DiskInfo,
  GpuInfo,
  GpuLiveStats,
  LiveSystemInfo,
  MotherboardInfo,
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
}

interface BackendGpuInfo {
  model: string
  vendor: string
  vram_bytes: number | null
}

interface BackendMotherboardInfo {
  manufacturer: string
  product: string
  bios_vendor: string
  bios_version: string
}

interface BackendDiskInfo {
  name: string
  mount_point: string
  total_bytes: number
  available_bytes: number
  kind: string
  file_system: string
}

interface BackendRamInfo {
  total_bytes: number
  speed_mhz: number | null
  used_slots: number
  total_slots: number
}

interface BackendStaticSystemInfo {
  windows: BackendWindowsInfo
  cpu: BackendCpuInfo
  ram: BackendRamInfo
  gpus: BackendGpuInfo[]
  motherboard: BackendMotherboardInfo
  disks: BackendDiskInfo[]
}

interface BackendNetworkIfaceStats {
  name: string
  rx_bytes_per_sec: number
  tx_bytes_per_sec: number
}

interface BackendGpuLiveStats {
  index: number
  usage_percent: number | null
  temperature_celsius: number | null
}

interface BackendLiveSystemInfo {
  cpu_usage_percent: number
  cpu_per_core: number[]
  ram_used_bytes: number
  network: BackendNetworkIfaceStats[]
  gpu_live: BackendGpuLiveStats[]
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
  }
}

function mapGpu(g: BackendGpuInfo): GpuInfo {
  return { model: g.model, vendor: g.vendor, vramBytes: g.vram_bytes }
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
    mountPoint: d.mount_point,
    totalBytes: d.total_bytes,
    availableBytes: d.available_bytes,
    kind: d.kind,
    fileSystem: d.file_system,
  }
}

function mapRam(r: BackendRamInfo): RamInfo {
  return {
    totalBytes: r.total_bytes,
    speedMhz: r.speed_mhz,
    usedSlots: r.used_slots,
    totalSlots: r.total_slots,
  }
}

function mapStatic(raw: BackendStaticSystemInfo): StaticSystemInfo {
  return {
    windows: mapWindows(raw.windows),
    cpu: mapCpu(raw.cpu),
    ram: mapRam(raw.ram),
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

function mapGpuLive(g: BackendGpuLiveStats): GpuLiveStats {
  return {
    index: g.index,
    usagePercent: g.usage_percent,
    temperatureCelsius: g.temperature_celsius,
  }
}

function mapLive(raw: BackendLiveSystemInfo): LiveSystemInfo {
  return {
    cpuUsagePercent: raw.cpu_usage_percent,
    cpuPerCore: raw.cpu_per_core,
    ramUsedBytes: raw.ram_used_bytes,
    network: raw.network.map(mapNetwork),
    gpuLive: raw.gpu_live.map(mapGpuLive),
  }
}

// ─── Public API ───────────────────────────────────────────────────────────────

export async function getStaticSystemInfo(): Promise<StaticSystemInfo> {
  const raw = await invoke<BackendStaticSystemInfo>('get_static_system_info')
  return mapStatic(raw)
}

export async function getLiveSystemInfo(): Promise<LiveSystemInfo> {
  const raw = await invoke<BackendLiveSystemInfo>('get_live_system_info')
  return mapLive(raw)
}
