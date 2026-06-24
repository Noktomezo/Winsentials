import type { TFunction } from 'i18next'
import type { StartupSource } from '@/entities/startup/model/types'
import { ChevronDown, Filter, Search } from 'lucide-react'
import { useLayoutEffect, useRef, useState } from 'react'
import { cn } from '@/shared/lib/utils'
import {
  Button,
  Input,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/shared/ui'

interface StartupFiltersProps {
  search: string
  setSearch: (value: string) => void
  sourceFilter: StartupSource | 'all'
  setSourceFilter: (value: StartupSource | 'all') => void
  statusFilter: 'all' | 'enabled' | 'disabled'
  setStatusFilter: (value: 'all' | 'enabled' | 'disabled') => void
  t: TFunction
}

export function StartupFilters({
  search,
  setSearch,
  sourceFilter,
  setSourceFilter,
  statusFilter,
  setStatusFilter,
  t,
}: StartupFiltersProps) {
  const [filtersOpen, setFiltersOpen] = useState(false)
  const [filtersHeight, setFiltersHeight] = useState(0)
  const filtersOuterRef = useRef<HTMLDivElement>(null)
  const filtersInnerRef = useRef<HTMLDivElement>(null)

  const hasActiveFilters = sourceFilter !== 'all' || statusFilter !== 'all'

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

  return (
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
          className={cn(
            'shrink-0 relative overflow-visible',
            (filtersOpen || hasActiveFilters)
            && 'border-[color:color-mix(in_oklch,var(--border)_58%,var(--primary)_42%)] bg-[color:color-mix(in_oklch,var(--secondary)_72%,var(--primary)_28%)] text-foreground hover:bg-[color:color-mix(in_oklch,var(--secondary)_62%,var(--primary)_38%)]',
          )}
          onClick={() => setFiltersOpen(open => !open)}
          size="icon-sm"
          type="button"
          variant="outline"
        >
          <span className="relative inline-flex size-4 items-center justify-center">
            <Filter
              className={cn(
                'size-4 transition-transform duration-200 ease-out',
                filtersOpen && '-translate-y-px scale-95',
              )}
            />
            <ChevronDown
              className={cn(
                'absolute -right-1 -bottom-1 size-2.5 rounded-full bg-background/85 text-muted-foreground transition-all duration-200 ease-out',
                filtersOpen ? 'rotate-180 text-foreground' : 'rotate-0',
              )}
            />
            <span
              aria-hidden="true"
              className={cn(
                'absolute -top-0.5 -right-0.5 size-2 rounded-full border border-background transition-all duration-200 ease-out',
                hasActiveFilters ? 'scale-100 bg-primary opacity-100' : 'scale-0 opacity-0',
              )}
            />
          </span>
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
  )
}
