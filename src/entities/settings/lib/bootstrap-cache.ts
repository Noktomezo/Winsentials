import { usePreferencesStore } from '@/entities/settings/model/preferences-store'

const BOOTSTRAP_STORAGE_KEY = 'winsentials-preferences-bootstrap'

function writeBootstrapCache() {
  const state = usePreferencesStore.getState()

  if (!state.hasHydrated) {
    return
  }

  try {
    window.localStorage.setItem(BOOTSTRAP_STORAGE_KEY, JSON.stringify({
      state: {
        chromeAcrylic: state.chromeAcrylic,
        language: state.language,
        palette: state.palette,
        theme: state.theme,
        updateChecksEnabled: state.updateChecksEnabled,
      },
      version: 1,
    }))
  }
  catch (error) {
    console.warn('Failed to write theme bootstrap cache', error)
  }
}

let initialized = false

export function initPreferencesBootstrapCache() {
  if (initialized) {
    return
  }

  initialized = true
  writeBootstrapCache()

  usePreferencesStore.subscribe((state, previousState) => {
    if (
      state.hasHydrated === previousState.hasHydrated
      && state.chromeAcrylic === previousState.chromeAcrylic
      && state.language === previousState.language
      && state.palette === previousState.palette
      && state.theme === previousState.theme
      && state.updateChecksEnabled === previousState.updateChecksEnabled
    ) {
      return
    }

    writeBootstrapCache()
  })
}
