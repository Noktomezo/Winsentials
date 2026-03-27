import type { Update } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
import { check } from '@tauri-apps/plugin-updater'
import { usePreferencesStore } from '@/entities/settings/model/preferences-store'
import i18n from '@/shared/i18n'
import { toast } from '@/shared/lib/toast'

const UPDATE_CHECK_INTERVAL = 30_000

let initialized = false

export function initAppUpdater() {
  if (initialized || import.meta.env.DEV) {
    return
  }

  initialized = true

  let activeToastId: string | undefined
  let intervalId: number | null = null
  let isChecking = false
  let isInstalling = false
  let promptedVersion: string | null = null

  const dismissActiveToast = () => {
    if (activeToastId) {
      toast.dismiss(activeToastId)
      activeToastId = undefined
    }
  }

  const showUpdatePrompt = (update: Update) => {
    activeToastId = toast.action(i18n.t('settings.updatePromptTitle'), {
      action: {
        label: i18n.t('settings.updateNow'),
        onClick: async () => {
          isInstalling = true
          activeToastId = undefined
          toast.message(i18n.t('settings.installingUpdate'))

          try {
            await update.downloadAndInstall()
            await relaunch()
          }
          catch (error) {
            promptedVersion = null
            console.warn('Failed to install update', error)
            toast.error(i18n.t('settings.updateInstallFailed'))
          }
          finally {
            isInstalling = false
          }
        },
      },
      cancel: {
        label: i18n.t('settings.disableUpdateChecks'),
        onClick: () => {
          usePreferencesStore.getState().setUpdateChecksEnabled(false)
          promptedVersion = null
          activeToastId = undefined
          toast.success(i18n.t('settings.updateChecksDisabled'))
        },
      },
      description: i18n.t('settings.updatePromptDescription', { version: update.version }),
      duration: Number.POSITIVE_INFINITY,
    }) as string
  }

  const runCheck = async () => {
    const state = usePreferencesStore.getState()
    if (!state.hasHydrated || !state.updateChecksEnabled || isChecking || isInstalling) {
      return
    }

    isChecking = true

    try {
      const update = await check()
      const latestState = usePreferencesStore.getState()

      if (!latestState.updateChecksEnabled) {
        promptedVersion = null
        dismissActiveToast()
        return
      }

      if (!update) {
        promptedVersion = null
        return
      }

      if (promptedVersion === update.version) {
        return
      }

      promptedVersion = update.version
      showUpdatePrompt(update)
    }
    catch (error) {
      console.warn('Failed to check for updates', error)
    }
    finally {
      isChecking = false
    }
  }

  const syncUpdater = () => {
    const state = usePreferencesStore.getState()

    if (!state.hasHydrated || !state.updateChecksEnabled) {
      if (intervalId !== null) {
        window.clearInterval(intervalId)
        intervalId = null
      }

      promptedVersion = null
      dismissActiveToast()
      return
    }

    if (intervalId === null) {
      void runCheck()
      intervalId = window.setInterval(() => {
        void runCheck()
      }, UPDATE_CHECK_INTERVAL)
    }
  }

  syncUpdater()

  usePreferencesStore.subscribe((state, previousState) => {
    if (
      state.hasHydrated === previousState.hasHydrated
      && state.updateChecksEnabled === previousState.updateChecksEnabled
    ) {
      return
    }

    syncUpdater()
  })
}
