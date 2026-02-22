import { useEffect } from 'react'
import { useSettingsStore } from '@/shared/store/settings'

function getSystemTheme(): 'light' | 'dark' {
  if (typeof window === 'undefined')
    return 'light'
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
}

export function useTheme() {
  const theme = useSettingsStore(state => state.theme)

  useEffect(() => {
    const root = document.documentElement

    function applyTheme(t: 'light' | 'dark') {
      if (t === 'dark') {
        root.classList.add('dark')
      }
      else {
        root.classList.remove('dark')
      }
    }

    if (theme === 'system') {
      applyTheme(getSystemTheme())

      const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
      const handleChange = (e: MediaQueryListEvent) => {
        applyTheme(e.matches ? 'dark' : 'light')
      }

      mediaQuery.addEventListener('change', handleChange)
      return () => mediaQuery.removeEventListener('change', handleChange)
    }
    else {
      applyTheme(theme)
    }
  }, [theme])
}
