import type { CleanupAccessReport, CleanupCategoryId, CleanupCategoryReport, CleanupEntry } from '@/entities/cleanup/model/types'
import { invoke } from '@tauri-apps/api/core'

interface BackendCleanupEntry {
  error: string | null
  icon_data_url: string | null
  id: string
  name: string
  path: string
  size_bytes: number
  status: CleanupEntry['status']
}

interface BackendCleanupCategoryReport {
  entries: BackendCleanupEntry[]
  id: CleanupCategoryId
}

interface BackendCleanupAccessReport extends CleanupAccessReport {}
function mapEntry(entry: BackendCleanupEntry): CleanupEntry {
  return {
    error: entry.error,
    iconDataUrl: entry.icon_data_url,
    id: entry.id,
    name: entry.name,
    path: entry.path,
    sizeBytes: entry.size_bytes,
    status: entry.status,
  }
}

function mapReport(report: BackendCleanupCategoryReport): CleanupCategoryReport {
  return {
    entries: report.entries.map(mapEntry),
    id: report.id,
  }
}

export async function scanCleanupCategory(categoryId: CleanupCategoryId): Promise<CleanupCategoryReport> {
  const report = await invoke<BackendCleanupCategoryReport>('cleanup_scan_category', { categoryId })
  return mapReport(report)
}

export async function cleanCleanupCategory(categoryId: CleanupCategoryId): Promise<CleanupCategoryReport> {
  const report = await invoke<BackendCleanupCategoryReport>('cleanup_clean_category', { categoryId })
  return mapReport(report)
}

export async function prepareCleanupAccess(): Promise<CleanupAccessReport> {
  return await invoke<BackendCleanupAccessReport>('cleanup_prepare_access')
}
