import type { CleanupCategoryId, CleanupCategoryReport, ReportMap } from './types'
import { useSyncExternalStore } from 'react'
import { scanAllCleanupCategories, scanCleanupCategory } from '@/entities/cleanup/api'

interface CleanupScanSnapshot {
  reports: ReportMap
  isLoading: boolean
}

const listeners = new Set<() => void>()

let reports: ReportMap = {}
let isLoading = false
let inflight: Promise<void> | null = null
let pendingForced: Promise<void> | null = null

let snapshot: CleanupScanSnapshot = {
  reports,
  isLoading,
}

function emitChange() {
  snapshot = {
    reports: { ...reports },
    isLoading,
  }
  listeners.forEach(listener => listener())
}

function setReports(updater: (current: ReportMap) => ReportMap) {
  reports = updater(reports)
  emitChange()
}

function reportMapFromReports(newReports: CleanupCategoryReport[]): ReportMap {
  return Object.fromEntries(newReports.map(report => [report.id as CleanupCategoryId, report])) as ReportMap
}

function loadAllCleanupReports(force = false): Promise<void> {
  if (!force && Object.keys(reports).length > 0) {
    return Promise.resolve()
  }

  if (inflight) {
    if (!force) {
      return inflight
    }
    if (!pendingForced) {
      pendingForced = inflight.then(() => {
        pendingForced = null
        return loadAllCleanupReports(true)
      })
    }
    return pendingForced
  }

  isLoading = true
  emitChange()

  inflight = scanAllCleanupCategories()
    .then((newReports) => {
      reports = reportMapFromReports(newReports)
      isLoading = false
    })
    .catch((error) => {
      console.error(error)
      isLoading = false
    })
    .finally(() => {
      inflight = null
      emitChange()
    })

  return inflight
}

function refreshCleanupCategory(categoryId: CleanupCategoryId): Promise<void> {
  return scanCleanupCategory(categoryId)
    .then((report) => {
      setReports(current => ({ ...current, [report.id]: report }))
    })
    .catch((error) => {
      console.error(error)
    })
}

function refreshAllCleanupReports(): Promise<void> {
  return loadAllCleanupReports(true)
}

function setCleanupReport(report: CleanupCategoryReport) {
  setReports(current => ({ ...current, [report.id as CleanupCategoryId]: report }))
}

function setCleanupReports(newReports: CleanupCategoryReport[]) {
  setReports(current => ({ ...current, ...reportMapFromReports(newReports) }))
}

function subscribeCleanupReports(callback: () => void) {
  listeners.add(callback)
  void loadAllCleanupReports()
  return () => {
    listeners.delete(callback)
  }
}

function getCleanupReportsSnapshot() {
  return snapshot
}

export function useCleanupReports() {
  const state = useSyncExternalStore(
    subscribeCleanupReports,
    getCleanupReportsSnapshot,
    getCleanupReportsSnapshot,
  )

  return {
    ...state,
    retry: () => {
      void loadAllCleanupReports(true)
    },
  }
}

export const cleanupScanCache = {
  refreshCategory: refreshCleanupCategory,
  refreshAll: refreshAllCleanupReports,
  setReport: setCleanupReport,
  setReports: setCleanupReports,
}
