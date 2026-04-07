import type { ResolvedTheme } from '@/shared/config/app'
import { usePreferencesStore } from '@/entities/settings/model/preferences-store'

export function useResolvedTheme() {
  return usePreferencesStore(state => state.theme) as ResolvedTheme
}
