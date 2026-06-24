import type { TFunction } from 'i18next'
import type { BackupEntry } from '@/entities/backup/model/types'
import { Check, Loader2 } from 'lucide-react'
import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  Input,
} from '@/shared/ui'

interface BackupDialogsProps {
  showCreate: boolean
  createLabel: string
  isCreating: boolean
  setShowCreate: (show: boolean) => void
  setCreateLabel: (label: string) => void
  handleCreate: () => Promise<void>

  renameTarget: BackupEntry | null
  renameLabel: string
  isRenaming: boolean
  setRenameTarget: (target: BackupEntry | null) => void
  setRenameLabel: (label: string) => void
  handleRename: () => Promise<void>

  deleteTarget: BackupEntry | null
  isDeleting: boolean
  setDeleteTarget: (target: BackupEntry | null) => void
  handleDelete: () => Promise<void>

  applyTarget: BackupEntry | null
  isApplying: boolean
  setApplyTarget: (target: BackupEntry | null) => void
  handleApply: () => Promise<void>

  t: TFunction
}

export function BackupDialogs({
  showCreate,
  createLabel,
  isCreating,
  setShowCreate,
  setCreateLabel,
  handleCreate,
  renameTarget,
  renameLabel,
  isRenaming,
  setRenameTarget,
  setRenameLabel,
  handleRename,
  deleteTarget,
  isDeleting,
  setDeleteTarget,
  handleDelete,
  applyTarget,
  isApplying,
  setApplyTarget,
  handleApply,
  t,
}: BackupDialogsProps) {
  return (
    <>
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
              {isApplying
                ? <Loader2 className="size-4 animate-spin" />
                : <Check className="size-4" />}
              {t('backup.apply')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </>
  )
}
