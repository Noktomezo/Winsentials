import type { StartupEntry, StartupSource } from '@/entities/startup/model/types'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { openPath, revealItemInDir } from '@tauri-apps/plugin-opener'
import {
  CalendarClock,
  Clock3,
  Copy,
  Database,
  Filter,
  FolderUp,
  MoreHorizontal,
  Search,
  Trash2,
  UserRound,
  UsersRound,
} from 'lucide-react'
import { memo, useDeferredValue, useLayoutEffect, useMemo, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useStartupStore } from '@/entities/startup/model/startup-store'
import { useMountEffect } from '@/shared/lib/hooks/use-mount-effect'
import { toast } from '@/shared/lib/toast'
import { cn } from '@/shared/lib/utils'
import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
  Input,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Separator,
  Skeleton,
  Switch,
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/shared/ui'

const startupSources: StartupSource[] = ['registry', 'startup_folder', 'scheduled_task']

function sourceIcon(source: StartupSource) {
  switch (source) {
    case 'registry':
      return Database
    case 'startup_folder':
      return FolderUp
    case 'scheduled_task':
      return CalendarClock
  }
}

function sourceColor(source: StartupSource) {
  switch (source) {
    case 'registry':
      return 'bg-sky-500/12 text-sky-700 dark:text-sky-300'
    case 'startup_folder':
      return 'bg-emerald-500/12 text-emerald-700 dark:text-emerald-300'
    case 'scheduled_task':
      return 'bg-amber-500/12 text-amber-700 dark:text-amber-300'
  }
}

function sourceMetaColor(source: StartupSource) {
  switch (source) {
    case 'registry':
      return 'text-sky-700 transition-[color,filter] hover:text-sky-800 hover:drop-shadow-[0_0_8px_currentColor] dark:text-sky-300 dark:hover:text-sky-200'
    case 'startup_folder':
      return 'text-emerald-700 transition-[color,filter] hover:text-emerald-800 hover:drop-shadow-[0_0_8px_currentColor] dark:text-emerald-300 dark:hover:text-emerald-200'
    case 'scheduled_task':
      return 'text-amber-700 transition-[color,filter] hover:text-amber-800 hover:drop-shadow-[0_0_8px_currentColor] dark:text-amber-300 dark:hover:text-amber-200'
  }
}

function sourceLabel(source: StartupSource, t: ReturnType<typeof useTranslation>['t']) {
  switch (source) {
    case 'registry':
      return t('startup.sources.registry')
    case 'startup_folder':
      return t('startup.sources.startupFolder')
    case 'scheduled_task':
      return t('startup.sources.scheduledTask')
  }
}

function scopeLabel(scope: StartupEntry['scope'], t: ReturnType<typeof useTranslation>['t']) {
  return scope === 'all_users'
    ? t('startup.scope.allUsers')
    : t('startup.scope.currentUser')
}

function scopeBadgeIcon(scope: StartupEntry['scope']) {
  return scope === 'all_users' ? UsersRound : UserRound
}

function publisherSummary(entry: StartupEntry, t: ReturnType<typeof useTranslation>['t']) {
  return entry.publisher?.trim() || t('startup.publishers.unknown')
}

interface StartupCardProps {
  entry: StartupEntry
  isPending: boolean
  onDelete: () => void
  onDisable: () => void
  onEnable: () => void
  onCopy: (value: string | null | undefined, successKey: string) => void
  onOpenLocation: (path: string | null | undefined) => void
  onReveal: (path: string | null | undefined) => void
}

function localizeStartupError(
  error: string | null,
  t: ReturnType<typeof useTranslation>['t'],
) {
  switch (error) {
    case 'Failed to load startup entries.':
    case 'Unknown startup source error.':
      return t('startup.errors.load')
    default:
      return error ? t('startup.errors.partial') : null
  }
}

const StartupCard = memo(({
  entry,
  isPending,
  onDelete,
  onDisable,
  onEnable,
  onCopy,
  onOpenLocation,
  onReveal,
}: StartupCardProps) => {
  const { t } = useTranslation()
  const Icon = sourceIcon(entry.source)
  const ScopeIcon = scopeBadgeIcon(entry.scope)

  return (
    <section className="rounded-xl border border-border/70 bg-card p-4">
      <div className="min-w-0 flex items-start gap-3">
        <span className={cn(
          'flex size-9 shrink-0 items-center justify-center rounded-lg',
          sourceColor(entry.source),
        )}
        >
          {entry.iconDataUrl
            ? <img alt="" className="size-4 object-contain" src={entry.iconDataUrl} />
            : <Icon className="size-4" />}
        </span>
        <div className="min-w-0 flex-1 space-y-1">
          <div className="flex items-start justify-between gap-4">
            <div className="min-w-0 flex flex-wrap items-center gap-2">
              <h2 className="truncate text-sm font-medium text-foreground">{entry.displayName}</h2>
              <Tooltip>
                <TooltipTrigger asChild>
                  <button
                    aria-label={sourceLabel(entry.source, t)}
                    className={cn(
                      'inline-flex items-center justify-center',
                      sourceMetaColor(entry.source),
                    )}
                    type="button"
                  >
                    <Icon className="size-3" />
                  </button>
                </TooltipTrigger>
                <TooltipContent sideOffset={8}>
                  {sourceLabel(entry.source, t)}
                </TooltipContent>
              </Tooltip>
              <Tooltip>
                <TooltipTrigger asChild>
                  <button
                    aria-label={scopeLabel(entry.scope, t)}
                    className="inline-flex items-center justify-center text-muted-foreground transition-[color,filter] hover:text-accent-foreground hover:drop-shadow-[0_0_8px_currentColor]"
                    type="button"
                  >
                    <ScopeIcon className="size-3" />
                  </button>
                </TooltipTrigger>
                <TooltipContent sideOffset={8}>
                  {scopeLabel(entry.scope, t)}
                </TooltipContent>
              </Tooltip>
              {entry.runOnce && (
                <Tooltip>
                  <TooltipTrigger asChild>
                    <button
                      aria-label={t('startup.badges.runOnce')}
                      className="inline-flex items-center justify-center text-primary transition-[color,filter] hover:text-primary/90 hover:drop-shadow-[0_0_8px_currentColor]"
                      type="button"
                    >
                      <Clock3 className="size-3" />
                    </button>
                  </TooltipTrigger>
                  <TooltipContent sideOffset={8}>
                    {t('startup.badges.runOnce')}
                  </TooltipContent>
                </Tooltip>
              )}
            </div>
            <div className="flex shrink-0 items-center gap-1 self-start">
              <Tooltip>
                <TooltipTrigger asChild>
                  <span className="flex h-6 items-center">
                    <Switch
                      aria-label={entry.status === 'enabled'
                        ? t('startup.actions.disable')
                        : t('startup.actions.enable')}
                      checked={entry.status === 'enabled'}
                      disabled={isPending}
                      onCheckedChange={checked => void (checked ? onEnable() : onDisable())}
                    />
                  </span>
                </TooltipTrigger>
                <TooltipContent sideOffset={8}>
                  {entry.status === 'enabled'
                    ? t('startup.actions.disable')
                    : t('startup.actions.enable')}
                </TooltipContent>
              </Tooltip>
              <Tooltip>
                <DropdownMenu>
                  <TooltipTrigger asChild>
                    <DropdownMenuTrigger asChild>
                      <Button
                        aria-label={t('startup.actions.more')}
                        disabled={isPending}
                        size="icon-xs"
                        type="button"
                        variant="ghost"
                      >
                        <MoreHorizontal className="size-3.5" />
                      </Button>
                    </DropdownMenuTrigger>
                  </TooltipTrigger>
                  <TooltipContent sideOffset={8}>{t('startup.actions.more')}</TooltipContent>
                  <DropdownMenuContent align="end">
                    {entry.source === 'registry' && (
                      <>
                        <DropdownMenuItem
                          disabled={!entry.command}
                          onSelect={() => onCopy(entry.command, 'startup.success.copyCommand')}
                        >
                          <Copy className="size-3.5" />
                          {t('startup.actions.copyCommand')}
                        </DropdownMenuItem>
                        <DropdownMenuItem
                          disabled={!entry.registryPath}
                          onSelect={() => onCopy(entry.registryPath, 'startup.success.copyRegistryPath')}
                        >
                          <Copy className="size-3.5" />
                          {t('startup.actions.copyRegistryPath')}
                        </DropdownMenuItem>
                      </>
                    )}

                    {entry.source === 'startup_folder' && (
                      <>
                        <DropdownMenuItem
                          disabled={!entry.targetPath}
                          onSelect={() => onReveal(entry.targetPath)}
                        >
                          <FolderUp className="size-3.5" />
                          {t('startup.actions.openContainingFolder')}
                        </DropdownMenuItem>
                        <DropdownMenuItem
                          disabled={!entry.targetPath}
                          onSelect={() => onCopy(entry.targetPath, 'startup.success.copyPath')}
                        >
                          <Copy className="size-3.5" />
                          {t('startup.actions.copyPath')}
                        </DropdownMenuItem>
                      </>
                    )}

                    {entry.source === 'scheduled_task' && (
                      <DropdownMenuItem
                        disabled={!entry.taskPath}
                        onSelect={() => onCopy(entry.taskPath, 'startup.success.copyTaskPath')}
                      >
                        <Copy className="size-3.5" />
                        {t('startup.actions.copyTaskPath')}
                      </DropdownMenuItem>
                    )}

                    {entry.source !== 'scheduled_task' && <DropdownMenuSeparator />}
                    {entry.source === 'startup_folder' && (
                      <DropdownMenuItem
                        disabled={!entry.locationLabel}
                        onSelect={() => onOpenLocation(entry.locationLabel)}
                      >
                        <FolderUp className="size-3.5" />
                        {t('startup.actions.openLocation')}
                      </DropdownMenuItem>
                    )}
                    <DropdownMenuItem
                      disabled={!entry.locationLabel}
                      onSelect={() => onCopy(entry.locationLabel, 'startup.success.copyPath')}
                    >
                      <Copy className="size-3.5" />
                      {t('startup.actions.copyLocation')}
                    </DropdownMenuItem>
                    <DropdownMenuSeparator />
                    <DropdownMenuItem
                      disabled={isPending}
                      onSelect={onDelete}
                      variant="destructive"
                    >
                      <Trash2 className="size-3.5" />
                      {t('startup.actions.delete')}
                    </DropdownMenuItem>
                  </DropdownMenuContent>
                </DropdownMenu>
              </Tooltip>
            </div>
          </div>
          <p className="text-xs leading-5 text-muted-foreground">
            {publisherSummary(entry, t)}
          </p>
        </div>
      </div>
    </section>
  )
})

export function StartupPage() {
  const { t } = useTranslation()
  const [filtersOpen, setFiltersOpen] = useState(false)
  const [filtersHeight, setFiltersHeight] = useState(0)
  const [entryToDelete, setEntryToDelete] = useState<StartupEntry | null>(null)
  const filtersOuterRef = useRef<HTMLDivElement>(null)
  const filtersInnerRef = useRef<HTMLDivElement>(null)
  const search = useStartupStore(state => state.search)
  const deferredSearch = useDeferredValue(search)
  const entries = useStartupStore(state => state.entries)
  const sourceLoading = useStartupStore(state => state.sourceLoading)
  const hasSettledSource = useStartupStore(state => state.hasSettledSource)
  const sourceFilter = useStartupStore(state => state.sourceFilter)
  const statusFilter = useStartupStore(state => state.statusFilter)
  const error = useStartupStore(state => state.error)
  const pendingIds = useStartupStore(state => state.pendingIds)
  const setSearch = useStartupStore(state => state.setSearch)
  const setSourceFilter = useStartupStore(state => state.setSourceFilter)
  const setStatusFilter = useStartupStore(state => state.setStatusFilter)
  const loadAllEntriesProgressive = useStartupStore(state => state.loadAllEntriesProgressive)
  const enableEntry = useStartupStore(state => state.enableEntry)
  const disableEntry = useStartupStore(state => state.disableEntry)
  const deleteEntry = useStartupStore(state => state.deleteEntry)

  useMountEffect(() => {
    void loadAllEntriesProgressive()
  })

  useLayoutEffect(() => {
    const inner = filtersInnerRef.current
    if (!inner) {
      return
    }

    const updateHeight = () => {
      setFiltersHeight(inner.scrollHeight)
    }

    updateHeight()
    const observer = new ResizeObserver(() => {
      updateHeight()
    })
    observer.observe(inner)

    return () => observer.disconnect()
  }, [])

  useLayoutEffect(() => {
    const inner = filtersInnerRef.current
    if (!inner) {
      return
    }

    setFiltersHeight(inner.scrollHeight)
  }, [filtersOpen, t])

  const normalizedSearch = deferredSearch.trim().toLowerCase()

  const visibleEntries = useMemo(() => entries.filter((entry) => {
    if (sourceFilter !== 'all' && entry.source !== sourceFilter) {
      return false
    }

    if (statusFilter !== 'all' && entry.status !== statusFilter) {
      return false
    }

    if (!normalizedSearch) {
      return true
    }

    return [
      entry.displayName,
      entry.name,
      entry.command,
      entry.locationLabel,
      entry.sourceDisplay,
      entry.publisher,
      entry.registryPath,
      entry.taskPath,
      entry.targetPath,
    ]
      .filter(Boolean)
      .some(value => value!.toLowerCase().includes(normalizedSearch))
  }), [entries, normalizedSearch, sourceFilter, statusFilter])

  const anyLoading = startupSources.some(source => sourceLoading[source])
  const anyLoaded = startupSources.some(source => hasSettledSource[source])
  const allLoaded = startupSources.every(source => hasSettledSource[source])
  const localizedError = localizeStartupError(error, t)

  function handleCopy(value: string | null | undefined, successKey: string) {
    if (!value) {
      return
    }

    void writeText(value)
      .then(() => {
        toast.success(t(successKey))
      })
      .catch(() => {
        toast.error(t('startup.errors.copy'))
      })
  }

  function handleReveal(path: string | null | undefined) {
    if (!path) {
      return
    }

    void revealItemInDir(path)
      .catch(() => {
        toast.error(t('startup.errors.openFolder'))
      })
  }

  function handleOpenLocation(path: string | null | undefined) {
    if (!path) {
      return
    }

    void openPath(path)
      .catch(() => {
        toast.error(t('startup.errors.openFolder'))
      })
  }

  function handleDelete(entry: StartupEntry) {
    setEntryToDelete(entry)
  }

  async function confirmDelete() {
    if (!entryToDelete) {
      return
    }

    try {
      await deleteEntry(entryToDelete.id)
      setEntryToDelete(null)
      toast.success(t('startup.success.deleted'))
    }
    catch {
      toast.error(t('startup.errors.delete'))
    }
  }

  async function handleEnable(entry: StartupEntry) {
    try {
      await enableEntry(entry.id)
      toast.success(t('startup.success.enabled'))
    }
    catch {
      toast.error(t('startup.errors.enable'))
    }
  }

  async function handleDisable(entry: StartupEntry) {
    try {
      await disableEntry(entry.id)
      toast.success(t('startup.success.disabled'))
    }
    catch {
      toast.error(t('startup.errors.disable'))
    }
  }

  return (
    <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
      <Dialog
        onOpenChange={open => !open && setEntryToDelete(null)}
        open={entryToDelete !== null}
      >
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{t('startup.confirm.deleteTitle')}</DialogTitle>
            <DialogDescription>
              {t('startup.confirm.deleteDescription', { name: entryToDelete?.displayName ?? '' })}
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button
              onClick={() => setEntryToDelete(null)}
              type="button"
              variant="outline"
            >
              {t('tweaks.actions.later')}
            </Button>
            <Button
              onClick={() => { void confirmDelete() }}
              type="button"
              variant="destructive"
            >
              <Trash2 className="size-4" />
              {t('startup.actions.delete')}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <div>
        <div className="flex items-center gap-3">
          <div className="relative min-w-0 flex-1">
            <Search className="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              aria-label={t('startup.searchLabel')}
              className="pl-9"
              onChange={event => setSearch(event.currentTarget.value)}
              placeholder={t('startup.searchPlaceholder')}
              value={search}
            />
          </div>
          <Button
            aria-expanded={filtersOpen}
            aria-label={t('startup.filters.source')}
            className="shrink-0"
            onClick={() => setFiltersOpen(open => !open)}
            size="icon-sm"
            type="button"
            variant="outline"
          >
            <Filter className="size-4" />
          </Button>
        </div>

        <div
          className={cn(
            'overflow-hidden transition-[height,opacity,margin-top] duration-200 ease-out',
            filtersOpen
              ? 'mt-3 opacity-100 pointer-events-auto'
              : 'mt-0 opacity-0 pointer-events-none',
          )}
          ref={filtersOuterRef}
          style={{ height: filtersOpen ? `${filtersHeight}px` : '0px' }}
        >
          <div
            className={cn(
              'transition-[transform,opacity] duration-200 ease-out',
              filtersOpen ? 'translate-y-0 opacity-100' : '-translate-y-1.5 opacity-0',
            )}
            ref={filtersInnerRef}
          >
            <div className="flex flex-wrap items-center gap-3">
              <Select onValueChange={value => setSourceFilter(value as typeof sourceFilter)} value={sourceFilter}>
                <SelectTrigger className="min-w-0 flex-1">
                  <SelectValue placeholder={t('startup.filters.source')} />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">{t('startup.filters.allSources')}</SelectItem>
                  <SelectItem value="registry">{t('startup.sources.registry')}</SelectItem>
                  <SelectItem value="startup_folder">{t('startup.sources.startupFolder')}</SelectItem>
                  <SelectItem value="scheduled_task">{t('startup.sources.scheduledTask')}</SelectItem>
                </SelectContent>
              </Select>
              <Select onValueChange={value => setStatusFilter(value as typeof statusFilter)} value={statusFilter}>
                <SelectTrigger className="min-w-0 flex-1">
                  <SelectValue placeholder={t('startup.filters.status')} />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="all">{t('startup.filters.allStatuses')}</SelectItem>
                  <SelectItem value="enabled">{t('startup.status.enabled')}</SelectItem>
                  <SelectItem value="disabled">{t('startup.status.disabled')}</SelectItem>
                </SelectContent>
              </Select>
            </div>
          </div>
        </div>
      </div>

      <Separator />

      {localizedError && (
        <section className="rounded-xl border border-destructive/30 bg-destructive/8 p-4">
          <p className="text-sm text-muted-foreground">{localizedError}</p>
        </section>
      )}

      {!anyLoaded && anyLoading
        ? (
            <div className="space-y-3">
              {Array.from({ length: 4 }).map((_, index) => (
                <section className="rounded-xl border border-border/70 bg-card p-4" key={index}>
                  <div className="min-w-0 flex items-start gap-3">
                    <Skeleton className="size-9 shrink-0 rounded-lg" />
                    <div className="min-w-0 flex-1 space-y-1">
                      <div className="flex items-start justify-between gap-4">
                        <div className="min-w-0 flex flex-wrap items-center gap-2">
                          <Skeleton className="h-4 w-48" />
                          <Skeleton className="size-3 rounded-full" />
                          <Skeleton className="size-3 rounded-full" />
                        </div>
                        <div className="flex h-6 items-center">
                          <Skeleton className="h-5 w-9 rounded-full" />
                        </div>
                      </div>
                      <Skeleton className="h-3 w-32" />
                    </div>
                  </div>
                </section>
              ))}
            </div>
          )
        : visibleEntries.length === 0
          ? (
              <section className="rounded-xl border border-border/70 bg-card p-8 text-center">
                <h2 className="text-sm font-medium text-foreground">
                  {allLoaded ? t('startup.empty.title') : t('startup.states.loading')}
                </h2>
                <p className="mt-2 text-xs leading-5 text-muted-foreground">
                  {allLoaded ? t('startup.empty.description') : t('startup.states.loadingDescription')}
                </p>
              </section>
            )
          : (
              <div className="space-y-3">
                {visibleEntries.map(entry => (
                  <StartupCard
                    entry={entry}
                    isPending={pendingIds.includes(entry.id)}
                    key={entry.id}
                    onCopy={handleCopy}
                    onDelete={() => { void handleDelete(entry) }}
                    onDisable={() => { void handleDisable(entry) }}
                    onEnable={() => { void handleEnable(entry) }}
                    onOpenLocation={handleOpenLocation}
                    onReveal={handleReveal}
                  />
                ))}
              </div>
            )}
    </section>
  )
}
