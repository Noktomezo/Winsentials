import type { AppTheme, ResolvedTheme } from '@/shared/config/app'

const SYSTEM_THEME_MEDIA_QUERY = '(prefers-color-scheme: dark)'

function getSystemThemeMediaQueryList() {
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') {
    return null
  }

  return window.matchMedia(SYSTEM_THEME_MEDIA_QUERY)
}

export function getSystemResolvedTheme(): ResolvedTheme {
  return getSystemThemeMediaQueryList()?.matches ? 'dark' : 'light'
}

export function resolveThemePreference(theme: AppTheme): ResolvedTheme {
  return theme === 'system' ? getSystemResolvedTheme() : theme
}

export function subscribeToSystemThemeChange(callback: () => void) {
  const mediaQueryList = getSystemThemeMediaQueryList()

  if (!mediaQueryList) {
    return () => {}
  }

  const listener = callback
  mediaQueryList.addEventListener('change', listener)

  return () => {
    mediaQueryList.removeEventListener('change', listener)
  }
}
