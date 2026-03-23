import type { StaticSystemInfo } from '@/entities/system-info/model/types'
import { useSyncExternalStore } from 'react'
import { getStaticSystemInfo } from '@/entities/system-info/api'

interface StaticSystemInfoSnapshot {
  info: StaticSystemInfo | null
  error: unknown | null
  isLoading: boolean
}

const listeners = new Set<() => void>()

let snapshot: StaticSystemInfoSnapshot = {
  info: null,
  error: null,
  isLoading: false,
}

let inflight: Promise<void> | null = null

function emitChange() {
  listeners.forEach(listener => listener())
}

function loadStaticSystemInfo(force = false): Promise<void> {
  if (snapshot.info && !force) {
    return Promise.resolve()
  }

  if (inflight) {
    return inflight
  }

  snapshot = {
    ...snapshot,
    error: null,
    isLoading: true,
  }
  emitChange()

  inflight = getStaticSystemInfo()
    .then((info) => {
      snapshot = {
        info,
        error: null,
        isLoading: false,
      }
    })
    .catch((error) => {
      console.error(error)
      snapshot = {
        ...snapshot,
        error,
        isLoading: false,
      }
    })
    .finally(() => {
      inflight = null
      emitChange()
    })

  return inflight
}

function subscribeStaticSystemInfo(callback: () => void) {
  listeners.add(callback)
  void loadStaticSystemInfo()
  return () => {
    listeners.delete(callback)
  }
}

function getStaticSystemInfoSnapshot() {
  return snapshot
}

export function useStaticSystemInfo() {
  const state = useSyncExternalStore(
    subscribeStaticSystemInfo,
    getStaticSystemInfoSnapshot,
    getStaticSystemInfoSnapshot,
  )

  return {
    ...state,
    retry: () => {
      void loadStaticSystemInfo(true)
    },
  }
}

export function useStaticInfo() {
  return useStaticSystemInfo().info
}
