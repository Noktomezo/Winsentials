import type {
  DeviceInventoryInfo,
  DiskLiveInfo,
  LiveCpuInfo,
  LiveGpuInfo,
  LiveHomeInfo,
  LiveRamInfo,
  NetworkIfaceStats,
} from '@/entities/system-info/model/types'
import { create } from 'zustand'
import {
  getDeviceInventoryInfo,
  getLiveCpuInfo,
  getLiveDiskInfo,
  getLiveGpuInfo,
  getLiveHomeInfo,
  getLiveNetworkInfo,
  getLiveRamInfo,
} from '@/entities/system-info/api'
import { useMountEffect } from '@/shared/lib/hooks/use-mount-effect'

const MAX_HISTORY = 60

type LiveSliceKey = 'home' | 'cpu' | 'ram' | 'gpu' | 'network' | 'disks' | 'deviceInventory'

interface LiveSliceMap {
  home: LiveHomeInfo | null
  cpu: LiveCpuInfo | null
  ram: LiveRamInfo | null
  gpu: LiveGpuInfo[] | null
  network: NetworkIfaceStats[] | null
  disks: DiskLiveInfo[] | null
  deviceInventory: DeviceInventoryInfo | null
}

interface GpuEngineHistory {
  threeD: number[]
  copy: number[]
  encode: number[]
  decode: number[]
  highPriority3d: number[]
  highPriorityCompute: number[]
  dedicatedPct: number[]
  sharedMb: number[]
}

interface LiveHistoryMap {
  cpuHistory: number[]
  ramHistory: number[]
  gpuHistory: Record<number, GpuEngineHistory>
  diskActiveHistory: Record<string, number[]>
  networkThroughputHistory: Record<string, number[]>
}

interface LiveSystemStoreState extends LiveSliceMap, LiveHistoryMap {
  errors: Partial<Record<LiveSliceKey, string>>
  fetching: Partial<Record<LiveSliceKey, boolean>>
  setError: (slice: LiveSliceKey, error: string | null) => void
  setFetching: (slice: LiveSliceKey, fetching: boolean) => void
  setSliceData: <T extends LiveSliceKey>(slice: T, data: NonNullable<LiveSliceMap[T]>) => void
}

interface LiveSliceState<T> {
  data: T | null
  error: string | null
  isFetching: boolean
  retry: () => void
}

const POLL_INTERVAL_MS = 1000

const subscribers: Record<LiveSliceKey, number> = {
  home: 0,
  cpu: 0,
  ram: 0,
  gpu: 0,
  network: 0,
  disks: 0,
  deviceInventory: 0,
}

const timers: Partial<Record<LiveSliceKey, ReturnType<typeof setInterval>>> = {}
const inflight: Partial<Record<LiveSliceKey, Promise<void>>> = {}

const fetchers: Record<LiveSliceKey, () => Promise<unknown>> = {
  home: getLiveHomeInfo,
  cpu: getLiveCpuInfo,
  ram: getLiveRamInfo,
  gpu: getLiveGpuInfo,
  network: getLiveNetworkInfo,
  disks: getLiveDiskInfo,
  deviceInventory: getDeviceInventoryInfo,
}

export const useLiveSystemStore = create<LiveSystemStoreState>()(set => ({
  home: null,
  cpu: null,
  ram: null,
  gpu: null,
  network: null,
  disks: null,
  deviceInventory: null,
  errors: {},
  fetching: {},
  cpuHistory: [],
  ramHistory: [],
  gpuHistory: {},
  diskActiveHistory: {},
  networkThroughputHistory: {},
  setError: (slice, error) => {
    set(state => ({
      errors: {
        ...state.errors,
        [slice]: error ?? undefined,
      },
    }))
  },
  setFetching: (slice, fetching) => {
    set(state => ({
      fetching: {
        ...state.fetching,
        [slice]: fetching,
      },
    }))
  },
  setSliceData: (slice, data) => {
    set((state) => {
      const next: Partial<LiveHistoryMap> = {}

      if (slice === 'cpu') {
        const d = data as LiveCpuInfo
        next.cpuHistory = [...state.cpuHistory, d.cpuUsagePercent].slice(-MAX_HISTORY)
      }

      if (slice === 'ram') {
        const d = data as LiveRamInfo
        const gb = d.ramUsedBytes / 1024 ** 3
        next.ramHistory = [...state.ramHistory, gb].slice(-MAX_HISTORY)
      }

      if (slice === 'gpu') {
        const gpus = data as LiveGpuInfo[]
        const prevGpuHistory = state.gpuHistory
        next.gpuHistory = Object.fromEntries(gpus.map((gpu) => {
          const prev: GpuEngineHistory = prevGpuHistory[gpu.index] ?? {
            threeD: [],
            copy: [],
            encode: [],
            decode: [],
            highPriority3d: [],
            highPriorityCompute: [],
            dedicatedPct: [],
            sharedMb: [],
          }
          const dedicatedBudget = gpu.vramTotalMb - gpu.vramReservedMb
          const dedicatedPct = dedicatedBudget > 0
            ? Math.min(100, (gpu.vramUsedMb / dedicatedBudget) * 100)
            : 0
          return [gpu.index, {
            threeD: [...prev.threeD, gpu.util3d].slice(-MAX_HISTORY),
            copy: [...prev.copy, gpu.utilCopy].slice(-MAX_HISTORY),
            encode: [...prev.encode, gpu.utilEncode].slice(-MAX_HISTORY),
            decode: [...prev.decode, gpu.utilDecode].slice(-MAX_HISTORY),
            highPriority3d: [...prev.highPriority3d, gpu.utilHighPriority3d].slice(-MAX_HISTORY),
            highPriorityCompute: [...prev.highPriorityCompute, gpu.utilHighPriorityCompute].slice(-MAX_HISTORY),
            dedicatedPct: [...prev.dedicatedPct, dedicatedPct].slice(-MAX_HISTORY),
            sharedMb: [...prev.sharedMb, gpu.vramSharedMb].slice(-MAX_HISTORY),
          }]
        }))
      }

      if (slice === 'disks') {
        const disks = data as DiskLiveInfo[]
        const nextActive: Record<string, number[]> = { ...state.diskActiveHistory }
        for (const disk of disks) {
          nextActive[disk.mountPoint] = [...(nextActive[disk.mountPoint] ?? []), disk.activeTimePercent].slice(-MAX_HISTORY)
        }
        next.diskActiveHistory = nextActive
      }

      if (slice === 'network') {
        const ifaces = data as NetworkIfaceStats[]
        const nextThroughput: Record<string, number[]> = { ...state.networkThroughputHistory }
        for (const iface of ifaces) {
          const throughput = iface.rxBytesPerSec + iface.txBytesPerSec
          nextThroughput[iface.name] = [...(nextThroughput[iface.name] ?? []), throughput].slice(-MAX_HISTORY)
        }
        next.networkThroughputHistory = nextThroughput
      }

      return {
        ...state,
        [slice]: data,
        ...next,
        errors: { ...state.errors, [slice]: undefined },
      }
    })
  },
}))

function getErrorMessage(error: unknown): string {
  if (error instanceof Error && error.message) {
    return error.message
  }

  if (typeof error === 'string' && error) {
    return error
  }

  return 'Failed to load live system information.'
}

function fetchSlice(slice: LiveSliceKey) {
  const existing = inflight[slice]
  if (existing) {
    return existing
  }

  const store = useLiveSystemStore.getState()
  store.setFetching(slice, true)

  const request = fetchers[slice]()
    .then((data) => {
      useLiveSystemStore.getState().setSliceData(slice, data as never)
    })
    .catch((error) => {
      console.error(error)
      useLiveSystemStore.getState().setError(slice, getErrorMessage(error))
    })
    .finally(() => {
      useLiveSystemStore.getState().setFetching(slice, false)
      delete inflight[slice]
    })

  inflight[slice] = request
  return request
}

function startPolling(slice: LiveSliceKey) {
  if (timers[slice]) {
    return
  }

  void fetchSlice(slice)
  timers[slice] = setInterval(() => {
    void fetchSlice(slice)
  }, POLL_INTERVAL_MS)
}

function stopPolling(slice: LiveSliceKey) {
  const timer = timers[slice]
  if (!timer) {
    return
  }

  clearInterval(timer)
  delete timers[slice]
}

function retainSlice(slice: LiveSliceKey) {
  subscribers[slice] += 1
  startPolling(slice)
}

function releaseSlice(slice: LiveSliceKey) {
  subscribers[slice] = Math.max(0, subscribers[slice] - 1)
  if (subscribers[slice] === 0) {
    stopPolling(slice)
  }
}

function useLiveSlice<T>(slice: LiveSliceKey, selector: (state: LiveSystemStoreState) => T | null): LiveSliceState<T> {
  const data = useLiveSystemStore(selector)
  const error = useLiveSystemStore(state => state.errors[slice] ?? null)
  const isFetching = useLiveSystemStore(state => state.fetching[slice] ?? false)

  // INVARIANT: `slice` must be a mount-time constant — this hook is only
  // ever called from typed wrappers (useLiveCpu, useLiveRam, …) that always
  // pass a literal string. Never call useLiveSlice with a dynamic variable.
  useMountEffect(() => {
    retainSlice(slice)
    return () => releaseSlice(slice)
  })

  return {
    data,
    error,
    isFetching,
    retry: () => {
      void fetchSlice(slice)
    },
  }
}

export function useLiveHome() {
  return useLiveSlice('home', state => state.home)
}

export function useLiveCpu() {
  return { ...useLiveSlice('cpu', state => state.cpu), history: useLiveSystemStore(s => s.cpuHistory) }
}

export function useLiveRam() {
  return { ...useLiveSlice('ram', state => state.ram), history: useLiveSystemStore(s => s.ramHistory) }
}

export function useLiveGpu() {
  return { ...useLiveSlice('gpu', state => state.gpu), history: useLiveSystemStore(s => s.gpuHistory) }
}

export function useLiveNetwork() {
  return {
    ...useLiveSlice('network', state => state.network),
    throughputHistory: useLiveSystemStore(s => s.networkThroughputHistory),
  }
}

export function useLiveDisks() {
  return {
    ...useLiveSlice('disks', state => state.disks),
    activeHistory: useLiveSystemStore(s => s.diskActiveHistory),
  }
}

export function useDeviceInventory() {
  return useLiveSlice('deviceInventory', state => state.deviceInventory)
}
