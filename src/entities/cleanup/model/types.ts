export type CleanupCategoryId
  = | 'app_cache'
    | 'browser_cache'
    | 'driver_cache'
    | 'game_cache'
    | 'system_error_reports'
    | 'thumbnail_cache'
    | 'unused_devices'
    | 'windows_logs'
    | 'windows_temp'

export type CleanupEntryStatus = 'busy' | 'clean' | 'failed' | 'pending'

export interface CleanupEntry {
  error: string | null
  iconDataUrl: string | null
  id: string
  name: string
  path: string
  sizeBytes: number
  status: CleanupEntryStatus
}

export interface CleanupCategoryReport {
  entries: CleanupEntry[]
  id: CleanupCategoryId
}

export interface CleanupAccessEntry {
  error: string | null
  id: string
  name: string
  path: string
  success: boolean
}

export interface CleanupAccessReport {
  entries: CleanupAccessEntry[]
}
