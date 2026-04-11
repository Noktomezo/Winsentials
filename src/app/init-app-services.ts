import type { PreferencesState } from '@/entities/settings/model/preferences-store'
import { usePreferencesStore } from '@/entities/settings/model/preferences-store'
import { initAppUpdater } from '@/features/app-updater/ui/app-updater-effect'
import i18n from '@/shared/i18n'
import { resolveLanguage } from '@/shared/i18n/resolve-language'
import { syncWebviewMaterial } from '@/shared/lib/desktop/window-effects'
import {
  resolveThemePreference,
  subscribeToSystemThemeChange,
} from '@/shared/lib/theme/resolve-theme'
import { toast } from '@/shared/lib/toast'

function applyDocumentAppearance(state: PreferencesState) {
  if (!state.hasHydrated) {
    return
  }

  const resolvedTheme = resolveThemePreference(state.theme)

  document.documentElement.dataset.theme = resolvedTheme
  document.documentElement.dataset.webviewMaterial = state.webviewMaterial
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

function createWebviewMaterialManager() {
  let applyRequestId = 0

  const syncMaterial = (state: PreferencesState) => {
    if (!state.hasHydrated) {
      return
    }

    const requestId = ++applyRequestId
    const resolvedTheme = resolveThemePreference(state.theme)

    void syncWebviewMaterial({
      material: state.webviewMaterial,
      theme: resolvedTheme,
    }).then((applied) => {
      if (requestId !== applyRequestId) {
        return
      }

      if (state.webviewMaterial !== 'none' && !applied) {
        toast.error(i18n.t('settings.materialUnavailable'))
        usePreferencesStore.getState().setWebviewMaterial('none')
      }
    }).catch(() => {
      if (requestId !== applyRequestId) {
        return
      }

      if (state.webviewMaterial !== 'none') {
        toast.error(i18n.t('settings.materialUnavailable'))
        usePreferencesStore.getState().setWebviewMaterial('none')
      }
    })
  }

  return { syncMaterial }
}

function createThemeManager(onSystemThemeChange: () => void) {
  let removeThemeListener: (() => void) | null = null

  const syncTheme = (state: PreferencesState) => {
    if (removeThemeListener) {
      removeThemeListener()
      removeThemeListener = null
    }

    if (!state.hasHydrated || state.theme !== 'system') {
      return
    }

    removeThemeListener = subscribeToSystemThemeChange(onSystemThemeChange)
  }

  return { syncTheme }
}

let initialized = false

export function initAppServices() {
  if (initialized) {
    return
  }

  initialized = true

  const languageManager = createLanguageManager()
  const materialManager = createWebviewMaterialManager()
  const themeManager = createThemeManager(() => {
    const state = usePreferencesStore.getState()
    applyDocumentAppearance(state)
    materialManager.syncMaterial(state)
  })

  const syncAll = () => {
    const state = usePreferencesStore.getState()
    applyDocumentAppearance(state)
    languageManager.syncLanguage(state)
    themeManager.syncTheme(state)
  }

  const syncMaterial = () => {
    materialManager.syncMaterial(usePreferencesStore.getState())
  }

  syncAll()
  syncMaterial()

  usePreferencesStore.subscribe((state, previousState) => {
    const appearanceOrLanguageChanged = state.hasHydrated !== previousState.hasHydrated
      || state.webviewMaterial !== previousState.webviewMaterial
      || state.language !== previousState.language
      || state.theme !== previousState.theme

    if (appearanceOrLanguageChanged) {
      syncAll()
    }

    if (
      state.hasHydrated !== previousState.hasHydrated
      || state.webviewMaterial !== previousState.webviewMaterial
      || state.theme !== previousState.theme
    ) {
      syncMaterial()
    }
  })

  initAppUpdater()
}
