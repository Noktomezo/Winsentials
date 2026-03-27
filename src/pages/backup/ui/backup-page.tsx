import type { BackupEntry } from '@/entities/backup/model/types'
import { ArchiveRestore, ChevronDown, ChevronUp, Pencil, Plus, Trash2 } from 'lucide-react'
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
import {
  Button,
  Card,
  CardAction,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Input,
  ScrollArea,
  Separator,
  Skeleton,
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/shared/ui'

export function BackupPage() {
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
    if (!renameTarget) { return }
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
    if (!deleteTarget) { return }
    setIsDeleting(true)
    try {
      await deleteBackup(deleteTarget.filename)
      setBackups(prev => prev.filter(b => b.filename !== deleteTarget.filename))
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
    if (!applyTarget) { return }
    setIsApplying(true)
    try {
      const report = await restoreBackup(applyTarget.filename)
      setApplyTarget(null)
      if (report.failed.length === 0) {
        toast.success(t('backup.snapshotRestored', { applied: report.applied }))
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

    if (Number.isNaN(date.getTime())) {
      return iso
    }

    return date.toLocaleString(i18n.language || undefined)
  }

  return (
    <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
      {/* Action bar */}
      <div className="flex items-center">
        <Button onClick={() => setShowCreate(true)} size="sm">
          <Plus className="size-4" />
          {t('backup.createSnapshot')}
        </Button>
      </div>

      {/* Backup list */}
      {isLoading
        ? (
            <div className="grid gap-3">
              {[0, 1, 2].map(i => (
                <Skeleton key={i} className="h-24 w-full rounded-xl" />
              ))}
            </div>
          )
        : loadError
          ? (
              <div className="flex flex-1 flex-col items-center justify-center gap-3 text-muted-foreground">
                <ArchiveRestore className="size-10 opacity-40" />
                <p className="text-sm">{t('backup.errors.load')}</p>
                <Button onClick={() => void loadBackups()} size="sm" type="button" variant="outline">
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
                <div className="grid gap-3">
                  {backups.map((backup) => {
                    const expanded = expandedCards.has(backup.filename)
                    const tweakEntries = Object.entries(backup.tweaks)
                    const panelId = `backup-panel-${backup.filename.replace(/[^\w-]/g, '-')}`
                    return (
                      <Card key={backup.filename} className="gap-0 p-4">
                        <CardHeader className="p-0">
                          <CardTitle className="truncate text-sm">{backup.label}</CardTitle>
                          <CardDescription className="text-xs">{formatDate(backup.createdAt)}</CardDescription>
                          <CardAction className="flex items-center gap-1.5">
                            <Button
                              size="sm"
                              variant="outline"
                              onClick={() => setApplyTarget(backup)}
                            >
                              <ArchiveRestore className="size-3.5" />
                              {t('backup.apply')}
                            </Button>
                            <Tooltip>
                              <TooltipTrigger asChild>
                                <Button
                                  aria-label={t('backup.rename')}
                                  size="icon"
                                  variant="ghost"
                                  className="size-8"
                                  onClick={() => {
                                    setRenameTarget(backup)
                                    setRenameLabel(backup.label)
                                  }}
                                >
                                  <Pencil className="size-3.5" />
                                </Button>
                              </TooltipTrigger>
                              <TooltipContent>{t('backup.rename')}</TooltipContent>
                            </Tooltip>
                            <Tooltip>
                              <TooltipTrigger asChild>
                                <Button
                                  aria-label={t('backup.delete')}
                                  size="icon"
                                  variant="ghost"
                                  className="size-8 text-destructive hover:text-destructive"
                                  onClick={() => setDeleteTarget(backup)}
                                >
                                  <Trash2 className="size-3.5" />
                                </Button>
                              </TooltipTrigger>
                              <TooltipContent>{t('backup.delete')}</TooltipContent>
                            </Tooltip>
                          </CardAction>
                        </CardHeader>

                        <CardContent className="p-0 pt-3">
                          <Separator className="mb-3" />
                          <Button
                            aria-controls={panelId}
                            aria-expanded={expanded}
                            variant="ghost"
                            size="sm"
                            className="h-auto gap-1 px-0 py-0 text-xs text-muted-foreground hover:bg-transparent hover:text-foreground"
                            onClick={() => toggleExpand(backup.filename)}
                          >
                            {expanded
                              ? <ChevronUp className="size-3.5" />
                              : <ChevronDown className="size-3.5" />}
                            {t('backup.tweakValues')}
                            {' '}
                            (
                            {tweakEntries.length}
                            )
                          </Button>

                          {expanded && (
                            <ScrollArea className="mt-2 h-48 rounded-lg border border-border/50 bg-muted/30" data-lenis-prevent id={panelId}>
                              <table className="w-full text-xs">
                                <thead className="sr-only">
                                  <tr>
                                    <th scope="col">{t('backup.key')}</th>
                                    <th scope="col">{t('backup.value')}</th>
                                  </tr>
                                </thead>
                                <tbody>
                                  {tweakEntries.map(([id, value]) => (
                                    <tr key={id} className="border-b border-border/30 last:border-0">
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
                          )}
                        </CardContent>
                      </Card>
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
            <DialogDescription>{t('backup.labelPlaceholder')}</DialogDescription>
          </DialogHeader>
          <Input
            aria-label={t('backup.createSnapshot')}
            value={createLabel}
            onChange={e => setCreateLabel(e.target.value)}
            placeholder={t('backup.labelPlaceholder')}
            onKeyDown={e => e.key === 'Enter' && !isCreating && void handleCreate()}
          />
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
          if (!open) { setRenameTarget(null) }
        }}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('backup.rename')}</DialogTitle>
          </DialogHeader>
          <Input
            aria-label={t('backup.rename')}
            value={renameLabel}
            onChange={e => setRenameLabel(e.target.value)}
            onKeyDown={e => e.key === 'Enter' && !isRenaming && void handleRename()}
          />
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
          if (!open) { setDeleteTarget(null) }
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
          if (!open) { setApplyTarget(null) }
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
