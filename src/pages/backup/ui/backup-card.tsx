import type { TFunction } from 'i18next'
import type { BackupEntry } from '@/entities/backup/model/types'
import {
  Check,
  ChevronDown,
  DatabaseBackup,
  Loader2,
  Pencil,
  Trash2,
} from 'lucide-react'
import { cn } from '@/shared/lib/utils'
import {
  Button,
  ScrollArea,
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/shared/ui'

interface BackupCardProps {
  backup: BackupEntry
  expanded: boolean
  isCardBusy: boolean
  isApplyLoading: boolean
  onToggleExpand: (filename: string) => void
  onApply: (backup: BackupEntry) => void
  onRename: (backup: BackupEntry) => void
  onDelete: (backup: BackupEntry) => void
  formatDate: (iso: string) => string
  t: TFunction
}

export function BackupCard({
  backup,
  expanded,
  isCardBusy,
  isApplyLoading,
  onToggleExpand,
  onApply,
  onRename,
  onDelete,
  formatDate,
  t,
}: BackupCardProps) {
  const tweakEntries = Object.entries(backup.tweaks)
  const panelId = `backup-panel-${backup.filename.replace(/[^\w-]/g, '-')}`

  return (
    <section className="flex h-fit flex-col overflow-hidden rounded-lg border border-border/70 bg-card">
      <div className="flex items-center gap-3 p-4">
        <button
          aria-controls={panelId}
          aria-expanded={expanded}
          className="flex min-w-0 flex-1 cursor-pointer items-center gap-3 text-left"
          onClick={() => onToggleExpand(backup.filename)}
          type="button"
        >
          {' '}
          <span className="ui-soft-surface flex size-9 shrink-0 items-center justify-center rounded-md">
            <DatabaseBackup className="size-4" />
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
              onApply(backup)
            }}
          >
            {isApplyLoading
              ? <Loader2 className="size-4 animate-spin" />
              : <Check className="size-4" />}
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
                  onRename(backup)
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
                  onDelete(backup)
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
}
