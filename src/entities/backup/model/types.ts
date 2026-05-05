export interface BackupMeta {
  filename: string
  label: string
  createdAt: string
}

export interface RestoreReport {
  applied: number
  failed: string[]
}

export interface BackupEntry extends BackupMeta {
  tweaks: Record<string, string>
}
