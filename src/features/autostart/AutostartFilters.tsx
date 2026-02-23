import { Download, RefreshCw, Search } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'

import { cn } from '@/lib/utils'
import { useAutostartStore } from '@/shared/store/autostart'

const filters = ['all', 'enabled', 'disabled'] as const

export function AutostartFilters() {
  const { t } = useTranslation()
  const { filter, setFilter, search, setSearch, load, loading } = useAutostartStore()
  const [exporting, setExporting] = useState(false)

  const handleExport = async () => {
    if (exporting)
      return
    setExporting(true)
    try {
      const { exportAutostart } = await import('@/shared/api/autostart')
      const csv = await exportAutostart()
      const blob = new Blob([csv], { type: 'text/csv' })
      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = 'autostart.csv'
      a.click()
      URL.revokeObjectURL(url)
    }
    catch (error) {
      console.error('Failed to export:', error)
    }
    finally {
      setExporting(false)
    }
  }

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
              'rounded-md px-3 py-1.5 text-sm transition-colors',
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
        onClick={() => load()}
        disabled={loading}
        className="flex items-center gap-2 rounded-lg border border-border bg-background px-3 py-2 text-sm hover:bg-accent disabled:opacity-50"
      >
        <RefreshCw className={cn('h-4 w-4', loading && 'animate-spin')} />
        {t('autostart.refresh')}
      </button>

      <button
        type="button"
        onClick={handleExport}
        disabled={exporting}
        className="flex items-center gap-2 rounded-lg border border-border bg-background px-3 py-2 text-sm hover:bg-accent disabled:opacity-50"
      >
        <Download className="h-4 w-4" />
        {t('autostart.exportCsv')}
      </button>
    </div>
  )
}
