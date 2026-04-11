import type { ResolvedTheme } from '@/shared/config/app'
import { useSyncExternalStore } from 'react'
import { usePreferencesStore } from '@/entities/settings/model/preferences-store'
import {
  getSystemResolvedTheme,
  subscribeToSystemThemeChange,
} from '@/shared/lib/theme/resolve-theme'

export function useResolvedTheme() {
  const theme = usePreferencesStore(state => state.theme)
  const systemTheme = useSyncExternalStore(
    subscribeToSystemThemeChange,
    getSystemResolvedTheme,
    getSystemResolvedTheme,
  )

  return (theme === 'system' ? systemTheme : theme) as ResolvedTheme
}
