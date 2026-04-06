import type {
  AppLanguagePreference,
  AppPalette,
  AppTheme,
  AppWebviewMaterial,
} from '@/shared/config/app'
import { create } from 'zustand'
import { createJSONStorage, persist } from 'zustand/middleware'
import { tauriStateStorage } from '@/entities/settings/lib/tauri-storage'
import {
  DEFAULT_LANGUAGE,
  DEFAULT_PALETTE,
  DEFAULT_THEME,
  DEFAULT_WEBVIEW_MATERIAL,
} from '@/shared/config/app'

interface PersistedPreferencesState {
  chromeMaterial?: AppWebviewMaterial | 'blur' | 'micaAlt'
  webviewMaterial?: AppWebviewMaterial | 'blur' | 'micaAlt'
  chromeAcrylic?: boolean
  language?: AppLanguagePreference
  palette?: AppPalette
  sidebarAcrylic?: boolean
  theme?: AppTheme | 'acrylic'
  updateChecksEnabled?: boolean
}

interface PreferencesState {
  webviewMaterial: AppWebviewMaterial
  hasHydrated: boolean
  language: AppLanguagePreference
  palette: AppPalette
  updateChecksEnabled: boolean
  setWebviewMaterial: (material: AppWebviewMaterial) => void
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
      webviewMaterial: DEFAULT_WEBVIEW_MATERIAL,
      hasHydrated: false,
      language: DEFAULT_LANGUAGE,
      palette: DEFAULT_PALETTE,
      updateChecksEnabled: true,
      setWebviewMaterial: webviewMaterial => set({ webviewMaterial }),
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
        const rawLegacyMaterial = state.webviewMaterial ?? state.chromeMaterial
        const normalizedWebviewMaterial = rawLegacyMaterial === 'micaAlt'
          ? 'tabbed'
          : rawLegacyMaterial === 'blur'
            ? 'none'
            : rawLegacyMaterial
        const webviewMaterial = normalizedWebviewMaterial
          ?? (legacyAcrylic || state.chromeAcrylic === true || state.sidebarAcrylic === true
            ? 'acrylic'
            : DEFAULT_WEBVIEW_MATERIAL)

        return {
          webviewMaterial,
          language: state.language ?? DEFAULT_LANGUAGE,
          palette: state.palette ?? DEFAULT_PALETTE,
          theme: legacyAcrylic ? 'dark' : (state.theme ?? DEFAULT_THEME),
          updateChecksEnabled: state.updateChecksEnabled ?? true,
        }
      },
      name: 'winsentials-preferences',
      onRehydrateStorage: () => state => state?.setHasHydrated(true),
      storage: createJSONStorage(() => tauriStateStorage),
      version: 6,
    },
  ),
)
