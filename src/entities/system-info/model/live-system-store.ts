import type {
  DiskLiveInfo,
  LiveCpuInfo,
  LiveGpuInfo,
  LiveHomeInfo,
  LiveRamInfo,
  NetworkIfaceStats,
} from '@/entities/system-info/model/types'
import { useEffect } from 'react'
import { create } from 'zustand'
import {
  getLiveCpuInfo,
  getLiveDiskInfo,
  getLiveGpuInfo,
  getLiveHomeInfo,
  getLiveNetworkInfo,
  getLiveRamInfo,
} from '@/entities/system-info/api'

type LiveSliceKey = 'home' | 'cpu' | 'ram' | 'gpu' | 'network' | 'disks'

interface LiveSliceMap {
  home: LiveHomeInfo | null
  cpu: LiveCpuInfo | null
  ram: LiveRamInfo | null
  gpu: LiveGpuInfo[] | null
  network: NetworkIfaceStats[] | null
  disks: DiskLiveInfo[] | null
}

interface LiveSystemStoreState extends LiveSliceMap {
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
}

export const useLiveSystemStore = create<LiveSystemStoreState>()(set => ({
  home: null,
  cpu: null,
  ram: null,
  gpu: null,
  network: null,
  disks: null,
  errors: {},
  fetching: {},
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
    set(state => ({
      ...state,
      [slice]: data,
      errors: {
        ...state.errors,
        [slice]: undefined,
      },
    }))
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

  useEffect(() => {
    retainSlice(slice)
    return () => releaseSlice(slice)
  }, [slice])

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
  return useLiveSlice('cpu', state => state.cpu)
}

export function useLiveRam() {
  return useLiveSlice('ram', state => state.ram)
}

export function useLiveGpu() {
  return useLiveSlice('gpu', state => state.gpu)
}

export function useLiveNetwork() {
  return useLiveSlice('network', state => state.network)
}

export function useLiveDisks() {
  return useLiveSlice('disks', state => state.disks)
}
