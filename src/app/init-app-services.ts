import type { PreferencesState } from '@/entities/settings/model/preferences-store'
import type { ResolvedTheme } from '@/shared/config/app'
import { usePreferencesStore } from '@/entities/settings/model/preferences-store'
import { initAppUpdater } from '@/features/app-updater/ui/app-updater-effect'
import i18n from '@/shared/i18n'
import { resolveLanguage } from '@/shared/i18n/resolve-language'
import { syncChromeAcrylic } from '@/shared/lib/desktop/window-effects'
import { getSystemResolvedTheme, subscribeSystemTheme } from '@/shared/lib/system-theme'
import { toast } from '@/shared/lib/toast'

function resolveTheme(theme: PreferencesState['theme']): ResolvedTheme {
  return theme === 'system'
    ? getSystemResolvedTheme()
    : theme
}

function applyDocumentAppearance(state: PreferencesState) {
  if (!state.hasHydrated) {
    return
  }

  const resolvedTheme = resolveTheme(state.theme)
  document.documentElement.dataset.theme = resolvedTheme
  document.documentElement.dataset.palette = state.palette
  document.documentElement.dataset.chromeMaterial = state.chromeAcrylic
    ? 'acrylic'
    : 'solid'
  document.documentElement.style.colorScheme = resolvedTheme
}

function createLanguageManager() {
  let removeLanguageListener: (() => void) | null = null

  const syncLanguage = (state: PreferencesState) => {
    if (!state.hasHydrated) {
      return
    }

    const applyResolvedLanguage = () => {
      const resolved = resolveLanguage(usePreferencesStore.getState().language)

      if (i18n.language !== resolved) {
        void i18n.changeLanguage(resolved)
      }
    }

    applyResolvedLanguage()

    if (removeLanguageListener) {
      removeLanguageListener()
      removeLanguageListener = null
    }

    if (state.language === 'system') {
      window.addEventListener('languagechange', applyResolvedLanguage)
      removeLanguageListener = () => {
        window.removeEventListener('languagechange', applyResolvedLanguage)
      }
    }
  }

  return { syncLanguage }
}

function createChromeAcrylicManager() {
  let applyRequestId = 0

  const syncAcrylic = (state: PreferencesState) => {
    if (!state.hasHydrated) {
      return
    }

    const requestId = ++applyRequestId
    const resolvedTheme = resolveTheme(state.theme)

    void syncChromeAcrylic({
      enabled: state.chromeAcrylic,
      theme: resolvedTheme,
    }).then((applied) => {
      if (requestId !== applyRequestId) {
        return
      }

      if (state.chromeAcrylic && !applied) {
        toast.error(i18n.t('settings.acrylicUnavailable'))
        usePreferencesStore.getState().setChromeAcrylic(false)
      }
    }).catch(() => {
      if (requestId !== applyRequestId) {
        return
      }

      if (state.chromeAcrylic) {
        toast.error(i18n.t('settings.acrylicUnavailable'))
        usePreferencesStore.getState().setChromeAcrylic(false)
      }
    })
  }

  return { syncAcrylic }
}

let initialized = false

export function initAppServices() {
  if (initialized) {
    return
  }

  initialized = true

  const languageManager = createLanguageManager()
  const acrylicManager = createChromeAcrylicManager()

  const syncAll = () => {
    const state = usePreferencesStore.getState()
    applyDocumentAppearance(state)
    languageManager.syncLanguage(state)
  }

  const syncAcrylic = () => {
    acrylicManager.syncAcrylic(usePreferencesStore.getState())
  }

  syncAll()
  syncAcrylic()

  usePreferencesStore.subscribe((state, previousState) => {
    const appearanceOrLanguageChanged = state.hasHydrated !== previousState.hasHydrated
      || state.chromeAcrylic !== previousState.chromeAcrylic
      || state.language !== previousState.language
      || state.palette !== previousState.palette
      || state.theme !== previousState.theme

    if (appearanceOrLanguageChanged) {
      syncAll()
    }

    if (
      state.hasHydrated !== previousState.hasHydrated
      || state.chromeAcrylic !== previousState.chromeAcrylic
      || state.theme !== previousState.theme
    ) {
      syncAcrylic()
    }
  })

  subscribeSystemTheme(() => {
    if (usePreferencesStore.getState().theme === 'system') {
      syncAll()
      syncAcrylic()
    }
  })

  initAppUpdater()
}
