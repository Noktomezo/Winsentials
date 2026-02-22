export interface OsInfo {
  name: string
  version: string
  build: string
  arch: string
  display_version: string
  hostname: string
  username: string
}

export interface StaticCpuInfo {
  name: string
  cores: number
  logical_cores: number
}

export interface DynamicCpuInfo {
  usage: number
  frequency: number
}

export interface StaticGpuInfo {
  name: string
  memory_total: number
}

export interface DynamicGpuInfo {
  usage: number
  memory_used: number
}

export interface StaticRamInfo {
  total: number
  slots_used: number
  slots_total: number
  speed: number
}

export interface DynamicRamInfo {
  used: number
  usage: number
}

export interface StaticDiskInfo {
  name: string
  mount_point: string
  label: string
  total: number
}

export interface DynamicDiskInfo {
  mount_point: string
  available: number
  usage: number
}

export interface StaticSystemInfo {
  os: OsInfo
  cpu: StaticCpuInfo
  gpu: StaticGpuInfo | null
  ram: StaticRamInfo
  disks: StaticDiskInfo[]
}

export interface DynamicSystemInfo {
  cpu: DynamicCpuInfo
  gpu: DynamicGpuInfo | null
  ram: DynamicRamInfo
  disks: DynamicDiskInfo[]
}

// Legacy types for backward compatibility
export interface CpuInfo {
  name: string
  usage: number
  frequency: number
  cores: number
  logical_cores: number
}

export interface GpuInfo {
  name: string
  usage: number
  memory_total: number
  memory_used: number
}

export interface RamInfo {
  total: number
  used: number
  usage: number
  slots_used: number
  slots_total: number
  speed: number
}

export interface DiskInfo {
  name: string
  mount_point: string
  label: string
  total: number
  available: number
  usage: number
}

export interface SystemInfo {
  os: OsInfo
  cpu: CpuInfo
  gpu: GpuInfo | null
  ram: RamInfo
  disks: DiskInfo[]
}
