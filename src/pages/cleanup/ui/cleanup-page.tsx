import type { LucideIcon } from 'lucide-react'
import type { CleanupCategoryId, CleanupCategoryReport, CleanupEntry, CleanupEntryStatus } from '@/entities/cleanup/model/types'
import { useVirtualizer } from '@tanstack/react-virtual'
import { Bug, Check, ChevronDown, Cpu, FileText, Gamepad2, Globe, Image, Loader2, PackageOpen, RefreshCw, Sparkles, Trash2, Unplug, X } from 'lucide-react'
import { useEffect, useMemo, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { cleanCleanupCategory, scanCleanupCategory } from '@/entities/cleanup/api'
import { addRefreshingCategories, hasRefreshingCategories, removeRefreshingCategories, setCleanupBusy, useCleanupUiState } from '@/entities/cleanup/model/ui-state'
import { formatBytesLocalized } from '@/shared/lib/format-size'
import { useMountEffect } from '@/shared/lib/hooks/use-mount-effect'
import { toast } from '@/shared/lib/toast'
import { cn } from '@/shared/lib/utils'
import { Button } from '@/shared/ui/button'
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
  busy: Check,
  clean: Check,
  failed: X,
  pending: X,
}

const STATUS_CLASS: Record<CleanupEntryStatus, string> = {
  busy: 'border-[color:color-mix(in_oklch,var(--warning)_30%,transparent)] bg-[color:color-mix(in_oklch,var(--warning)_12%,transparent)] text-[var(--warning)]',
  clean: 'border-[color:color-mix(in_oklch,var(--success)_30%,transparent)] bg-[color:color-mix(in_oklch,var(--success)_12%,transparent)] text-[var(--success)]',
  failed: 'border-[color:color-mix(in_oklch,var(--badge-red)_30%,transparent)] bg-[color:color-mix(in_oklch,var(--badge-red)_12%,transparent)] text-[var(--badge-red)]',
  pending: 'border-[color:color-mix(in_oklch,var(--badge-red)_30%,transparent)] bg-[color:color-mix(in_oklch,var(--badge-red)_12%,transparent)] text-[var(--badge-red)]',
}

type ReportMap = Partial<Record<CleanupCategoryId, CleanupCategoryReport>>
type BusyAction = 'all' | CleanupCategoryId | null
const CLEAN_ALL_EVENT = 'winsentials:cleanup-clean-all'
const REFRESH_ALL_EVENT = 'winsentials:cleanup-refresh-all'
const CLEANUP_SUMMARY_EVENT = 'winsentials:cleanup-summary'
const EMPTY_CLEANUP_SUMMARY = { cleanableCount: 0, sizeBytes: 0, targetCount: 0 }

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

function errorMessageFromReason(reason: unknown): string {
  if (reason instanceof Error && reason.message) return reason.message
  if (typeof reason === 'string' && reason) return reason
  return 'Failed to scan cleanup category.'
}

function cleanupEntryMessage(error: string, t: ReturnType<typeof useTranslation>['t']): string {
  const skippedBusyPrefix = 'Some files are in use and were skipped.'
  if (error.startsWith(`${skippedBusyPrefix} (`)) {
    return t('cleanup.messages.skippedBusyFiles')
  }

  const scheduledRebootPrefix = 'Scheduled for deletion on reboot.'
  if (error.startsWith(`${scheduledRebootPrefix} (`)) {
    return `${t('cleanup.messages.scheduledOnReboot')} ${error.slice(scheduledRebootPrefix.length).trim()}`
  }

  const knownMessages: Record<string, string> = {
    'Failed to scan cleanup category.': 'cleanup.messages.scanCategoryFailed',
    'Scheduled for deletion on reboot': 'cleanup.messages.scheduledOnReboot',
    'Some files are in use and were skipped.': 'cleanup.messages.skippedBusyFiles',
  }

  const key = knownMessages[error]
  return key ? t(key) : error
}

function failedScanReport(categoryId: CleanupCategoryId, reason: unknown, t: ReturnType<typeof useTranslation>['t']): CleanupCategoryReport {
  return {
    id: categoryId,
    entries: [
      {
        error: errorMessageFromReason(reason),
        iconDataUrl: null,
        id: `${categoryId}-scan-error`,
        name: t(`cleanup.categories.${categoryId}.name`),
        path: '',
        sizeBytes: 0,
        status: 'failed',
      },
    ],
  }
}

function cleanupSummaryFromReports(reports: ReportMap) {
  return Object.values(reports).reduce(
    (summary, report) => {
      if (!report) return summary

      return {
        cleanableCount: summary.cleanableCount + (hasCleanableEntries(report) && !isCategoryClean(report) ? 1 : 0),
        sizeBytes: summary.sizeBytes + categoryTotalSize(report),
        targetCount: summary.targetCount + report.entries.length,
      }
    },
    EMPTY_CLEANUP_SUMMARY,
  )
}

function dispatchCleanupSummary(summary = EMPTY_CLEANUP_SUMMARY) {
  window.dispatchEvent(new CustomEvent(CLEANUP_SUMMARY_EVENT, {
    detail: summary,
  }))
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
        {entry.error && (
          <p className={cn(
            'text-[11px]',
            entry.status === 'busy' || entry.status === 'clean' ? 'text-[var(--warning)]' : 'text-[var(--badge-red)]',
          )}
          >
            {cleanupEntryMessage(entry.error, t)}
          </p>
        )}
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

function CleanupPage() {
  const { t } = useTranslation()
  const cleanupUiState = useCleanupUiState()
  const [reports, setReports] = useState<ReportMap>({})
  const [refreshingCategoriesToRelease, setRefreshingCategoriesToRelease] = useState<Set<CleanupCategoryId>>(() => new Set())
  const [openCards, setOpenCards] = useState<Set<CleanupCategoryId>>(() => new Set())
  const [busyAction, setBusyAction] = useState<BusyAction>(null)
  const busyActionRef = useRef<BusyAction>(null)

  function setBusyActionState(action: BusyAction) {
    busyActionRef.current = action
    setBusyAction(action)
    setCleanupBusy(action !== null)
  }

  function updateReports(updater: (current: ReportMap) => ReportMap) {
    setReports(updater)
  }

  useEffect(() => {
    dispatchCleanupSummary(cleanupSummaryFromReports(reports))

    if (refreshingCategoriesToRelease.size === 0) return

    removeRefreshingCategories([...refreshingCategoriesToRelease])
    setRefreshingCategoriesToRelease(new Set())
  }, [refreshingCategoriesToRelease, reports])

  function scanCategories(categoryIds: CleanupCategoryId[]) {
    addRefreshingCategories(categoryIds)

    return Promise.allSettled(categoryIds.map(categoryId => scanCleanupCategory(categoryId)))
      .then((results) => {
        const categoryReports = results.map((result, index) => (
          result.status === 'fulfilled'
            ? result.value
            : failedScanReport(categoryIds[index], result.reason, t)
        ))

        updateReports(current => ({ ...current, ...reportMapFromReports(categoryReports) }))

        const failures = results.filter((result): result is PromiseRejectedResult => result.status === 'rejected')
        if (failures.length > 0) {
          failures.forEach(result => console.error(result.reason))
          toast.error(t('cleanup.errors.scan'))
        }
      })
      .finally(() => {
        setRefreshingCategoriesToRelease((current) => {
          const next = new Set(current)
          categoryIds.forEach(categoryId => next.add(categoryId))
          return next
        })
      })
  }

  useMountEffect(() => {
    void scanCategories(CLEANUP_CATEGORIES.map(category => category.id))
    return () => dispatchCleanupSummary()
  })

  function refreshCategory(categoryId: CleanupCategoryId) {
    void scanCategories([categoryId])
  }

  function refreshAllCategories() {
    if (busyActionRef.current !== null || hasRefreshingCategories()) return

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
    if (busyActionRef.current !== null || cleanupUiState.refreshingCategories.has(categoryId)) return

    setBusyActionState(categoryId)
    cleanCleanupCategory(categoryId)
      .then((report) => {
        updateReports(current => ({ ...current, [report.id]: report }))
        toast.success(t('cleanup.cleaned'))
      })
      .catch((error) => {
        console.error(error)
        toast.error(t('cleanup.errors.clean'))
      })
      .finally(() => {
        setBusyActionState(null)
      })
  }

  function cleanAllCategories() {
    if (busyActionRef.current !== null || hasRefreshingCategories()) return

    setBusyActionState('all')
    Promise.allSettled(CLEANUP_CATEGORIES.map(category => cleanCleanupCategory(category.id)))
      .then((results) => {
        const categoryReports = results
          .filter((result): result is PromiseFulfilledResult<CleanupCategoryReport> => result.status === 'fulfilled')
          .map(result => result.value)

        if (categoryReports.length > 0) {
          updateReports(current => ({ ...current, ...reportMapFromReports(categoryReports) }))
        }

        const failures = results.filter((result): result is PromiseRejectedResult => result.status === 'rejected')
        if (failures.length > 0) {
          failures.forEach(result => console.error(result.reason))
          toast.error(t('cleanup.errors.clean'))
          return
        }

        toast.success(t('cleanup.cleanedAll'))
      })
      .finally(() => {
        setBusyActionState(null)
      })
  }

  const cards = useMemo(() => CLEANUP_CATEGORIES, [])

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
            isRefreshing={cleanupUiState.refreshingCategories.has(category.id)}
            key={category.id}
            onClean={cleanCategory}
            onRefresh={refreshCategory}
            onToggle={toggleCard}
            open={openCards.has(category.id)}
            report={reports[category.id] ?? null}
          />
        ))}
      </div>
    </section>
  )
}

export default CleanupPage
