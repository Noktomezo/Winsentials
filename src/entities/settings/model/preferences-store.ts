import type { AppLanguagePreference, AppPalette, AppTheme } from '@/shared/config/app'
import { create } from 'zustand'
import { createJSONStorage, persist } from 'zustand/middleware'
import { tauriStateStorage } from '@/entities/settings/lib/tauri-storage'
import {
  DEFAULT_LANGUAGE,
  DEFAULT_PALETTE,
  DEFAULT_THEME,
} from '@/shared/config/app'

interface PersistedPreferencesState {
  chromeAcrylic?: boolean
  language?: AppLanguagePreference
  palette?: AppPalette
  sidebarAcrylic?: boolean
  theme?: AppTheme | 'acrylic'
  updateChecksEnabled?: boolean
}

interface PreferencesState {
  chromeAcrylic: boolean
  hasHydrated: boolean
  language: AppLanguagePreference
  palette: AppPalette
  updateChecksEnabled: boolean
  setChromeAcrylic: (enabled: boolean) => void
  setHasHydrated: (hasHydrated: boolean) => void
  setPalette: (palette: AppPalette) => void
  theme: AppTheme
  setLanguage: (language: AppLanguagePreference) => void
  setTheme: (theme: AppTheme) => void
  setUpdateChecksEnabled: (enabled: boolean) => void
}

export type { PreferencesState }

export const usePreferencesStore = create<PreferencesState>()(
  persist(
    set => ({
      chromeAcrylic: false,
      hasHydrated: false,
      language: DEFAULT_LANGUAGE,
      palette: DEFAULT_PALETTE,
      updateChecksEnabled: true,
      setChromeAcrylic: chromeAcrylic => set({ chromeAcrylic }),
      setHasHydrated: hasHydrated => set({ hasHydrated }),
      setPalette: palette => set({ palette }),
      setUpdateChecksEnabled: updateChecksEnabled => set({ updateChecksEnabled }),
      theme: DEFAULT_THEME,
      setLanguage: language => set({ language }),
      setTheme: theme => set({ theme }),
    }),
    {
      migrate: (persistedState: unknown) => {
        const state = (persistedState ?? {}) as PersistedPreferencesState
        const legacyAcrylic = state.theme === 'acrylic'

        return {
          chromeAcrylic:
            legacyAcrylic
            || state.chromeAcrylic === true
            || state.sidebarAcrylic === true,
          language: state.language ?? DEFAULT_LANGUAGE,
          palette: state.palette ?? DEFAULT_PALETTE,
          theme: legacyAcrylic ? 'dark' : (state.theme ?? DEFAULT_THEME),
          updateChecksEnabled: state.updateChecksEnabled ?? true,
        }
      },
      name: 'winsentials-preferences',
      onRehydrateStorage: () => state => state?.setHasHydrated(true),
      storage: createJSONStorage(() => tauriStateStorage),
      version: 5,
    },
  ),
)
