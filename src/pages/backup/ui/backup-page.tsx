import type { BackupEntry } from '@/entities/backup/model/types'
import { ArchiveRestore } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'
import { listBackups } from '@/entities/backup/api'
import { useMountEffect } from '@/shared/lib/hooks/use-mount-effect'
import { Button, Skeleton } from '@/shared/ui'
import { BackupCard } from './backup-card'
import { BackupDialogs } from './backup-dialogs'
import { useBackupDialogs } from './use-backup-dialogs'

function BackupPage() {
  const { t, i18n } = useTranslation()

  const [backups, setBackups] = useState<BackupEntry[]>([])
  const [loadError, setLoadError] = useState(false)
  const [isLoading, setIsLoading] = useState(true)
  const [expandedCards, setExpandedCards] = useState<Set<string>>(new Set())

  const dialogs = useBackupDialogs(setBackups, setExpandedCards, t)

  useMountEffect(() => {
    void loadBackups()

    const handleCreateRequest = () => dialogs.setShowCreate(true)
    window.addEventListener('winsentials:backup-create', handleCreateRequest)
    return () =>
      window.removeEventListener('winsentials:backup-create', handleCreateRequest)
  })

  async function loadBackups() {
    setLoadError(false)
    setIsLoading(true)

    try {
      setBackups(await listBackups())
    }
    catch {
      setLoadError(true)
      toast.error(t('backup.errors.load'))
    }
    finally {
      setIsLoading(false)
    }
  }

  function toggleExpand(filename: string) {
    setExpandedCards((prev) => {
      const next = new Set(prev)
      if (next.has(filename)) {
        next.delete(filename)
      }
      else {
        next.add(filename)
      }
      return next
    })
  }

  function formatDate(iso: string) {
    const date = new Date(iso)
    if (Number.isNaN(date.getTime())) return iso
    return date.toLocaleString(i18n.language || undefined)
  }

  return (
    <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
      {isLoading
        ? (
            <div className="grid gap-3">
              {['backup-skeleton-primary', 'backup-skeleton-secondary', 'backup-skeleton-tertiary'].map(key => (
                <Skeleton key={key} className="h-24 w-full rounded-lg" />
              ))}
            </div>
          )
        : loadError
          ? (
              <div className="flex flex-1 flex-col items-center justify-center gap-3 text-muted-foreground">
                <ArchiveRestore className="size-10 opacity-40" />
                <p className="text-sm">{t('backup.errors.load')}</p>
                <Button
                  onClick={() => void loadBackups()}
                  size="sm"
                  type="button"
                  variant="outline"
                >
                  {t('tweaks.actions.retry')}
                </Button>
              </div>
            )
          : backups.length === 0
            ? (
                <div className="flex flex-1 flex-col items-center justify-center gap-2 text-muted-foreground">
                  <ArchiveRestore className="size-10 opacity-40" />
                  <p className="text-sm">{t('backup.noSnapshots')}</p>
                </div>
              )
            : (
                <div className="tweak-card-grid">
                  {backups.map((backup) => {
                    const expanded = expandedCards.has(backup.filename)
                    const isCardBusy = (dialogs.applyTarget?.filename === backup.filename && dialogs.isApplying)
                      || (dialogs.deleteTarget?.filename === backup.filename && dialogs.isDeleting)
                      || (dialogs.renameTarget?.filename === backup.filename && dialogs.isRenaming)
                    const isApplyLoading = dialogs.applyTarget?.filename === backup.filename && dialogs.isApplying

                    return (
                      <BackupCard
                        key={backup.filename}
                        backup={backup}
                        expanded={expanded}
                        isCardBusy={isCardBusy}
                        isApplyLoading={isApplyLoading}
                        onToggleExpand={toggleExpand}
                        onApply={dialogs.setApplyTarget}
                        onRename={(b) => {
                          dialogs.setRenameTarget(b)
                          dialogs.setRenameLabel(b.label)
                        }}
                        onDelete={dialogs.setDeleteTarget}
                        formatDate={formatDate}
                        t={t}
                      />
                    )
                  })}
                </div>
              )}

      <BackupDialogs
        showCreate={dialogs.showCreate}
        createLabel={dialogs.createLabel}
        isCreating={dialogs.isCreating}
        setShowCreate={dialogs.setShowCreate}
        setCreateLabel={dialogs.setCreateLabel}
        handleCreate={dialogs.handleCreate}
        renameTarget={dialogs.renameTarget}
        renameLabel={dialogs.renameLabel}
        isRenaming={dialogs.isRenaming}
        setRenameTarget={dialogs.setRenameTarget}
        setRenameLabel={dialogs.setRenameLabel}
        handleRename={dialogs.handleRename}
        deleteTarget={dialogs.deleteTarget}
        isDeleting={dialogs.isDeleting}
        setDeleteTarget={dialogs.setDeleteTarget}
        handleDelete={dialogs.handleDelete}
        applyTarget={dialogs.applyTarget}
        isApplying={dialogs.isApplying}
        setApplyTarget={dialogs.setApplyTarget}
        handleApply={dialogs.handleApply}
        t={t}
      />
    </section>
  )
}

export default BackupPage
