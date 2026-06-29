import type { LucideIcon } from 'lucide-react'
import type { CleanupCategoryId, CleanupCategoryReport, CleanupEntry, CleanupEntryStatus, ReportMap } from '@/entities/cleanup/model/types'
import { useVirtualizer } from '@tanstack/react-virtual'
import { AlertTriangle, AppWindow, Check, CheckSquare, ChevronDown, Code2, Gamepad2, Globe, Loader2, MonitorCog, PackageOpen, RefreshCw, RotateCcw, Square, Trash2, Unplug, Video, X } from 'lucide-react'
import { useEffect, useMemo, useReducer, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { cleanAllCleanupCategories, cleanCleanupCategory } from '@/entities/cleanup/api'
import { cleanupScanCache, useCleanupReports } from '@/entities/cleanup/model/scan-cache'
import { addRefreshingCategories, hasRefreshingCategories, removeRefreshingCategories, setCleanupBusy, useCleanupUiState } from '@/entities/cleanup/model/ui-state'
import { formatBytesLocalized } from '@/shared/lib/format-size'
import { useMountEffect } from '@/shared/lib/hooks/use-mount-effect'
import { toast } from '@/shared/lib/toast'
import { cn } from '@/shared/lib/utils'
import { Button } from '@/shared/ui/button'
import { Skeleton } from '@/shared/ui/skeleton'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/shared/ui/tooltip'
import WinappDbStatusBar from './winapp-db-status-bar'

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

type BusyAction = 'all' | CleanupCategoryId | null
const CLEAN_ALL_EVENT = 'winsentials:cleanup-clean-all'
const REFRESH_ALL_EVENT = 'winsentials:cleanup-refresh-all'
const TOGGLE_ALL_CATEGORIES_EVENT = 'winsentials:cleanup-toggle-all-categories'
const CLEANUP_SUMMARY_EVENT = 'winsentials:cleanup-summary'
const EMPTY_CLEANUP_SUMMARY = { cleanableCount: 0, sizeBytes: 0, targetCount: 0 }

interface CleanupSummary {
  cleanableCount: number
  sizeBytes: number
  targetCount: number
  hasAnyChecked?: boolean
}

function formatBytes(bytes: number, t: ReturnType<typeof useTranslation>['t'], locale: string): string {
  return formatBytesLocalized(bytes, { decimals: 1, locale, t })
}

function isCategoryClean(entries: CleanupEntry[]): boolean {
  return entries.length > 0 && entries.every(entry => entry.status === 'clean' || entry.status === 'removed')
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

function formatEntryPath(path: string, t: ReturnType<typeof useTranslation>['t']): string {
  if (path === 'No matching cleanup targets found') {
    return t('cleanup.noMatchedTargets')
  }
  const match = path.match(/^(\d+) matched cleanup targets?$/)
  if (match) {
    const count = Number.parseInt(match[1], 10)
    return t('cleanup.matchedTargetsCount', { count })
  }
  return path
}

function cleanupSummaryFromReports(
  reports: ReportMap,
  checkedCategories: Set<CleanupCategoryId>,
  uncheckedEntries: Record<CleanupCategoryId, Set<string>>,
) {
  const summary = Object.values(reports).reduce(
    (acc, report) => {
      if (!report || !checkedCategories.has(report.id)) return acc

      const uncheckedSet = uncheckedEntries[report.id] || new Set<string>()
      const activeEntries = report.entries.filter(entry => !uncheckedSet.has(entry.id))
      const hasCleanableActive = activeEntries.some(
        entry => entry.status === 'pending' || entry.status === 'busy' || (entry.status === 'failed' && !entry.id.endsWith('-scan-error')),
      )
      const clean = activeEntries.length > 0 && activeEntries.every(entry => entry.status === 'clean' || entry.status === 'removed')
      const activeSize = activeEntries.reduce((sum, entry) => sum + entry.sizeBytes, 0)

      return {
        ...acc,
        cleanableCount: acc.cleanableCount + (hasCleanableActive && !clean ? 1 : 0),
        sizeBytes: acc.sizeBytes + activeSize,
        targetCount: acc.targetCount + activeEntries.length,
      }
    },
    { ...EMPTY_CLEANUP_SUMMARY, hasAnyChecked: checkedCategories.size > 0 } as CleanupSummary,
  )
  return summary
}

function dispatchCleanupSummary(summary: CleanupSummary = { ...EMPTY_CLEANUP_SUMMARY, hasAnyChecked: true }) {
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
  const displayName = entry.name || (isErrorEntry ? t(`cleanup.categories.${entry.id.replace(/-scan-error$/, '')}.name`) : entry.name)

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
        <div className="flex items-center gap-1.5">
          <span className="truncate text-xs font-medium text-foreground">{displayName}</span>
          {entry.warning && (
            <Tooltip>
              <TooltipTrigger asChild>
                <AlertTriangle className="size-3.5 shrink-0 text-[var(--warning)]" />
              </TooltipTrigger>
              <TooltipContent side="top" className="max-w-xs">
                {entry.warning}
              </TooltipContent>
            </Tooltip>
          )}
        </div>
        <p className="truncate text-[11px] text-muted-foreground">{formatEntryPath(entry.path, t)}</p>
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
  onToggleAllEntries,
  onResetToDefaults,
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
  onToggleAllEntries: (id: CleanupCategoryId) => void
  onResetToDefaults: (id: CleanupCategoryId) => void
}) {
  const { t, i18n } = useTranslation()
  const Icon = category.icon
  const activeEntries = report?.entries.filter(entry => !uncheckedEntryIds.has(entry.id)) ?? []
  const totalSize = activeEntries.reduce((sum, entry) => sum + entry.sizeBytes, 0)
  const clean = report ? isCategoryClean(activeEntries) : false
  const canClean = activeEntries.length > 0 && !clean && !isBusy && !isRefreshing && isChecked
  const showEntrySize = category.id !== 'unused_devices' && category.id !== 'appx'
  const validEntries = report?.entries.filter(entry => !entry.id.endsWith('-scan-error')) ?? []
  const checkedEntriesCount = validEntries.filter(entry => !uncheckedEntryIds.has(entry.id)).length
  const hasAnyCheckedEntries = checkedEntriesCount > 0

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
              aria-label={hasAnyCheckedEntries ? t('cleanup.uncheckAll') : t('cleanup.checkAll')}
              disabled={isBusy || isRefreshing || !report || report.entries.length === 0}
              onClick={() => onToggleAllEntries(category.id)}
              size="icon"
              type="button"
              variant="ghost"
              className="ui-soft-surface transition-colors hover:bg-accent/50!"
            >
              {hasAnyCheckedEntries ? <CheckSquare className="size-4" /> : <Square className="size-4" />}
            </Button>
          </TooltipTrigger>
          <TooltipContent sideOffset={8}>
            {hasAnyCheckedEntries ? t('cleanup.uncheckAll') : t('cleanup.checkAll')}
          </TooltipContent>
        </Tooltip>
        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              aria-label={t('cleanup.selectDefaults')}
              disabled={isBusy || isRefreshing || !report || report.entries.length === 0}
              onClick={() => onResetToDefaults(category.id)}
              size="icon"
              type="button"
              variant="ghost"
              className="ui-soft-surface transition-colors hover:bg-accent/50!"
            >
              <RotateCcw className="size-4" />
            </Button>
          </TooltipTrigger>
          <TooltipContent sideOffset={8}>{t('cleanup.selectDefaults')}</TooltipContent>
        </Tooltip>
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

interface CleanupSelectionState {
  openCards: Set<CleanupCategoryId>
  checkedCategories: Set<CleanupCategoryId>
  uncheckedEntries: Record<CleanupCategoryId, Set<string>>
  defaultsAppliedCategories: Set<CleanupCategoryId>
}

type CleanupSelectionAction
  = | { type: 'toggleCard', categoryId: CleanupCategoryId }
    | { type: 'toggleCategory', categoryId: CleanupCategoryId }
    | { type: 'toggleEntry', categoryId: CleanupCategoryId, entryId: string }
    | { type: 'toggleAllCategories' }
    | { type: 'toggleAllEntries', categoryId: CleanupCategoryId, validEntries: CleanupEntry[] }
    | { type: 'initDefaults', reports: ReportMap }
    | { type: 'resetToDefaults', categoryId: CleanupCategoryId, defaultFalseEntryIds: string[] }

const INITIAL_CLEANUP_SELECTION: CleanupSelectionState = {
  openCards: new Set(),
  checkedCategories: new Set(CLEANUP_CATEGORIES.map(c => c.id)),
  uncheckedEntries: Object.fromEntries(
    CLEANUP_CATEGORIES.map(c => [c.id, new Set<string>()]),
  ) as Record<CleanupCategoryId, Set<string>>,
  defaultsAppliedCategories: new Set(),
}

function defaultFalseEntryIds(report: CleanupCategoryReport | null | undefined): string[] {
  if (!report) return []
  const ids: string[] = []
  for (const entry of report.entries) {
    if (!entry.defaultChecked && !entry.id.endsWith('-scan-error')) {
      ids.push(entry.id)
    }
  }
  return ids
}

function cleanupSelectionReducer(
  state: CleanupSelectionState,
  action: CleanupSelectionAction,
): CleanupSelectionState {
  switch (action.type) {
    case 'toggleCard': {
      const openCards = new Set(state.openCards)
      if (openCards.has(action.categoryId)) {
        openCards.delete(action.categoryId)
      }
      else {
        openCards.add(action.categoryId)
      }
      return { ...state, openCards }
    }
    case 'toggleCategory': {
      const checkedCategories = new Set(state.checkedCategories)
      if (checkedCategories.has(action.categoryId)) {
        checkedCategories.delete(action.categoryId)
      }
      else {
        checkedCategories.add(action.categoryId)
      }
      return { ...state, checkedCategories }
    }
    case 'toggleEntry': {
      const uncheckedEntries = { ...state.uncheckedEntries }
      const currentSet = uncheckedEntries[action.categoryId]
        ? new Set(uncheckedEntries[action.categoryId])
        : new Set<string>()
      if (currentSet.has(action.entryId)) {
        currentSet.delete(action.entryId)
      }
      else {
        currentSet.add(action.entryId)
      }
      uncheckedEntries[action.categoryId] = currentSet
      return { ...state, uncheckedEntries }
    }
    case 'toggleAllCategories': {
      const checkedCategories = state.checkedCategories.size > 0
        ? new Set<CleanupCategoryId>()
        : new Set(CLEANUP_CATEGORIES.map(c => c.id))
      return { ...state, checkedCategories }
    }
    case 'toggleAllEntries': {
      const uncheckedEntries = { ...state.uncheckedEntries }
      const currentUnchecked = state.uncheckedEntries[action.categoryId] || new Set<string>()
      const checkedEntriesCount = action.validEntries.filter(
        entry => !currentUnchecked.has(entry.id),
      ).length
      const hasAnyChecked = checkedEntriesCount > 0
      uncheckedEntries[action.categoryId] = hasAnyChecked
        ? new Set<string>(action.validEntries.map(entry => entry.id))
        : new Set<string>()
      return { ...state, uncheckedEntries }
    }
    case 'initDefaults': {
      const uncheckedEntries = { ...state.uncheckedEntries }
      const applied = new Set(state.defaultsAppliedCategories)
      let changed = false
      for (const categoryId of CLEANUP_CATEGORIES.map(c => c.id)) {
        if (applied.has(categoryId)) continue
        const report = action.reports[categoryId]
        if (!report) continue
        const ids = defaultFalseEntryIds(report)
        if (ids.length > 0) {
          const existing = uncheckedEntries[categoryId] || new Set<string>()
          uncheckedEntries[categoryId] = new Set([...existing, ...ids])
        }
        applied.add(categoryId)
        changed = true
      }
      if (!changed) return state
      return { ...state, uncheckedEntries, defaultsAppliedCategories: applied }
    }
    case 'resetToDefaults': {
      const uncheckedEntries = { ...state.uncheckedEntries }
      uncheckedEntries[action.categoryId] = new Set(action.defaultFalseEntryIds)
      return { ...state, uncheckedEntries }
    }
  }
}

function refreshCleanupCategory(categoryId: CleanupCategoryId) {
  addRefreshingCategories([categoryId])
  cleanupScanCache.refreshCategory(categoryId).finally(() => {
    removeRefreshingCategories([categoryId])
  })
}

function CleanupPage() {
  const { t } = useTranslation()
  const cleanupUiState = useCleanupUiState()
  const { reports } = useCleanupReports()
  const [busyAction, setBusyAction] = useState<BusyAction>(null)
  const busyActionRef = useRef<BusyAction>(null)
  const [selection, dispatchSelection] = useReducer(cleanupSelectionReducer, INITIAL_CLEANUP_SELECTION)
  const selectionRef = useRef(selection)

  useEffect(() => {
    selectionRef.current = selection
  }, [selection])

  useEffect(() => {
    if (Object.keys(reports).length > 0) {
      dispatchSelection({ type: 'initDefaults', reports })
    }
  }, [reports])

  function setBusyActionState(action: BusyAction) {
    busyActionRef.current = action
    setBusyAction(action)
    setCleanupBusy(action !== null)
  }

  useEffect(() => {
    dispatchCleanupSummary(
      cleanupSummaryFromReports(reports, selection.checkedCategories, selection.uncheckedEntries),
    )
  }, [reports, selection.checkedCategories, selection.uncheckedEntries])

  useMountEffect(() => {
    return () => dispatchCleanupSummary()
  })

  function refreshAllCategories() {
    if (busyActionRef.current !== null || hasRefreshingCategories()) return

    const allIds = CLEANUP_CATEGORIES.map(category => category.id)
    addRefreshingCategories(allIds)
    cleanupScanCache.refreshAll().finally(() => {
      removeRefreshingCategories(allIds)
    })
  }

  function cleanCategory(categoryId: CleanupCategoryId) {
    if (busyActionRef.current !== null || cleanupUiState.refreshingCategories.has(categoryId)) return

    setBusyActionState(categoryId)
    const excludeEntryIds = Array.from(selection.uncheckedEntries[categoryId] || [])
    cleanCleanupCategory(categoryId, excludeEntryIds)
      .then((report) => {
        cleanupScanCache.setReport(report)
        toast.success(t('cleanup.cleaned'))
        refreshCleanupCategory(categoryId)
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

    const activeCategories = CLEANUP_CATEGORIES.filter(
      category => selectionRef.current.checkedCategories.has(category.id),
    )
    if (activeCategories.length === 0) {
      toast.error(t('cleanup.errors.nothingSelected') || 'No categories selected for cleaning')
      return
    }

    setBusyActionState('all')
    const requests = activeCategories.map(category => ({
      categoryId: category.id,
      excludeEntryIds: Array.from(selectionRef.current.uncheckedEntries[category.id] || []),
    }))
    cleanAllCleanupCategories(requests)
      .then((newReports) => {
        cleanupScanCache.setReports(newReports)
        const hasFailures = newReports.some(report =>
          report.entries.some(entry =>
            entry.id.endsWith('-scan-error') || entry.status === 'failed',
          ),
        )
        if (hasFailures) {
          toast.error(t('cleanup.errors.clean'))
        }
        else {
          toast.success(t('cleanup.cleanedAll'))
        }
      })
      .catch((error) => {
        console.error(error)
        toast.error(t('cleanup.errors.clean'))
      })
      .finally(() => {
        setBusyActionState(null)
        refreshAllCategories()
      })
  }

  function handleToggleAllCategories() {
    dispatchSelection({ type: 'toggleAllCategories' })
  }

  function toggleAllEntries(categoryId: CleanupCategoryId) {
    const report = reports[categoryId]
    if (!report) return
    const validEntries = report.entries.filter(entry => !entry.id.endsWith('-scan-error'))
    dispatchSelection({ type: 'toggleAllEntries', categoryId, validEntries })
  }

  function resetToDefaults(categoryId: CleanupCategoryId) {
    const defaultFalseIds = defaultFalseEntryIds(reports[categoryId])
    dispatchSelection({ type: 'resetToDefaults', categoryId, defaultFalseEntryIds: defaultFalseIds })
  }

  const cards = useMemo(() => CLEANUP_CATEGORIES, [])

  useMountEffect(() => {
    window.addEventListener(CLEAN_ALL_EVENT, cleanAllCategories)
    window.addEventListener(REFRESH_ALL_EVENT, refreshAllCategories)
    window.addEventListener(TOGGLE_ALL_CATEGORIES_EVENT, handleToggleAllCategories)
    return () => {
      window.removeEventListener(CLEAN_ALL_EVENT, cleanAllCategories)
      window.removeEventListener(REFRESH_ALL_EVENT, refreshAllCategories)
      window.removeEventListener(TOGGLE_ALL_CATEGORIES_EVENT, handleToggleAllCategories)
    }
  })

  return (
    <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
      <WinappDbStatusBar />
      <div className="tweak-card-grid">
        {cards.map(category => (
          <CleanupCard
            category={category}
            isBusy={busyAction === category.id || busyAction === 'all'}
            isRefreshing={cleanupUiState.refreshingCategories.has(category.id)}
            key={category.id}
            onClean={cleanCategory}
            onRefresh={refreshCleanupCategory}
            onToggle={categoryId => dispatchSelection({ type: 'toggleCard', categoryId })}
            open={selection.openCards.has(category.id)}
            report={reports[category.id] ?? null}
            isChecked={selection.checkedCategories.has(category.id)}
            onCategoryToggle={categoryId => dispatchSelection({ type: 'toggleCategory', categoryId })}
            uncheckedEntryIds={selection.uncheckedEntries[category.id] || new Set<string>()}
            onToggleEntry={(categoryId, entryId) => dispatchSelection({ type: 'toggleEntry', categoryId, entryId })}
            onToggleAllEntries={toggleAllEntries}
            onResetToDefaults={resetToDefaults}
          />
        ))}
      </div>
    </section>
  )
}

export default CleanupPage
