export type CleanupCategoryId
  = | 'applications'
    | 'appx'
    | 'browsers'
    | 'development'
    | 'gaming'
    | 'media'
    | 'unused_devices'
    | 'windows'

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
