import type { CleanupCategoryId } from './types'
import { useSyncExternalStore } from 'react'

interface CleanupUiSnapshot {
  busy: boolean
  refreshingCategories: ReadonlySet<CleanupCategoryId>
}

const listeners = new Set<() => void>()
let busy = false
let refreshingCategories = new Set<CleanupCategoryId>()
let snapshot: CleanupUiSnapshot = {
  busy,
  refreshingCategories: new Set(refreshingCategories),
}

function emit() {
  snapshot = { busy, refreshingCategories: new Set(refreshingCategories) }
  listeners.forEach(listener => listener())
}

function subscribe(listener: () => void) {
  listeners.add(listener)
  return () => listeners.delete(listener)
}

function getSnapshot() {
  return snapshot
}

export function useCleanupUiState() {
  return useSyncExternalStore(subscribe, getSnapshot, getSnapshot)
}

export function setCleanupBusy(nextBusy: boolean) {
  if (busy === nextBusy) return
  busy = nextBusy
  emit()
}

export function addRefreshingCategories(categoryIds: CleanupCategoryId[]) {
  const next = new Set(refreshingCategories)
  categoryIds.forEach(categoryId => next.add(categoryId))
  refreshingCategories = next
  emit()
}

export function removeRefreshingCategories(categoryIds: CleanupCategoryId[]) {
  const next = new Set(refreshingCategories)
  categoryIds.forEach(categoryId => next.delete(categoryId))
  refreshingCategories = next
  emit()
}

export function hasRefreshingCategories() {
  return refreshingCategories.size > 0
}
