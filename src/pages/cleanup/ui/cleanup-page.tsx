import type { LucideIcon } from 'lucide-react'
import type { CleanupCategoryId, CleanupCategoryReport, CleanupEntry, CleanupEntryStatus } from '@/entities/cleanup/model/types'
import { useVirtualizer } from '@tanstack/react-virtual'
import { AppWindow, Check, ChevronDown, Code2, Gamepad2, Globe, Loader2, MonitorCog, PackageOpen, RefreshCw, Trash2, Unplug, Video, X } from 'lucide-react'
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
  { id: 'windows', icon: MonitorCog },
  { id: 'browsers', icon: Globe },
  { id: 'applications', icon: AppWindow },
  { id: 'development', icon: Code2 },
  { id: 'gaming', icon: Gamepad2 },
  { id: 'media', icon: Video },
  { id: 'appx', icon: PackageOpen },
  { id: 'unused_devices', icon: Unplug },
]

const STATUS_ICON: Record<CleanupEntryStatus, LucideIcon> = {
  busy: Check,
  clean: Check,
  failed: X,
  pending: X,
  removed: Check,
}

const STATUS_CLASS: Record<CleanupEntryStatus, string> = {
  busy: 'border-[color:color-mix(in_oklch,var(--warning)_30%,transparent)] bg-[color:color-mix(in_oklch,var(--warning)_12%,transparent)] text-[var(--warning)]',
  clean: 'border-[color:color-mix(in_oklch,var(--success)_30%,transparent)] bg-[color:color-mix(in_oklch,var(--success)_12%,transparent)] text-[var(--success)]',
  failed: 'border-[color:color-mix(in_oklch,var(--badge-red)_30%,transparent)] bg-[color:color-mix(in_oklch,var(--badge-red)_12%,transparent)] text-[var(--badge-red)]',
  pending: 'border-[color:color-mix(in_oklch,var(--badge-red)_30%,transparent)] bg-[color:color-mix(in_oklch,var(--badge-red)_12%,transparent)] text-[var(--badge-red)]',
  removed: 'border-[color:color-mix(in_oklch,var(--success)_30%,transparent)] bg-[color:color-mix(in_oklch,var(--success)_12%,transparent)] text-[var(--success)]',
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

function isCategoryClean(entries: CleanupEntry[]): boolean {
  return entries.length > 0 && entries.every(entry => entry.status === 'clean' || entry.status === 'removed')
}

function reportMapFromReports(reports: CleanupCategoryReport[]): ReportMap {
  return Object.fromEntries(reports.map(report => [report.id, report])) as ReportMap
}

function errorMessageFromReason(reason: unknown): string {
  if (reason instanceof Error && reason.message) return reason.message
  if (typeof reason === 'string' && reason) return reason
  return 'Failed to scan cleanup category'
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
    'Failed to scan cleanup category': 'cleanup.messages.scanCategoryFailed',
    'Scheduled for deletion on reboot': 'cleanup.messages.scheduledOnReboot',
    'Some files are in use and were skipped': 'cleanup.messages.skippedBusyFiles',
  }

  const normalizedError = error.replace(/\.\s*$/, '').trim()
  const key = knownMessages[normalizedError]
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

function cleanupSummaryFromReports(
  reports: ReportMap,
  checkedCategories: Set<CleanupCategoryId>,
  uncheckedEntries: Record<CleanupCategoryId, Set<string>>,
) {
  return Object.values(reports).reduce(
    (summary, report) => {
      if (!report || !checkedCategories.has(report.id)) return summary

      const uncheckedSet = uncheckedEntries[report.id] || new Set<string>()
      const activeEntries = report.entries.filter(entry => !uncheckedSet.has(entry.id))
      const hasCleanableActive = activeEntries.some(
        entry => entry.status === 'pending' || entry.status === 'busy' || (entry.status === 'failed' && !entry.id.endsWith('-scan-error')),
      )
      const clean = activeEntries.length > 0 && activeEntries.every(entry => entry.status === 'clean' || entry.status === 'removed')
      const activeSize = activeEntries.reduce((sum, entry) => sum + entry.sizeBytes, 0)

      return {
        cleanableCount: summary.cleanableCount + (hasCleanableActive && !clean ? 1 : 0),
        sizeBytes: summary.sizeBytes + activeSize,
        targetCount: summary.targetCount + activeEntries.length,
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

function Checkbox({
  checked,
  onCheckedChange,
  disabled,
  className,
}: {
  checked: boolean
  onCheckedChange: (checked: boolean) => void
  disabled?: boolean
  className?: string
}) {
  return (
    <button
      type="button"
      role="checkbox"
      aria-checked={checked}
      disabled={disabled}
      onClick={(e) => {
        e.stopPropagation()
        onCheckedChange(!checked)
      }}
      className={cn(
        'flex size-4 shrink-0 items-center justify-center rounded border transition-all duration-200 cursor-pointer select-none outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50',
        checked
          ? 'border-primary bg-primary text-primary-foreground'
          : 'border-border bg-background/50 hover:border-primary/50',
        className,
      )}
    >
      {checked && <Check className="size-3 stroke-[3]" />}
    </button>
  )
}

function CleanupEntryRow({
  entry,
  showSize = true,
  checked,
  onToggle,
  disabled,
}: {
  entry: CleanupEntry
  showSize?: boolean
  checked: boolean
  onToggle: () => void
  disabled?: boolean
}) {
  const { t, i18n } = useTranslation()
  const Icon = STATUS_ICON[entry.status]
  const isErrorEntry = entry.id.endsWith('-scan-error')

  return (
    <div
      className={cn(
        'flex items-center gap-3 rounded-md border p-2.5 transition-all duration-200',
        isErrorEntry
          ? 'border-border/60 bg-background/50'
          : checked
            ? 'border-border/60 bg-background/50'
            : 'border-border/40 bg-background/20 opacity-60 hover:opacity-85',
      )}
    >
      {!isErrorEntry && (
        <Checkbox
          checked={checked}
          onCheckedChange={onToggle}
          disabled={disabled}
        />
      )}
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
      {showSize
        ? (
            <span className="shrink-0 self-center text-xs tabular-nums text-muted-foreground">
              {formatBytes(entry.sizeBytes, t, i18n.language)}
            </span>
          )
        : null}
    </div>
  )
}

function CleanupEntryVirtualList({
  entries,
  showSize = true,
  uncheckedEntryIds,
  onToggleEntry,
  disabled,
}: {
  entries: CleanupEntry[]
  showSize?: boolean
  uncheckedEntryIds: Set<string>
  onToggleEntry: (id: string) => void
  disabled?: boolean
}) {
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

          const isChecked = !uncheckedEntryIds.has(entry.id)

          return (
            <div
              className="absolute left-0 top-0 w-full pb-2"
              data-index={virtualRow.index}
              key={virtualRow.key}
              ref={virtualizer.measureElement}
              style={{ transform: `translateY(${virtualRow.start}px)` }}
            >
              <CleanupEntryRow
                entry={entry}
                showSize={showSize}
                checked={isChecked}
                onToggle={() => onToggleEntry(entry.id)}
                disabled={disabled}
              />
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
  isChecked,
  onCategoryToggle,
  uncheckedEntryIds,
  onToggleEntry,
}: {
  category: CleanupCategoryDefinition
  isBusy: boolean
  isRefreshing: boolean
  onClean: (id: CleanupCategoryId) => void
  onRefresh: (id: CleanupCategoryId) => void
  onToggle: (id: CleanupCategoryId) => void
  open: boolean
  report: CleanupCategoryReport | null
  isChecked: boolean
  onCategoryToggle: (id: CleanupCategoryId) => void
  uncheckedEntryIds: Set<string>
  onToggleEntry: (categoryId: CleanupCategoryId, entryId: string) => void
}) {
  const { t, i18n } = useTranslation()
  const Icon = category.icon
  const activeEntries = report?.entries.filter(entry => !uncheckedEntryIds.has(entry.id)) ?? []
  const totalSize = activeEntries.reduce((sum, entry) => sum + entry.sizeBytes, 0)
  const clean = report ? isCategoryClean(activeEntries) : false
  const canClean = activeEntries.length > 0 && !clean && !isBusy && !isRefreshing && isChecked
  const showEntrySize = category.id !== 'unused_devices' && category.id !== 'appx'

  return (
    <section
      className={cn(
        'flex h-fit flex-col overflow-hidden rounded-lg border transition-all duration-200',
        isChecked
          ? 'border-border/70 bg-card'
          : 'border-border/40 bg-card/60 opacity-70 hover:opacity-90',
      )}
    >
      <div className="flex items-center gap-3 p-4">
        <Checkbox
          checked={isChecked}
          onCheckedChange={() => onCategoryToggle(category.id)}
          disabled={isBusy || isRefreshing}
        />
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
                {report
                  ? `${activeEntries.length} / ${report.entries.length}`
                  : t('cleanup.scanning')}
              </span>
              {showEntrySize && report
                ? (
                    <span className="rounded-md border border-border/60 bg-accent/45 px-1.5 py-0.5 text-[11px] text-muted-foreground">
                      {formatBytes(totalSize, t, i18n.language)}
                    </span>
                  )
                : null}
            </div>
          </div>
          <ChevronDown className={cn('size-4 shrink-0 text-muted-foreground transition-transform', open && 'rotate-180')} />
        </button>
        <Button disabled={!canClean} onClick={() => onClean(category.id)} type="button">
          {isBusy ? <Loader2 className="size-4 animate-spin" /> : <Trash2 className="size-4" />}
          {t('cleanup.clean')}
        </Button>
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              aria-label={t('cleanup.refresh')}
              disabled={isBusy || isRefreshing}
              onClick={() => onRefresh(category.id)}
              size="icon"
              type="button"
              variant="ghost"
              className="ui-soft-surface transition-colors hover:bg-accent/50!"
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
                  <CleanupEntryVirtualList
                    entries={report.entries}
                    showSize={showEntrySize}
                    uncheckedEntryIds={uncheckedEntryIds}
                    onToggleEntry={entryId => onToggleEntry(category.id, entryId)}
                    disabled={isBusy || isRefreshing}
                  />
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
  const [openCards, setOpenCards] = useState<Set<CleanupCategoryId>>(() => new Set())
  const [busyAction, setBusyAction] = useState<BusyAction>(null)
  const busyActionRef = useRef<BusyAction>(null)

  const [checkedCategories, setCheckedCategories] = useState<Set<CleanupCategoryId>>(
    () => new Set(CLEANUP_CATEGORIES.map(c => c.id)),
  )
  const [uncheckedEntries, setUncheckedEntries] = useState<Record<CleanupCategoryId, Set<string>>>(
    () => Object.fromEntries(CLEANUP_CATEGORIES.map(c => [c.id, new Set<string>()])) as Record<CleanupCategoryId, Set<string>>,
  )

  const checkedCategoriesRef = useRef(checkedCategories)
  const uncheckedEntriesRef = useRef(uncheckedEntries)

  useEffect(() => {
    checkedCategoriesRef.current = checkedCategories
  }, [checkedCategories])

  useEffect(() => {
    uncheckedEntriesRef.current = uncheckedEntries
  }, [uncheckedEntries])

  function setBusyActionState(action: BusyAction) {
    busyActionRef.current = action
    setBusyAction(action)
    setCleanupBusy(action !== null)
  }

  function updateReports(updater: (current: ReportMap) => ReportMap) {
    setReports(updater)
  }

  useEffect(() => {
    dispatchCleanupSummary(cleanupSummaryFromReports(reports, checkedCategories, uncheckedEntries))
  }, [reports, checkedCategories, uncheckedEntries])

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
        removeRefreshingCategories(categoryIds)
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

  function toggleCategoryChecked(categoryId: CleanupCategoryId) {
    setCheckedCategories((current) => {
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

  function toggleEntryChecked(categoryId: CleanupCategoryId, entryId: string) {
    setUncheckedEntries((current) => {
      const next = { ...current }
      const currentSet = next[categoryId] ? new Set(next[categoryId]) : new Set<string>()
      if (currentSet.has(entryId)) {
        currentSet.delete(entryId)
      }
      else {
        currentSet.add(entryId)
      }
      next[categoryId] = currentSet
      return next
    })
  }

  function cleanCategory(categoryId: CleanupCategoryId) {
    if (busyActionRef.current !== null || cleanupUiState.refreshingCategories.has(categoryId)) return

    setBusyActionState(categoryId)
    const excludeEntryIds = Array.from(uncheckedEntries[categoryId] || [])
    cleanCleanupCategory(categoryId, excludeEntryIds)
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

    const activeCategories = CLEANUP_CATEGORIES.filter(category => checkedCategoriesRef.current.has(category.id))
    if (activeCategories.length === 0) {
      toast.error(t('cleanup.errors.nothingSelected') || 'No categories selected for cleaning')
      return
    }

    setBusyActionState('all')
    Promise.allSettled(
      activeCategories.map(category =>
        cleanCleanupCategory(
          category.id,
          Array.from(uncheckedEntriesRef.current[category.id] || []),
        ),
      ),
    )
      .then((results) => {
        const categoryReports: CleanupCategoryReport[] = []
        const failures: PromiseRejectedResult[] = []

        for (const result of results) {
          if (result.status === 'fulfilled') {
            categoryReports.push(result.value)
          }
          else {
            failures.push(result)
          }
        }

        if (categoryReports.length > 0) {
          updateReports(current => ({ ...current, ...reportMapFromReports(categoryReports) }))
        }

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
            isChecked={checkedCategories.has(category.id)}
            onCategoryToggle={toggleCategoryChecked}
            uncheckedEntryIds={uncheckedEntries[category.id] || new Set<string>()}
            onToggleEntry={toggleEntryChecked}
          />
        ))}
      </div>
    </section>
  )
}

export default CleanupPage
