import type { BackupEntry, RestoreReport } from '../model/types'
import { invoke } from '@tauri-apps/api/core'

export function listBackups(): Promise<BackupEntry[]> {
  return invoke('backup_list')
}

export function createBackup(label?: string): Promise<BackupEntry> {
  return invoke('backup_create', { label: label ?? null })
}

export function restoreBackup(filename: string): Promise<RestoreReport> {
  return invoke('backup_restore', { filename })
}

export function renameBackup(filename: string, newLabel: string): Promise<void> {
  return invoke('backup_rename', { filename, new_label: newLabel })
}

export function deleteBackup(filename: string): Promise<void> {
  return invoke('backup_delete', { filename })
}
