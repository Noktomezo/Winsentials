export type CleanupCategoryId
  = | 'applications'
    | 'appx'
    | 'browsers'
    | 'development'
    | 'gaming'
    | 'media'
    | 'unused_devices'
    | 'windows'

export type CleanupEntryStatus = 'busy' | 'clean' | 'failed' | 'pending' | 'removed'

export interface CleanupEntry {
  error: string | null
  iconDataUrl: string | null
  id: string
  name: string
  path: string
  sizeBytes: number
  status: CleanupEntryStatus
  defaultChecked: boolean
  warning: string | null
}

export interface CleanupCategoryReport {
  entries: CleanupEntry[]
  id: CleanupCategoryId
}

export type ReportMap = Partial<Record<CleanupCategoryId, CleanupCategoryReport>>

export interface CategoryCleanRequest {
  categoryId: CleanupCategoryId
  excludeEntryIds: string[]
}

export type WinappDbSource = 'bundled' | 'cache' | 'custom'

export interface WinappDbStatus {
  source: WinappDbSource
  lastUpdated: number | null
  customPath: string | null
  cachePath: string | null
}
