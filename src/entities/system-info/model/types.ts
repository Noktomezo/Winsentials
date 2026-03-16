export interface WindowsInfo {
  productName: string
  displayVersion: string
  build: number
  ubr: number
  hostname: string
  username: string
  architecture: string
  activationStatus: string
}

export interface CpuInfo {
  model: string
  physicalCores: number
  logicalCores: number
  baseFreqMhz: number
}

export interface GpuInfo {
  model: string
  vendor: string
  vramBytes: number | null
}

export interface MotherboardInfo {
  manufacturer: string
  product: string
  biosVendor: string
  biosVersion: string
}

export interface DiskInfo {
  name: string
  mountPoint: string
  totalBytes: number
  availableBytes: number
  kind: string
  fileSystem: string
}

export interface RamInfo {
  totalBytes: number
  speedMhz: number | null
  usedSlots: number
  totalSlots: number
}

export interface StaticSystemInfo {
  windows: WindowsInfo
  cpu: CpuInfo
  ram: RamInfo
  gpus: GpuInfo[]
  motherboard: MotherboardInfo
  disks: DiskInfo[]
}

export interface NetworkIfaceStats {
  name: string
  rxBytesPerSec: number
  txBytesPerSec: number
}

export interface GpuLiveStats {
  index: number
  usagePercent: number | null
  temperatureCelsius: number | null
}

export interface LiveSystemInfo {
  cpuUsagePercent: number
  cpuPerCore: number[]
  ramUsedBytes: number
  network: NetworkIfaceStats[]
  gpuLive: GpuLiveStats[]
}
