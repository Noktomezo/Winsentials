import type { TweakMeta } from '@/entities/tweak/model/types'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  applyTweak,
  logoutUser,
  restartPc,
  runTweakExtra,
} from '@/entities/tweak/api'
import {
  EMPTY_CATEGORY,
  useTweakCacheStore,
} from '@/entities/tweak/model/tweak-cache-store'
import {
  TweakCard,
  TweakCardSkeleton,
} from '@/features/tweak-card/ui/tweak-card'
import { useMountEffect } from '@/shared/lib/hooks/use-mount-effect'
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
  const cachedCategory = useTweakCacheStore(
    state => state.categories[category],
  )
  const currentBuild = useTweakCacheStore(state => state.windowsBuild)
  const ensureCategory = useTweakCacheStore(state => state.ensureCategory)
  const revalidateCategory = useTweakCacheStore(
    state => state.revalidateCategory,
  )
  const updateCachedTweak = useTweakCacheStore(
    state => state.updateCachedTweak,
  )
  const [pendingIds, setPendingIds] = useState<string[]>([])
  const [extraPendingIds, setExtraPendingIds] = useState<string[]>([])
  const categoryState = cachedCategory ?? EMPTY_CATEGORY

  useMountEffect(() => {
    const cached = useTweakCacheStore.getState().categories[category]

    if (cached?.hasLoaded) {
      void revalidateCategory(category)
      return
    }

    void ensureCategory(category)
  })

  const setPending = (id: string, pending: boolean) => {
    setPendingIds(current =>
      pending
        ? [...new Set([...current, id])]
        : current.filter(item => item !== id),
    )
  }

  const setExtraPending = (id: string, pending: boolean) => {
    setExtraPendingIds(current =>
      pending
        ? [...new Set([...current, id])]
        : current.filter(item => item !== id),
    )
  }

  const handleApplyValue = async (tweak: TweakMeta, nextValue: string) => {
    setPending(tweak.id, true)

    try {
      const result = await applyTweak(tweak.id, nextValue)
      updateCachedTweak(category, tweak.id, result.currentValue)

      if (tweak.requiresAction.type === 'restart_app') {
        toast.message(
          t('tweaks.prompts.restartApp', {
            appName: tweak.requiresAction.appName,
          }),
        )
      }

      if (tweak.requiresAction.type === 'restart_pc') {
        toast.action(t('tweaks.prompts.restartPc'), {
          description: t('tweaks.prompts.restartPcDescription'),
          action: {
            label: t('tweaks.actions.restartNow'),
            onClick: () => {
              void restartPc()
            },
          },
          cancel: {
            label: t('tweaks.actions.later'),
          },
          duration: Number.POSITIVE_INFINITY,
        })
      }

      if (tweak.requiresAction.type === 'logout') {
        toast.action(t('tweaks.prompts.logout'), {
          description: t('tweaks.prompts.logoutDescription'),
          action: {
            label: t('tweaks.actions.logoutNow'),
            onClick: () => {
              void logoutUser()
            },
          },
          cancel: {
            label: t('tweaks.actions.later'),
          },
          duration: Number.POSITIVE_INFINITY,
        })
      }

      if (tweak.requiresAction.type === 'restart_service') {
        toast.message(
          t('tweaks.prompts.restartService', {
            serviceName: tweak.requiresAction.serviceName,
          }),
        )
      }

      if (tweak.requiresAction.type === 'restart_device') {
        toast.message(
          t('tweaks.prompts.restartDevice', {
            deviceName: tweak.requiresAction.deviceName,
          }),
        )
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

  const handleRunExtra = (tweak: TweakMeta) => {
    if (tweak.id !== 'disable_game_dvr') {
      return
    }

    toast.action(t('tweaks.prompts.uninstallXboxGameBar'), {
      description: t('tweaks.prompts.uninstallXboxGameBarDescription'),
      action: {
        label: t('tweaks.actions.uninstallNow'),
        onClick: () => {
          setExtraPending(tweak.id, true)

          void toast.promise(
            runTweakExtra(tweak.id).finally(() => {
              setExtraPending(tweak.id, false)
            }),
            {
              loading: t('tweaks.progress.uninstallXboxGameBarLoading'),
              success: t('tweaks.success.uninstallXboxGameBar'),
              error: error =>
                getErrorMessage(error, t('tweaks.errors.uninstallXboxGameBar')),
            },
          )
        },
      },
      cancel: {
        label: t('tweaks.actions.later'),
      },
      duration: Number.POSITIVE_INFINITY,
    })
  }

  return (
    <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
      {!categoryState.hasLoaded && (
        <div className="tweak-card-grid">
          <TweakCardSkeleton />
          <TweakCardSkeleton />
        </div>
      )}

      {!categoryState.hasLoaded && categoryState.error && (
        <div className="rounded-xl border border-destructive/30 bg-destructive/5 p-4">
          <div className="space-y-3">
            <div className="space-y-1">
              <h2 className="text-sm font-medium text-foreground">
                {t('tweaks.errors.loadCategory')}
              </h2>
              <p className="text-sm leading-6 text-muted-foreground">
                {getErrorMessage(
                  categoryState.error,
                  t('tweaks.errors.loadCategory'),
                )}
              </p>
            </div>
            <Button
              onClick={() => void ensureCategory(category)}
              type="button"
              variant="outline"
            >
              {t('tweaks.actions.retry')}
            </Button>
          </div>
        </div>
      )}

      {categoryState.hasLoaded && currentBuild !== null && (
        <div className="tweak-card-grid">
          {categoryState.tweaks.map((tweak: TweakMeta) => (
            <TweakCard
              key={tweak.id}
              currentBuild={currentBuild}
              isExtraPending={extraPendingIds.includes(tweak.id)}
              isPending={pendingIds.includes(tweak.id)}
              onApplyValue={value => void handleApplyValue(tweak, value)}
              onRunExtra={() => handleRunExtra(tweak)}
              tweak={tweak}
            />
          ))}
        </div>
      )}
    </section>
  )
}
