import type { ResolvedTheme } from '@/shared/config/app'
import { useSyncExternalStore } from 'react'
import { usePreferencesStore } from '@/entities/settings/model/preferences-store'
import {
  getSystemResolvedTheme,
  subscribeToSystemThemeChange,
} from '@/shared/lib/theme/resolve-theme'

function subscribeNever() {
  return () => {}
}

export function useResolvedTheme() {
  const theme = usePreferencesStore(state => state.theme)
  const systemTheme = useSyncExternalStore(
    theme === 'system' ? subscribeToSystemThemeChange : subscribeNever,
    getSystemResolvedTheme,
    getSystemResolvedTheme,
  )

  return (theme === 'system' ? systemTheme : theme) as ResolvedTheme
}
