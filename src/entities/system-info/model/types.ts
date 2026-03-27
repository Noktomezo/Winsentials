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
  sockets: number
  virtualization: boolean
  l1CacheKb: number | null
  l2CacheKb: number | null
  l3CacheKb: number | null
}

export interface GpuProcess {
  pid: number
  name: string
  dedicatedMemMb: number
}

export interface GpuInfo {
  index: number
  name: string
  vendor: string
  isIntegrated: boolean
  driverVersion: string | null
  driverDate: string | null
  directxVersion: string | null
  vramTotalMb: number
  vramUsedMb: number
  vramSharedMb: number
  vramReservedMb: number
  temperatureC: number | null
  powerW: number | null
  util3d: number
  utilCopy: number
  utilEncode: number
  utilDecode: number
  utilHighPriority3d: number
  utilHighPriorityCompute: number
  processes: GpuProcess[]
  pciBus: number | null
  pciDevice: number | null
  pciFunction: number | null
}

export interface MotherboardInfo {
  manufacturer: string
  product: string
  biosVendor: string
  biosVersion: string
}

export interface DiskInfo {
  name: string
  model: string | null
  mountPoint: string
  totalBytes: number
  availableBytes: number
  kind: string
  fileSystem: string
  volumeLabel: string | null
  isSystemDisk: boolean
  hasPagefile: boolean
  typeLabel: string
}

export interface DiskLiveInfo {
  mountPoint: string
  activeTimePercent: number
  avgResponseMs: number
  readBytesPerSec: number
  writeBytesPerSec: number
}

export interface RamInfo {
  totalBytes: number
  speedMhz: number | null
  usedSlots: number
  totalSlots: number
  formFactor: string | null
}

export interface NetworkAdapterInfo {
  index: number
  name: string
  adapterDescription: string
  dnsName: string | null
  connectionType: string
  ipv4Addresses: string[]
  ipv6Addresses: string[]
  isWifi: boolean
  ssid: string | null
  signalPercent: number | null
}

export interface StaticSystemInfo {
  windows: WindowsInfo
  cpu: CpuInfo
  ram: RamInfo
  networkAdapters: NetworkAdapterInfo[]
  gpus: GpuInfo[]
  motherboard: MotherboardInfo
  disks: DiskInfo[]
}

export interface DeviceInventoryInfo {
  networkAdapters: NetworkAdapterInfo[]
  disks: DiskInfo[]
}

export interface NetworkIfaceStats {
  name: string
  rxBytesPerSec: number
  txBytesPerSec: number
}

export interface LiveSystemInfo {
  cpuUsagePercent: number
  cpuPerCore: number[]
  cpuCurrentFreqMhz: number
  cpuProcessCount: number
  cpuThreadCount: number
  cpuHandleCount: number
  cpuUptimeSecs: number
  ramUsedBytes: number
  ramAvailableBytes: number
  ramCommittedBytes: number
  ramCommitLimitBytes: number
  ramCachedBytes: number
  ramCompressedBytes: number
  ramPagedPoolBytes: number
  ramNonpagedPoolBytes: number
  disks: DiskLiveInfo[]
  network: NetworkIfaceStats[]
  gpus: LiveGpuInfo[]
}

export interface LiveHomeInfo {
  cpuUsagePercent: number
  ramUsedBytes: number
  network: NetworkIfaceStats[]
  gpus: LiveGpuInfo[]
}

export interface LiveCpuInfo {
  cpuUsagePercent: number
  cpuPerCore: number[]
  cpuCurrentFreqMhz: number
  cpuProcessCount: number
  cpuThreadCount: number
  cpuHandleCount: number
  cpuUptimeSecs: number
}

export interface LiveRamInfo {
  ramUsedBytes: number
  ramAvailableBytes: number
  ramCommittedBytes: number
  ramCommitLimitBytes: number
  ramCachedBytes: number
  ramCompressedBytes: number
  ramPagedPoolBytes: number
  ramNonpagedPoolBytes: number
}

export interface LiveGpuInfo {
  index: number
  vramTotalMb: number
  vramUsedMb: number
  vramSharedMb: number
  vramReservedMb: number
  temperatureC: number | null
  powerW: number | null
  util3d: number
  utilCopy: number
  utilEncode: number
  utilDecode: number
  utilHighPriority3d: number
  utilHighPriorityCompute: number
  processes: GpuProcess[]
}
