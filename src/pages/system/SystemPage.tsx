import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { TweakCard } from '@/features/tweak'
import { getTweaksByCategory } from '@/shared/api/tweaks'
import { PageLoader } from '@/shared/ui/page-loader'
import { Skeleton } from '@/shared/ui/skeleton'

export function SystemPage() {
  const { t } = useTranslation()

  const { data: tweaks, isLoading } = useQuery({
    queryKey: ['tweaks', 'system'],
    queryFn: () => getTweaksByCategory('system'),
  })

  return (
    <PageLoader
      isLoading={isLoading}
      skeleton={(
        <div className="space-y-6">
          <div className="space-y-2">
            <Skeleton className="h-8 w-48" />
            <Skeleton className="h-4 w-64" />
          </div>
          <div className="space-y-4">
            {Array.from({ length: 6 }).map((_, i) => (
              <Skeleton key={i} className="h-24 w-full" />
            ))}
          </div>
        </div>
      )}
    >
      <div className="space-y-6">
        <div>
          <h1 className="text-2xl font-bold">{t('sidebar.categories.system')}</h1>
          <p className="text-muted-foreground">{t('categoryDescriptions.system')}</p>
        </div>
        <div className="space-y-4">
          {tweaks?.map(tweak => (
            <TweakCard key={tweak.meta.id} tweak={tweak} />
          ))}
        </div>
      </div>
    </PageLoader>
  )
}
