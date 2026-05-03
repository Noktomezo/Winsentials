import type { LucideIcon } from 'lucide-react'
import type { CleanupCategoryId, CleanupCategoryReport, CleanupEntry, CleanupEntryStatus } from '@/entities/cleanup/model/types'
import { useVirtualizer } from '@tanstack/react-virtual'
import { Bug, Check, ChevronDown, Cpu, FileText, Gamepad2, Globe, Image, KeyRound, Loader2, PackageOpen, RefreshCw, Sparkles, Trash2, Unplug, X } from 'lucide-react'
import { useEffect, useMemo, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { cleanCleanupCategory, prepareCleanupAccess, scanCleanupCategory } from '@/entities/cleanup/api'
import { formatBytesLocalized } from '@/shared/lib/format-size'
import { useMountEffect } from '@/shared/lib/hooks/use-mount-effect'
import { toast } from '@/shared/lib/toast'
import { cn } from '@/shared/lib/utils'
import { Button } from '@/shared/ui/button'
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/shared/ui/dialog'
import { Skeleton } from '@/shared/ui/skeleton'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/shared/ui/tooltip'

interface CleanupCategoryDefinition {
  icon: LucideIcon
  id: CleanupCategoryId
}

const CLEANUP_CATEGORIES: CleanupCategoryDefinition[] = [
  { id: 'windows_temp', icon: Sparkles },
  { id: 'thumbnail_cache', icon: Image },
  { id: 'browser_cache', icon: Globe },
  { id: 'driver_cache', icon: Cpu },
  { id: 'game_cache', icon: Gamepad2 },
  { id: 'windows_logs', icon: FileText },
  { id: 'system_error_reports', icon: Bug },
  { id: 'app_cache', icon: PackageOpen },
  { id: 'unused_devices', icon: Unplug },
]

const STATUS_ICON: Record<CleanupEntryStatus, LucideIcon> = {
  busy: X,
  clean: Check,
  failed: X,
  pending: X,
}

const STATUS_CLASS: Record<CleanupEntryStatus, string> = {
  busy: 'border-[color:color-mix(in_oklch,var(--badge-red)_30%,transparent)] bg-[color:color-mix(in_oklch,var(--badge-red)_12%,transparent)] text-[var(--badge-red)]',
  clean: 'border-[color:color-mix(in_oklch,var(--success)_30%,transparent)] bg-[color:color-mix(in_oklch,var(--success)_12%,transparent)] text-[var(--success)]',
  failed: 'border-[color:color-mix(in_oklch,var(--badge-red)_30%,transparent)] bg-[color:color-mix(in_oklch,var(--badge-red)_12%,transparent)] text-[var(--badge-red)]',
  pending: 'border-[color:color-mix(in_oklch,var(--badge-red)_30%,transparent)] bg-[color:color-mix(in_oklch,var(--badge-red)_12%,transparent)] text-[var(--badge-red)]',
}

type ReportMap = Partial<Record<CleanupCategoryId, CleanupCategoryReport>>
type BusyAction = 'access' | 'all' | CleanupCategoryId | null
const CLEAN_ALL_EVENT = 'winsentials:cleanup-clean-all'
const REFRESH_ALL_EVENT = 'winsentials:cleanup-refresh-all'
const CLEANUP_SUMMARY_EVENT = 'winsentials:cleanup-summary'

function formatBytes(bytes: number, t: ReturnType<typeof useTranslation>['t'], locale: string): string {
  return formatBytesLocalized(bytes, { decimals: 1, locale, t })
}

function categoryTotalSize(report: CleanupCategoryReport | null): number {
  return report?.entries.reduce((sum, entry) => sum + entry.sizeBytes, 0) ?? 0
}

function isCategoryClean(report: CleanupCategoryReport | null): boolean {
  return !!report && report.entries.length > 0 && report.entries.every(entry => entry.status === 'clean')
}

function hasCleanableEntries(report: CleanupCategoryReport | null): boolean {
  return !!report && report.entries.some(entry => entry.status === 'pending' || entry.status === 'failed' || entry.status === 'busy')
}

function reportMapFromReports(reports: CleanupCategoryReport[]): ReportMap {
  return Object.fromEntries(reports.map(report => [report.id, report])) as ReportMap
}

function busyEntriesFromReports(reports: CleanupCategoryReport[]): CleanupEntry[] {
  return reports.flatMap(report => report.entries.filter(entry => entry.status === 'busy'))
}

function cleanupSummaryFromReports(reports: ReportMap) {
  return Object.values(reports).reduce(
    (summary, report) => {
      if (!report) return summary

      return {
        sizeBytes: summary.sizeBytes + categoryTotalSize(report),
        targetCount: summary.targetCount + report.entries.length,
      }
    },
    { sizeBytes: 0, targetCount: 0 },
  )
}

function CleanupEntryRow({ entry, showSize = true }: { entry: CleanupEntry, showSize?: boolean }) {
  const { t, i18n } = useTranslation()
  const Icon = STATUS_ICON[entry.status]

  return (
    <div className="flex items-center gap-3 rounded-md border border-border/60 bg-background/50 p-2.5">
      <span
        className={cn(
          'flex size-6 shrink-0 items-center justify-center rounded-md',
          entry.iconDataUrl ? 'bg-transparent' : ['border', STATUS_CLASS[entry.status]],
        )}
      >
        {entry.iconDataUrl
          ? <img alt="" className="size-full object-contain" src={entry.iconDataUrl} />
          : <Icon className="size-3.5" />}
      </span>
      <div className="flex min-w-0 flex-1 flex-col gap-0.5">
        <span className="truncate text-xs font-medium text-foreground">{entry.name}</span>
        <p className="truncate text-[11px] text-muted-foreground">{entry.path}</p>
        {entry.error && <p className="text-[11px] text-[var(--badge-red)]">{entry.error}</p>}
      </div>
      {showSize && (
        <span className="shrink-0 self-center text-xs tabular-nums text-muted-foreground">
          {formatBytes(entry.sizeBytes, t, i18n.language)}
        </span>
      )}
    </div>
  )
}

function CleanupEntryVirtualList({ entries, showSize = true }: { entries: CleanupEntry[], showSize?: boolean }) {
  const parentRef = useRef<HTMLDivElement>(null)
  const virtualizer = useVirtualizer({
    count: entries.length,
    estimateSize: () => 60,
    getItemKey: index => entries[index]?.id ?? `cleanup-entry-${index}`,
    getScrollElement: () => parentRef.current,
    overscan: 8,
  })

  return (
    <div ref={parentRef} className="max-h-96 overflow-y-auto pr-1 [overflow-anchor:none]" data-lenis-prevent>
      <div className="relative w-full" style={{ height: `${virtualizer.getTotalSize()}px` }}>
        {virtualizer.getVirtualItems().map((virtualRow) => {
          const entry = entries[virtualRow.index]
          if (!entry) return null

          return (
            <div
              className="absolute left-0 top-0 w-full pb-2"
              data-index={virtualRow.index}
              key={virtualRow.key}
              ref={virtualizer.measureElement}
              style={{ transform: `translateY(${virtualRow.start}px)` }}
            >
              <CleanupEntryRow entry={entry} showSize={showSize} />
            </div>
          )
        })}
      </div>
    </div>
  )
}

function CleanupCard({
  category,
  isBusy,
  isRefreshing,
  onClean,
  onRefresh,
  onToggle,
  open,
  report,
}: {
  category: CleanupCategoryDefinition
  isBusy: boolean
  isRefreshing: boolean
  onClean: (id: CleanupCategoryId) => void
  onRefresh: (id: CleanupCategoryId) => void
  onToggle: (id: CleanupCategoryId) => void
  open: boolean
  report: CleanupCategoryReport | null
}) {
  const { t, i18n } = useTranslation()
  const Icon = category.icon
  const totalSize = categoryTotalSize(report)
  const clean = isCategoryClean(report)
  const canClean = hasCleanableEntries(report) && !clean && !isBusy && !isRefreshing
  const showEntrySize = category.id !== 'unused_devices'

  return (
    <section className="flex h-fit flex-col overflow-hidden rounded-lg border border-border/70 bg-card">
      <div className="flex items-center gap-3 p-4">
        <button
          className="flex min-w-0 flex-1 cursor-pointer items-center gap-3 text-left"
          onClick={() => onToggle(category.id)}
          type="button"
        >
          <span className="ui-soft-surface flex size-9 shrink-0 items-center justify-center rounded-md">
            <Icon className="size-4" />
          </span>
          <div className="min-w-0 flex-1">
            <h2 className="truncate text-sm font-medium text-foreground">
              {t(`cleanup.categories.${category.id}.name`)}
            </h2>
            <div className="mt-1 flex flex-wrap items-center gap-2">
              <span className="rounded-md border border-border/60 bg-accent/45 px-1.5 py-0.5 text-[11px] text-muted-foreground">
                {report ? t('cleanup.itemsCount', { count: report.entries.length }) : t('cleanup.scanning')}
              </span>
              {showEntrySize && (
                <span className="rounded-md border border-border/60 bg-accent/45 px-1.5 py-0.5 text-[11px] text-muted-foreground">
                  {formatBytes(totalSize, t, i18n.language)}
                </span>
              )}
            </div>
          </div>
          <ChevronDown className={cn('size-4 shrink-0 text-muted-foreground transition-transform', open && 'rotate-180')} />
        </button>
        <Button disabled={!canClean} onClick={() => onClean(category.id)} size="sm" type="button">
          {isBusy ? <Loader2 className="size-4 animate-spin" /> : <Trash2 className="size-4" />}
          {t('cleanup.clean')}
        </Button>
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              aria-label={t('cleanup.refresh')}
              disabled={isBusy || isRefreshing}
              onClick={() => onRefresh(category.id)}
              size="icon-sm"
              type="button"
              variant="outline"
            >
              <RefreshCw className={cn('size-4', isRefreshing && 'animate-spin')} />
            </Button>
          </TooltipTrigger>
          <TooltipContent sideOffset={8}>{t('cleanup.refresh')}</TooltipContent>
        </Tooltip>
      </div>
      {open && (
        <div className="border-t border-border/70 p-3">
          {report
            ? report.entries.length === 0
              ? <p className="px-1 text-xs text-muted-foreground">{t('cleanup.noTargets')}</p>
              : (
                  <CleanupEntryVirtualList entries={report.entries} showSize={showEntrySize} />
                )
            : (
                <div className="flex flex-col gap-2">
                  {Array.from({ length: 4 }).map((_, index) => <Skeleton className="h-12 w-full" key={index} />)}
                </div>
              )}
        </div>
      )}
    </section>
  )
}

export function CleanupPage() {
  const { t } = useTranslation()
  const [reports, setReports] = useState<ReportMap>({})
  const [openCards, setOpenCards] = useState<Set<CleanupCategoryId>>(() => new Set())
  const [busyAction, setBusyAction] = useState<BusyAction>(null)
  const [refreshingCategories, setRefreshingCategories] = useState<Set<CleanupCategoryId>>(() => new Set())
  const [accessDialogEntries, setAccessDialogEntries] = useState<CleanupEntry[]>([])

  function scanCategories(categoryIds: CleanupCategoryId[]) {
    setRefreshingCategories((current) => {
      const next = new Set(current)
      categoryIds.forEach(categoryId => next.add(categoryId))
      return next
    })

    return Promise.all(categoryIds.map(categoryId => scanCleanupCategory(categoryId)))
      .then((categoryReports) => {
        setReports(current => ({ ...current, ...reportMapFromReports(categoryReports) }))
      })
      .catch((error) => {
        console.error(error)
        toast.error(t('cleanup.errors.scan'))
      })
      .finally(() => {
        setRefreshingCategories((current) => {
          const next = new Set(current)
          categoryIds.forEach(categoryId => next.delete(categoryId))
          return next
        })
      })
  }

  useMountEffect(() => {
    void scanCategories(CLEANUP_CATEGORIES.map(category => category.id))
  })

  function refreshCategory(categoryId: CleanupCategoryId) {
    void scanCategories([categoryId])
  }

  function refreshAllCategories() {
    void scanCategories(CLEANUP_CATEGORIES.map(category => category.id))
  }

  function toggleCard(categoryId: CleanupCategoryId) {
    setOpenCards((current) => {
      const next = new Set(current)
      if (next.has(categoryId)) {
        next.delete(categoryId)
      }
      else {
        next.add(categoryId)
      }
      return next
    })
  }

  function cleanCategory(categoryId: CleanupCategoryId) {
    setBusyAction(categoryId)
    cleanCleanupCategory(categoryId)
      .then((report) => {
        setReports(current => ({ ...current, [report.id]: report }))
        toast.success(t('cleanup.cleaned'))
        const busyEntries = busyEntriesFromReports([report])
        if (busyEntries.length > 0) {
          setAccessDialogEntries(busyEntries)
        }
      })
      .catch((error) => {
        console.error(error)
        toast.error(t('cleanup.errors.clean'))
      })
      .finally(() => {
        setBusyAction(null)
      })
  }

  function cleanAllCategories() {
    setBusyAction('all')
    Promise.all(CLEANUP_CATEGORIES.map(category => cleanCleanupCategory(category.id)))
      .then((categoryReports) => {
        setReports(reportMapFromReports(categoryReports))
        toast.success(t('cleanup.cleanedAll'))
        const busyEntries = busyEntriesFromReports(categoryReports)
        if (busyEntries.length > 0) {
          setAccessDialogEntries(busyEntries)
        }
      })
      .catch((error) => {
        console.error(error)
        toast.error(t('cleanup.errors.clean'))
      })
      .finally(() => {
        setBusyAction(null)
      })
  }

  function grantOneTimeAccess() {
    setBusyAction('access')
    prepareCleanupAccess()
      .then((report) => {
        const failedCount = report.entries.filter(entry => !entry.success).length
        if (failedCount > 0) {
          toast.error(t('cleanup.accessPreparedWithErrors', { count: failedCount }))
        }
        else {
          toast.success(t('cleanup.accessPrepared'))
        }
        return Promise.all(CLEANUP_CATEGORIES.map(category => scanCleanupCategory(category.id)))
      })
      .then((categoryReports) => {
        setReports(reportMapFromReports(categoryReports))
        setAccessDialogEntries([])
      })
      .catch((error) => {
        console.error(error)
        toast.error(t('cleanup.errors.access'))
      })
      .finally(() => {
        setBusyAction(null)
      })
  }

  const cards = useMemo(() => CLEANUP_CATEGORIES, [])
  const isBusy = busyAction !== null

  useEffect(() => {
    window.dispatchEvent(new CustomEvent(CLEANUP_SUMMARY_EVENT, {
      detail: cleanupSummaryFromReports(reports),
    }))
  }, [reports])

  useMountEffect(() => {
    window.addEventListener(CLEAN_ALL_EVENT, cleanAllCategories)
    window.addEventListener(REFRESH_ALL_EVENT, refreshAllCategories)
    return () => {
      window.removeEventListener(CLEAN_ALL_EVENT, cleanAllCategories)
      window.removeEventListener(REFRESH_ALL_EVENT, refreshAllCategories)
    }
  })

  return (
    <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
      <div className="tweak-card-grid">
        {cards.map(category => (
          <CleanupCard
            category={category}
            isBusy={busyAction === category.id || busyAction === 'all'}
            isRefreshing={refreshingCategories.has(category.id)}
            key={category.id}
            onClean={cleanCategory}
            onRefresh={refreshCategory}
            onToggle={toggleCard}
            open={openCards.has(category.id)}
            report={reports[category.id] ?? null}
          />
        ))}
      </div>
      <Dialog open={accessDialogEntries.length > 0} onOpenChange={open => !open && setAccessDialogEntries([])}>
        <DialogContent className="max-w-lg">
          <DialogHeader>
            <DialogTitle>{t('cleanup.accessDialog.title')}</DialogTitle>
            <DialogDescription>
              {t('cleanup.accessDialog.description', { count: accessDialogEntries.length })}
            </DialogDescription>
          </DialogHeader>
          <div className="flex max-h-56 flex-col gap-2 overflow-y-auto px-5 py-1" data-lenis-prevent>
            {accessDialogEntries.map(entry => (
              <div className="rounded-md border border-border/60 bg-background/50 p-2" key={entry.id}>
                <p className="truncate text-xs font-medium text-foreground">{entry.name}</p>
                <p className="truncate text-[11px] text-muted-foreground">{entry.path}</p>
              </div>
            ))}
          </div>
          <DialogFooter>
            <DialogClose asChild>
              <Button type="button" variant="outline">{t('common.cancel')}</Button>
            </DialogClose>
            <Button disabled={isBusy} onClick={grantOneTimeAccess} type="button">
              {busyAction === 'access' ? <Loader2 className="size-4 animate-spin" /> : <KeyRound className="size-4" />}
              {t('cleanup.prepareAccess')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </section>
  )
}
