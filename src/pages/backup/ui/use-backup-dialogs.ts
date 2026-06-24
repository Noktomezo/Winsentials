import type { TFunction } from 'i18next'
import type { Dispatch, SetStateAction } from 'react'
import type { BackupEntry } from '@/entities/backup/model/types'
import { useState } from 'react'
import { toast } from 'sonner'
import {
  createBackup,
  deleteBackup,
  renameBackup,
  restoreBackup,
} from '@/entities/backup/api'

export function useBackupDialogs(
  setBackups: Dispatch<SetStateAction<BackupEntry[]>>,
  setExpandedCards: Dispatch<SetStateAction<Set<string>>>,
  t: TFunction,
) {
  const [showCreate, setShowCreate] = useState(false)
  const [createLabel, setCreateLabel] = useState('')
  const [isCreating, setIsCreating] = useState(false)

  const [renameTarget, setRenameTarget] = useState<BackupEntry | null>(null)
  const [renameLabel, setRenameLabel] = useState('')
  const [isRenaming, setIsRenaming] = useState(false)

  const [deleteTarget, setDeleteTarget] = useState<BackupEntry | null>(null)
  const [isDeleting, setIsDeleting] = useState(false)

  const [applyTarget, setApplyTarget] = useState<BackupEntry | null>(null)
  const [isApplying, setIsApplying] = useState(false)

  async function handleCreate() {
    setIsCreating(true)
    try {
      const entry = await createBackup(createLabel.trim() || undefined)
      setBackups(prev => [entry, ...prev])
      setShowCreate(false)
      setCreateLabel('')
      toast.success(t('backup.snapshotCreated'))
    }
    catch {
      toast.error(t('backup.errors.create'))
    }
    finally {
      setIsCreating(false)
    }
  }

  async function handleRename() {
    if (!renameTarget) return
    const newLabel = renameLabel.trim()
    if (newLabel === '') {
      toast.error(t('backup.errors.rename'))
      return
    }
    setIsRenaming(true)
    try {
      await renameBackup(renameTarget.filename, newLabel)
      setBackups(prev =>
        prev.map(b =>
          b.filename === renameTarget.filename ? { ...b, label: newLabel } : b,
        ),
      )
      setRenameTarget(null)
      toast.success(t('backup.renamed'))
    }
    catch {
      toast.error(t('backup.errors.rename'))
    }
    finally {
      setIsRenaming(false)
    }
  }

  async function handleDelete() {
    if (!deleteTarget) return

    setIsDeleting(true)
    try {
      await deleteBackup(deleteTarget.filename)
      setBackups(prev =>
        prev.filter(b => b.filename !== deleteTarget.filename),
      )
      setExpandedCards((prev) => {
        const next = new Set(prev)
        next.delete(deleteTarget.filename)
        return next
      })
      setDeleteTarget(null)
      toast.success(t('backup.deleted'))
    }
    catch {
      toast.error(t('backup.errors.delete'))
    }
    finally {
      setIsDeleting(false)
    }
  }

  async function handleApply() {
    if (!applyTarget) return

    setIsApplying(true)
    try {
      const report = await restoreBackup(applyTarget.filename)
      setApplyTarget(null)
      if (report.failed.length === 0) {
        toast.success(
          t('backup.snapshotRestored', { applied: report.applied }),
        )
      }
      else {
        toast.warning(
          t('backup.snapshotRestoredWithErrors', {
            applied: report.applied,
            failed: report.failed.length,
          }),
        )
      }
    }
    catch {
      toast.error(t('backup.errors.restore'))
    }
    finally {
      setIsApplying(false)
    }
  }

  return {
    showCreate,
    setShowCreate,
    createLabel,
    setCreateLabel,
    isCreating,
    handleCreate,
    renameTarget,
    setRenameTarget,
    renameLabel,
    setRenameLabel,
    isRenaming,
    handleRename,
    deleteTarget,
    setDeleteTarget,
    isDeleting,
    handleDelete,
    applyTarget,
    setApplyTarget,
    isApplying,
    handleApply,
  }
}
