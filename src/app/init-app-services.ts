import type { PreferencesState } from '@/entities/settings/model/preferences-store'
import type { ResolvedTheme } from '@/shared/config/app'
import { usePreferencesStore } from '@/entities/settings/model/preferences-store'
import { initAppUpdater } from '@/features/app-updater/ui/app-updater-effect'
import i18n from '@/shared/i18n'
import { resolveLanguage } from '@/shared/i18n/resolve-language'
import { syncWebviewMaterial } from '@/shared/lib/desktop/window-effects'
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
    const resolvedTheme = resolveTheme(state.theme)

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

let initialized = false

export function initAppServices() {
  if (initialized) {
    return
  }

  initialized = true

  const languageManager = createLanguageManager()
  const materialManager = createWebviewMaterialManager()

  const syncAll = () => {
    const state = usePreferencesStore.getState()
    applyDocumentAppearance(state)
    languageManager.syncLanguage(state)
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
      || state.palette !== previousState.palette
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

  subscribeSystemTheme(() => {
    if (usePreferencesStore.getState().theme === 'system') {
      syncAll()
      syncMaterial()
    }
  })

  initAppUpdater()
}
