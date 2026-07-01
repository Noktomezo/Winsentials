import type { CategoryCleanRequest, CleanupCategoryId, CleanupCategoryReport, WinappDbStatus } from '@/entities/cleanup/model/types'
import { invoke } from '@tauri-apps/api/core'

export async function scanCleanupCategory(categoryId: CleanupCategoryId): Promise<CleanupCategoryReport> {
  return await invoke<CleanupCategoryReport>('cleanup_scan_category', { categoryId })
}

export async function cleanCleanupCategory(categoryId: CleanupCategoryId, excludeEntryIds: string[] = []): Promise<CleanupCategoryReport> {
  return await invoke<CleanupCategoryReport>('cleanup_clean_category', { categoryId, excludeEntryIds })
}

export async function scanAllCleanupCategories(): Promise<CleanupCategoryReport[]> {
  return await invoke<CleanupCategoryReport[]>('cleanup_scan_all')
}

export async function cleanAllCleanupCategories(requests: CategoryCleanRequest[]): Promise<CleanupCategoryReport[]> {
  return await invoke<CleanupCategoryReport[]>('cleanup_clean_all', { requests })
}

export async function updateWinappDb(): Promise<WinappDbStatus> {
  return await invoke<WinappDbStatus>('cleanup_update_winapp_db')
}

export async function getWinappDbStatus(): Promise<WinappDbStatus> {
  return await invoke<WinappDbStatus>('cleanup_winapp_db_status')
}

export async function setCustomWinapp2Path(path: string | null): Promise<WinappDbStatus> {
  return await invoke<WinappDbStatus>('cleanup_set_custom_winapp2_path', { path })
}
