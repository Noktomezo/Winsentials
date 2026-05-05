import type { BackupEntry } from '@/entities/backup/model/types'
import {
  ArchiveRestore,
  ChevronDown,
  Loader2,
  Pencil,
  Trash2,
} from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'
import {
  createBackup,
  deleteBackup,
  listBackups,
  renameBackup,
  restoreBackup,
} from '@/entities/backup/api'
import { useMountEffect } from '@/shared/lib/hooks/use-mount-effect'
import { cn } from '@/shared/lib/utils'
import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Input,
  ScrollArea,
  Skeleton,
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/shared/ui'

function BackupPage() {
  const { t, i18n } = useTranslation()

  const [backups, setBackups] = useState<BackupEntry[]>([])
  const [loadError, setLoadError] = useState(false)
  const [isLoading, setIsLoading] = useState(true)

  // Create
  const [showCreate, setShowCreate] = useState(false)
  const [createLabel, setCreateLabel] = useState('')
  const [isCreating, setIsCreating] = useState(false)

  // Rename
  const [renameTarget, setRenameTarget] = useState<BackupEntry | null>(null)
  const [renameLabel, setRenameLabel] = useState('')
  const [isRenaming, setIsRenaming] = useState(false)

  // Delete
  const [deleteTarget, setDeleteTarget] = useState<BackupEntry | null>(null)
  const [isDeleting, setIsDeleting] = useState(false)

  // Apply
  const [applyTarget, setApplyTarget] = useState<BackupEntry | null>(null)
  const [isApplying, setIsApplying] = useState(false)

  // Expanded tweak-value panels
  const [expandedCards, setExpandedCards] = useState<Set<string>>(new Set())

  useMountEffect(() => {
    void loadBackups()

    const handleCreateRequest = () => setShowCreate(true)
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

  function formatDate(iso: string) {
    const date = new Date(iso)
    if (Number.isNaN(date.getTime())) return iso
    return date.toLocaleString(i18n.language || undefined)
  }

  return (
    <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
      {/* Backup list */}
      {isLoading
        ? (
            <div className="grid gap-3">
              {[0, 1, 2].map(i => (
                <Skeleton key={i} className="h-24 w-full rounded-lg" />
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
                    const tweakEntries = Object.entries(backup.tweaks)
                    const panelId = `backup-panel-${backup.filename.replace(/[^\w-]/g, '-')}`
                    const isCardBusy = (applyTarget?.filename === backup.filename && isApplying)
                      || (deleteTarget?.filename === backup.filename && isDeleting)
                      || (renameTarget?.filename === backup.filename && isRenaming)

                    return (
                      <section key={backup.filename} className="flex h-fit flex-col overflow-hidden rounded-lg border border-border/70 bg-card">
                        <div className="flex items-center gap-3 p-4">
                          <button
                            className="flex min-w-0 flex-1 cursor-pointer items-center gap-3 text-left"
                            onClick={() => toggleExpand(backup.filename)}
                            type="button"
                          >
                            <span className="ui-soft-surface flex size-9 shrink-0 items-center justify-center rounded-md">
                              <ArchiveRestore className="size-4" />
                            </span>
                            <div className="min-w-0 flex-1">
                              <h2 className="truncate text-sm font-medium text-foreground">
                                {backup.label}
                              </h2>
                              <div className="mt-1 flex flex-wrap items-center gap-2">
                                <span className="rounded-md border border-border/60 bg-accent/45 px-1.5 py-0.5 text-[11px] text-muted-foreground">
                                  {formatDate(backup.createdAt)}
                                </span>
                                <span className="rounded-md border border-border/60 bg-accent/45 px-1.5 py-0.5 text-[11px] text-muted-foreground">
                                  {tweakEntries.length}
                                  {' '}
                                  {t('backup.tweakValues').toLowerCase()}
                                </span>
                              </div>
                            </div>
                            <ChevronDown className={cn('size-4 shrink-0 text-muted-foreground transition-transform', expanded && 'rotate-180')} />
                          </button>
                          <div className="flex items-center gap-2">
                            <Button
                              disabled={isCardBusy}
                              onClick={(e) => {
                                e.stopPropagation()
                                setApplyTarget(backup)
                              }}
                            >
                              {applyTarget?.filename === backup.filename && isApplying
                                ? <Loader2 className="size-4 animate-spin" />
                                : <ArchiveRestore className="size-4" />}
                              {t('backup.apply')}
                            </Button>
                            <Tooltip>
                              <TooltipTrigger asChild>
                                <Button
                                  aria-label={t('backup.rename')}
                                  size="icon"
                                  variant="ghost"
                                  className="ui-soft-surface transition-colors hover:bg-accent/50!"
                                  disabled={isCardBusy}
                                  onClick={(e) => {
                                    e.stopPropagation()
                                    setRenameTarget(backup)
                                    setRenameLabel(backup.label)
                                  }}
                                >
                                  <Pencil className="size-4" />
                                </Button>
                              </TooltipTrigger>
                              <TooltipContent sideOffset={8}>{t('backup.rename')}</TooltipContent>
                            </Tooltip>
                            <Tooltip>
                              <TooltipTrigger asChild>
                                <Button
                                  aria-label={t('backup.delete')}
                                  size="icon"
                                  variant="ghost"
                                  className="ui-soft-surface transition-colors hover:border-destructive/30! hover:bg-destructive/10! hover:text-destructive!"
                                  disabled={isCardBusy}
                                  onClick={(e) => {
                                    e.stopPropagation()
                                    setDeleteTarget(backup)
                                  }}
                                >
                                  <Trash2 className="size-4" />
                                </Button>
                              </TooltipTrigger>
                              <TooltipContent sideOffset={8}>{t('backup.delete')}</TooltipContent>
                            </Tooltip>
                          </div>
                        </div>

                        {expanded && (
                          <div className="border-t border-border/70 p-3">
                            <ScrollArea
                              className="h-48 rounded-lg border border-border/50 bg-muted/30"
                              data-lenis-prevent
                              id={panelId}
                            >
                              <table className="w-full text-xs">
                                <thead className="sr-only">
                                  <tr>
                                    <th scope="col">{t('backup.key')}</th>
                                    <th scope="col">{t('backup.value')}</th>
                                  </tr>
                                </thead>
                                <tbody>
                                  {tweakEntries.map(([id, value]) => (
                                    <tr
                                      key={id}
                                      className="border-b border-border/30 last:border-0"
                                    >
                                      <td className="px-3 py-1.5 font-mono text-muted-foreground">
                                        {id}
                                      </td>
                                      <td className="px-3 py-1.5 text-right font-medium">
                                        {value}
                                      </td>
                                    </tr>
                                  ))}
                                </tbody>
                              </table>
                            </ScrollArea>
                          </div>
                        )}
                      </section>
                    )
                  })}
                </div>
              )}

      {/* Create dialog */}
      <Dialog
        open={showCreate}
        onOpenChange={(open) => {
          if (!open) {
            setShowCreate(false)
            setCreateLabel('')
          }
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('backup.createSnapshot')}</DialogTitle>
            <DialogDescription>
              {t('backup.labelPlaceholder')}
            </DialogDescription>
          </DialogHeader>
          <div className="px-5">
            <Input
              aria-label={t('backup.createSnapshot')}
              value={createLabel}
              onChange={e => setCreateLabel(e.target.value)}
              placeholder={t('backup.labelPlaceholder')}
              onKeyDown={e =>
                e.key === 'Enter' && !isCreating && void handleCreate()}
            />
          </div>
          <DialogFooter>
            <Button
              variant="outline"
              onClick={() => {
                setShowCreate(false)
                setCreateLabel('')
              }}
            >
              {t('dialog.close')}
            </Button>
            <Button onClick={() => void handleCreate()} disabled={isCreating}>
              {t('backup.createSnapshot')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Rename dialog */}
      <Dialog
        open={renameTarget !== null}
        onOpenChange={(open) => {
          if (!open) {
            setRenameTarget(null)
          }
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('backup.rename')}</DialogTitle>
          </DialogHeader>
          <div className="px-5">
            <Input
              aria-label={t('backup.rename')}
              value={renameLabel}
              onChange={e => setRenameLabel(e.target.value)}
              onKeyDown={e =>
                e.key === 'Enter' && !isRenaming && void handleRename()}
            />
          </div>
          <DialogFooter>
            <Button variant="outline" onClick={() => setRenameTarget(null)}>
              {t('dialog.close')}
            </Button>
            <Button
              onClick={() => void handleRename()}
              disabled={isRenaming || renameLabel.trim() === ''}
            >
              {t('backup.rename')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Delete dialog */}
      <Dialog
        open={deleteTarget !== null}
        onOpenChange={(open) => {
          if (!open) {
            setDeleteTarget(null)
          }
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('backup.delete')}</DialogTitle>
            <DialogDescription>{t('backup.deleteConfirm')}</DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setDeleteTarget(null)}>
              {t('dialog.close')}
            </Button>
            <Button
              variant="destructive"
              onClick={() => void handleDelete()}
              disabled={isDeleting}
            >
              {t('backup.delete')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* Apply dialog */}
      <Dialog
        open={applyTarget !== null}
        onOpenChange={(open) => {
          if (!open) {
            setApplyTarget(null)
          }
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('backup.apply')}</DialogTitle>
            <DialogDescription>{t('backup.applyConfirm')}</DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={() => setApplyTarget(null)}>
              {t('dialog.close')}
            </Button>
            <Button onClick={() => void handleApply()} disabled={isApplying}>
              {t('backup.apply')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </section>
  )
}

export default BackupPage
