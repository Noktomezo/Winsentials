import { Rocket } from 'lucide-react'
import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'

import { AutostartFilters } from '@/features/autostart/AutostartFilters'
import { AutostartTable } from '@/features/autostart/AutostartTable'
import { useAutostartStore } from '@/shared/store/autostart'

export function AutostartPage() {
  const { t } = useTranslation()
  const { load, loading } = useAutostartStore()

  useEffect(() => {
    load()
  }, [load])

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

      {loading
        ? (
            <div className="rounded-lg border border-border bg-card p-8 text-center text-muted-foreground">
              {t('autostart.loading')}
            </div>
          )
        : (
            <AutostartTable />
          )}
    </div>
  )
}
