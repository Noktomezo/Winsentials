import { useQuery } from '@tanstack/react-query'
import { useTranslation } from 'react-i18next'
import { TweakCard } from '@/features/tweak'
import { getTweaksByCategory } from '@/shared/api/tweaks'

export function MemoryPage() {
  const { t } = useTranslation()

  const { data: tweaks, isLoading } = useQuery({
    queryKey: ['tweaks', 'memory'],
    queryFn: () => getTweaksByCategory('memory'),
  })

  if (isLoading) {
    return <div className="text-muted-foreground">Loading...</div>
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold">{t('sidebar.categories.memory')}</h1>
        <p className="text-muted-foreground">{t('categoryDescriptions.memory')}</p>
      </div>
      <div className="space-y-4">
        {tweaks?.map(tweak => (
          <TweakCard key={tweak.meta.id} tweak={tweak} />
        ))}
      </div>
    </div>
  )
}
