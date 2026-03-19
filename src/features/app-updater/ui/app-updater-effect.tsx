import type { Update } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
import { check } from '@tauri-apps/plugin-updater'
import { useEffect, useRef } from 'react'
import { usePreferencesStore } from '@/entities/settings/model/preferences-store'
import i18n from '@/shared/i18n'
import { toast } from '@/shared/lib/toast'

const UPDATE_CHECK_INTERVAL = 30_000

export function AppUpdaterEffect() {
  const hasHydrated = usePreferencesStore(state => state.hasHydrated)
  const setUpdateChecksEnabled = usePreferencesStore(state => state.setUpdateChecksEnabled)
  const updateChecksEnabled = usePreferencesStore(state => state.updateChecksEnabled)
  const activeToastIdRef = useRef<string | undefined>(undefined)
  const deferredUntilRestartRef = useRef(false)
  const isCheckingRef = useRef(false)
  const isInstallingRef = useRef(false)
  const promptedVersionRef = useRef<string | null>(null)

  useEffect(() => {
    if (!hasHydrated) {
      return
    }

    if (!updateChecksEnabled && activeToastIdRef.current) {
      toast.dismiss(activeToastIdRef.current)
      activeToastIdRef.current = undefined
    }
  }, [hasHydrated, updateChecksEnabled])

  useEffect(() => {
    if (
      import.meta.env.DEV
        || !hasHydrated
        || !updateChecksEnabled
    ) {
      return
    }

    let cancelled = false

    const showUpdatePrompt = (update: Update) => {
      activeToastIdRef.current = toast.action(i18n.t('settings.updatePromptTitle'), {
        action: {
          label: i18n.t('settings.updateNow'),
          onClick: async () => {
            isInstallingRef.current = true
            activeToastIdRef.current = undefined
            toast.message(i18n.t('settings.installingUpdate'))

            try {
              await update.downloadAndInstall()
              await relaunch()
            }
            catch (error) {
              promptedVersionRef.current = null
              console.warn('Failed to install update', error)
              toast.error(i18n.t('settings.updateInstallFailed'))
            }
            finally {
              isInstallingRef.current = false
            }
          },
        },
        cancel: {
          label: i18n.t('settings.disableUpdateChecks'),
          onClick: () => {
            setUpdateChecksEnabled(false)
            activeToastIdRef.current = undefined
            toast.success(i18n.t('settings.updateChecksDisabled'))
          },
        },
        description: i18n.t('settings.updatePromptDescription', { version: update.version }),
        duration: Number.POSITIVE_INFINITY,
      }) as string
    }

    const runCheck = async () => {
      if (
        cancelled
        || deferredUntilRestartRef.current
        || isCheckingRef.current
        || isInstallingRef.current
      ) {
        return
      }

      isCheckingRef.current = true

      try {
        const update = await check()

        if (cancelled || !updateChecksEnabled) {
          return
        }

        if (!update) {
          promptedVersionRef.current = null
          return
        }

        if (promptedVersionRef.current === update.version) {
          return
        }

        promptedVersionRef.current = update.version
        showUpdatePrompt(update)
      }
      catch (error) {
        console.warn('Failed to check for updates', error)
      }
      finally {
        isCheckingRef.current = false
      }
    }

    void runCheck()

    const interval = window.setInterval(() => {
      void runCheck()
    }, UPDATE_CHECK_INTERVAL)

    return () => {
      cancelled = true
      window.clearInterval(interval)
    }
  }, [hasHydrated, setUpdateChecksEnabled, updateChecksEnabled])

  return null
}
