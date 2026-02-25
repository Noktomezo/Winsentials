import { Rocket } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { AutostartFilters } from '@/features/autostart/AutostartFilters'
import { AutostartTable } from '@/features/autostart/AutostartTable'
import { useAutostartStore } from '@/shared/store/autostart'
import { Skeleton } from '@/shared/ui/skeleton'

function AutostartSkeleton() {
  return (
    <div className="space-y-3">
      {[1, 2, 3, 4, 5].map(i => (
        <div key={i} className="rounded-lg border border-border bg-card p-3">
          <div className="flex items-center gap-3">
            <Skeleton className="h-8 w-8 rounded" />
            <div className="flex-1 space-y-2">
              <Skeleton className="h-4 w-1/3" />
              <Skeleton className="h-3 w-1/2" />
            </div>
            <Skeleton className="h-4 w-12" />
            <Skeleton className="h-4 w-4 rounded" />
          </div>
        </div>
      ))}
    </div>
  )
}

export function AutostartPage() {
  const { t } = useTranslation()
  const { loading } = useAutostartStore()

  return (
    <div className="space-y-6">
      <div className="flex items-center gap-3">
        <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10">
          <Rocket className="h-5 w-5 text-primary" />
        </div>
        <div>
          <h1 className="text-2xl font-bold">{t('autostart.title')}</h1>
          <p className="text-muted-foreground">{t('autostart.description')}</p>
        </div>
      </div>

      <AutostartFilters />

      {loading ? <AutostartSkeleton /> : <AutostartTable />}
    </div>
  )
}
