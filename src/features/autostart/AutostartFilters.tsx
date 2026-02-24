import { RefreshCw, Search } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { cn } from '@/lib/utils'
import { useAutostartStore } from '@/shared/store/autostart'

const filters = ['all', 'enabled', 'disabled'] as const

export function AutostartFilters() {
  const { t } = useTranslation()
  const { filter, setFilter, search, setSearch, forceReload, loading, enriching } = useAutostartStore()

  return (
    <div className="flex flex-wrap items-center gap-3">
      <div className="relative flex-1 min-w-[200px]">
        <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
        <input
          type="text"
          placeholder={t('autostart.search')}
          value={search}
          onChange={e => setSearch(e.target.value)}
          className="w-full rounded-lg border border-border bg-background py-2 pl-9 pr-3 text-sm outline-none focus:border-primary"
        />
      </div>

      <div className="flex rounded-lg border border-border bg-background p-1">
        {filters.map(f => (
          <button
            key={f}
            type="button"
            onClick={() => setFilter(f)}
            className={cn(
              'rounded-md px-3 py-1.5 text-sm transition-colors cursor-pointer',
              filter === f
                ? 'bg-primary text-primary-foreground'
                : 'text-muted-foreground hover:text-foreground',
            )}
          >
            {t(`autostart.filter.${f}`)}
          </button>
        ))}
      </div>

      <button
        type="button"
        onClick={() => forceReload()}
        disabled={loading || enriching}
        className="flex items-center gap-2 rounded-lg border border-border bg-background px-3 py-2 text-sm hover:bg-accent disabled:opacity-50 cursor-pointer"
      >
        <RefreshCw className={cn('h-4 w-4', (loading || enriching) && 'animate-spin')} />
        {t('autostart.refresh')}
      </button>
    </div>
  )
}
