import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { useTheme } from '@/shared/hooks/useTheme'
import { useSettingsStore } from '@/shared/store/settings'

export function SettingsProvider({ children }: { children: React.ReactNode }) {
  const { i18n } = useTranslation()
  const language = useSettingsStore(state => state.language)

  useTheme()

  useEffect(() => {
    i18n.changeLanguage(language)
  }, [language, i18n])

  return <>{children}</>
}
