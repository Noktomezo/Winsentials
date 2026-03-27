import type { ResolvedTheme } from '@/shared/config/app'
import { useSyncExternalStore } from 'react'
import { usePreferencesStore } from '@/entities/settings/model/preferences-store'
import { getSystemResolvedTheme, subscribeSystemTheme } from '@/shared/lib/system-theme'

export function useResolvedTheme() {
  const theme = usePreferencesStore(state => state.theme)
  const systemResolvedTheme = useSyncExternalStore<ResolvedTheme>(
    subscribeSystemTheme,
    getSystemResolvedTheme,
    () => 'dark',
  )

  return theme === 'system'
    ? systemResolvedTheme
    : theme
}
