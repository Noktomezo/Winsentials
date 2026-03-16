import type { TweakMeta } from '@/entities/tweak/model/types'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { applyTweak, runTweakExtra } from '@/entities/tweak/api'
import { EMPTY_CATEGORY, useTweakCacheStore } from '@/entities/tweak/model/tweak-cache-store'
import {
  TweakCard,
  TweakCardSkeleton,
} from '@/features/tweak-card/ui/tweak-card'
import { toast } from '@/shared/lib/toast'
import { Button } from '@/shared/ui/button'

function getErrorMessage(error: unknown, fallback: string) {
  if (error instanceof Error && error.message) {
    return error.message
  }

  if (typeof error === 'string' && error) {
    return error
  }

  return fallback
}

interface TweakCategoryPageProps {
  category: string
}

export function TweakCategoryPage({ category }: TweakCategoryPageProps) {
  const { t } = useTranslation()
  const cachedCategory = useTweakCacheStore(state => state.categories[category])
  const currentBuild = useTweakCacheStore(state => state.windowsBuild)
  const ensureCategory = useTweakCacheStore(state => state.ensureCategory)
  const revalidateCategory = useTweakCacheStore(state => state.revalidateCategory)
  const updateCachedTweak = useTweakCacheStore(state => state.updateCachedTweak)
  const [pendingIds, setPendingIds] = useState<string[]>([])
  const categoryState = cachedCategory ?? EMPTY_CATEGORY

  useEffect(() => {
    const cached = useTweakCacheStore.getState().categories[category]

    if (cached?.hasLoaded) {
      void revalidateCategory(category)
      return
    }

    void ensureCategory(category)
  }, [category, ensureCategory, revalidateCategory])

  const setPending = (id: string, pending: boolean) => {
    setPendingIds(current =>
      pending
        ? [...new Set([...current, id])]
        : current.filter(item => item !== id),
    )
  }

  const handleRestartExplorer = async (id: string) => {
    setPending(id, true)

    try {
      await runTweakExtra(id)
      toast.success(t('tweaks.success.restartApp', { appName: t('tweaks.apps.explorer') }))
    }
    catch (restartError) {
      toast.error(t('tweaks.errors.restartApp'), {
        description: getErrorMessage(restartError, t('tweaks.errors.restartApp')),
      })
    }
    finally {
      setPending(id, false)
    }
  }

  const handleToggle = async (tweak: TweakMeta, checked: boolean) => {
    const nextValue = checked ? 'enabled' : 'disabled'
    setPending(tweak.id, true)

    try {
      const result = await applyTweak(tweak.id, nextValue)
      updateCachedTweak(category, tweak.id, result.currentValue)

      if (tweak.requiresAction.type === 'restart_app' && tweak.requiresAction.appName === 'Explorer') {
        toast.action(t('tweaks.prompts.restartExplorer'), {
          action: {
            label: t('tweaks.actions.restartNow'),
            onClick: () => {
              void handleRestartExplorer(tweak.id)
            },
          },
          cancel: {
            label: t('tweaks.actions.later'),
            onClick: () => {},
          },
        })
      }

      if (tweak.requiresAction.type === 'restart_app' && tweak.requiresAction.appName !== 'Explorer') {
        toast.message(t('tweaks.prompts.restartApp', { appName: tweak.requiresAction.appName }))
      }

      if (tweak.requiresAction.type === 'restart_pc') {
        toast.message(t('tweaks.prompts.restartPc'), {
          description: t('tweaks.prompts.restartPcDescription'),
          duration: 7000,
        })
      }

      if (tweak.requiresAction.type === 'logout') {
        toast.message(t('tweaks.prompts.logout'))
      }

      if (tweak.requiresAction.type === 'restart_service') {
        toast.message(t('tweaks.prompts.restartService', { serviceName: tweak.requiresAction.serviceName }))
      }

      if (tweak.requiresAction.type === 'restart_device') {
        toast.message(t('tweaks.prompts.restartDevice', { deviceName: tweak.requiresAction.deviceName }))
      }
    }
    catch (applyError) {
      toast.error(t('tweaks.errors.apply'), {
        description: getErrorMessage(applyError, t('tweaks.errors.apply')),
      })
    }
    finally {
      setPending(tweak.id, false)
    }
  }

  return (
    <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
      {!categoryState.hasLoaded && (
        <>
          <TweakCardSkeleton />
          <TweakCardSkeleton />
        </>
      )}

      {!categoryState.hasLoaded && categoryState.error && (
        <div className="rounded-xl border border-destructive/30 bg-destructive/5 p-4">
          <div className="space-y-3">
            <div className="space-y-1">
              <h2 className="text-sm font-medium text-foreground">
                {t('tweaks.errors.loadCategory')}
              </h2>
              <p className="text-sm leading-6 text-muted-foreground">
                {getErrorMessage(categoryState.error, t('tweaks.errors.loadCategory'))}
              </p>
            </div>
            <Button onClick={() => void ensureCategory(category)} type="button" variant="outline">
              {t('tweaks.actions.retry')}
            </Button>
          </div>
        </div>
      )}

      {categoryState.hasLoaded && currentBuild !== null && categoryState.tweaks.map((tweak: TweakMeta) => (
        <TweakCard
          key={tweak.id}
          currentBuild={currentBuild}
          isPending={pendingIds.includes(tweak.id)}
          onToggle={checked => void handleToggle(tweak, checked)}
          tweak={tweak}
        />
      ))}
    </section>
  )
}
