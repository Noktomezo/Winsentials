import type { ResolvedTheme } from '@/shared/config/app'

export function getSystemResolvedTheme(): ResolvedTheme {
  if (typeof window === 'undefined') {
    return 'dark'
  }

  return window.matchMedia('(prefers-color-scheme: dark)').matches
    ? 'dark'
    : 'light'
}

export function subscribeSystemTheme(onStoreChange: () => void) {
  if (typeof window === 'undefined') {
    return () => {}
  }

  const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
  mediaQuery.addEventListener('change', onStoreChange)

  return () => {
    mediaQuery.removeEventListener('change', onStoreChange)
  }
}
