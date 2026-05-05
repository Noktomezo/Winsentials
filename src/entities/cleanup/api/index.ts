import type { CleanupCategoryId, CleanupCategoryReport } from '@/entities/cleanup/model/types'
import { invoke } from '@tauri-apps/api/core'

export async function scanCleanupCategory(categoryId: CleanupCategoryId): Promise<CleanupCategoryReport> {
  return await invoke<CleanupCategoryReport>('cleanup_scan_category', { categoryId })
}

export async function cleanCleanupCategory(categoryId: CleanupCategoryId): Promise<CleanupCategoryReport> {
  return await invoke<CleanupCategoryReport>('cleanup_clean_category', { categoryId })
}
