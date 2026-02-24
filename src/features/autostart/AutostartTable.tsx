import type { AutostartItem, AutostartSource } from '@/shared/types/autostart'
import { ChevronDown, ChevronRight, Clock, FileText, Folder, Settings } from 'lucide-react'
import { useMemo, useState } from 'react'

import { useTranslation } from 'react-i18next'
import { useAutostartStore } from '@/shared/store/autostart'
import { AutostartRow } from './AutostartRow'

const sourceConfig: Record<AutostartSource, { icon: typeof Folder, label: string }> = {
  Registry: { icon: FileText, label: 'sourceRegistry' },
  Folder: { icon: Folder, label: 'sourceFolder' },
  Task: { icon: Clock, label: 'sourceTask' },
  Service: { icon: Settings, label: 'sourceService' },
}

export function AutostartTable() {
  const { t } = useTranslation()
  const { items, filter, search } = useAutostartStore()
  const [expanded, setExpanded] = useState<Record<AutostartSource, boolean>>({
    Registry: true,
    Folder: true,
    Task: true,
    Service: true,
  })

  const filteredItems = useMemo(() => {
    return items.filter((item) => {
      const matchesFilter
        = filter === 'all'
          || (filter === 'enabled' && item.is_enabled)
          || (filter === 'disabled' && !item.is_enabled)

      const matchesSearch
        = !search
          || item.name.toLowerCase().includes(search.toLowerCase())
          || item.publisher.toLowerCase().includes(search.toLowerCase())
          || item.location.toLowerCase().includes(search.toLowerCase())

      return matchesFilter && matchesSearch
    })
  }, [items, filter, search])

  const grouped = useMemo(() => {
    const groups: Record<AutostartSource, AutostartItem[]> = {
      Registry: [],
      Folder: [],
      Task: [],
      Service: [],
    }

    for (const item of filteredItems) {
      groups[item.source].push(item)
    }

    return groups
  }, [filteredItems])

  const toggleGroup = (source: AutostartSource) => {
    setExpanded(prev => ({ ...prev, [source]: !prev[source] }))
  }

  const sources: AutostartSource[] = ['Folder', 'Registry', 'Task', 'Service']

  return (
    <div className="space-y-4">
      {sources.map((source) => {
        const groupItems = grouped[source]
        if (groupItems.length === 0)
          return null

        const config = sourceConfig[source]
        const Icon = config.icon
        const isExpanded = expanded[source]

        return (
          <div key={source} className="rounded-lg border border-border bg-card">
            <button
              type="button"
              onClick={() => toggleGroup(source)}
              className="flex w-full items-center gap-2 px-4 py-3 text-left hover:bg-accent/50 cursor-pointer"
            >
              {isExpanded
                ? (
                    <ChevronDown className="h-4 w-4 text-muted-foreground" />
                  )
                : (
                    <ChevronRight className="h-4 w-4 text-muted-foreground" />
                  )}
              <Icon className="h-4 w-4 text-muted-foreground" />
              <span className="font-medium">{t(`autostart.${config.label}`)}</span>
              <span className="ml-auto text-sm text-muted-foreground">
                {groupItems.length}
              </span>
            </button>

            {isExpanded && (
              <div className="border-t border-border p-3 space-y-2">
                {groupItems.map(item => (
                  <AutostartRow key={item.id} item={item} />
                ))}
              </div>
            )}
          </div>
        )
      })}

      {filteredItems.length === 0 && (
        <div className="rounded-lg border border-border bg-card p-8 text-center text-muted-foreground">
          {t('autostart.noItems')}
        </div>
      )}
    </div>
  )
}
