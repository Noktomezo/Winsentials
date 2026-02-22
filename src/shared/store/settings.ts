import { create } from 'zustand'
import { persist } from 'zustand/middleware'

export type Theme = 'light' | 'dark' | 'system'
export type Language = 'en' | 'ru'

function getSystemLanguage(): Language {
  if (typeof navigator === 'undefined')
    return 'en'
  const lang = navigator.language.split('-')[0]
  return lang === 'ru' ? 'ru' : 'en'
}

interface SettingsState {
  theme: Theme
  language: Language
  setTheme: (theme: Theme) => void
  setLanguage: (language: Language) => void
}

export const useSettingsStore = create<SettingsState>()(
  persist(
    set => ({
      theme: 'system',
      language: getSystemLanguage(),
      setTheme: theme => set({ theme }),
      setLanguage: language => set({ language }),
    }),
    {
      name: 'winsentials-settings',
    },
  ),
)
