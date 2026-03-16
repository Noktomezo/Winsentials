import type { PropsWithChildren } from 'react'
import type { ResolvedTheme } from '@/shared/config/app'
import { useEffect } from 'react'
import { usePreferencesStore } from '@/entities/settings/model/preferences-store'
import { AppUpdaterEffect } from '@/features/app-updater/ui/app-updater-effect'
import i18n from '@/shared/i18n'
import { syncChromeAcrylic } from '@/shared/lib/desktop/window-effects'
import { useResolvedTheme } from '@/shared/lib/hooks/use-resolved-theme'
import { toast } from '@/shared/lib/toast'
import { AppToastProvider } from '@/shared/lib/toast/toast-provider'
import { TooltipProvider } from '@/shared/ui/tooltip'

function AppPreferencesEffect({ resolvedTheme }: { resolvedTheme: ResolvedTheme }) {
  const chromeAcrylic = usePreferencesStore(state => state.chromeAcrylic)
  const hasHydrated = usePreferencesStore(state => state.hasHydrated)
  const language = usePreferencesStore(state => state.language)
  const palette = usePreferencesStore(state => state.palette)
  const setChromeAcrylic = usePreferencesStore(state => state.setChromeAcrylic)

  useEffect(() => {
    if (!hasHydrated) {
      return
    }

    document.documentElement.dataset.theme = resolvedTheme
    document.documentElement.dataset.palette = palette
    document.documentElement.dataset.chromeMaterial = chromeAcrylic
      ? 'acrylic'
      : 'solid'
  }, [chromeAcrylic, hasHydrated, palette, resolvedTheme])

  useEffect(() => {
    if (!hasHydrated) {
      return
    }

    let cancelled = false

    const applyChromeAcrylic = async () => {
      try {
        const applied = await syncChromeAcrylic({
          enabled: chromeAcrylic,
          theme: resolvedTheme,
        })

        if (!cancelled && chromeAcrylic && !applied) {
          toast.error(i18n.t('settings.acrylicUnavailable'))
          setChromeAcrylic(false)
        }
      }
      catch {
        if (!cancelled && chromeAcrylic) {
          toast.error(i18n.t('settings.acrylicUnavailable'))
          setChromeAcrylic(false)
        }
      }
    }

    void applyChromeAcrylic()

    return () => {
      cancelled = true
    }
  }, [chromeAcrylic, hasHydrated, resolvedTheme, setChromeAcrylic])

  useEffect(() => {
    if (!hasHydrated) {
      return
    }

    if (i18n.language !== language) {
      void i18n.changeLanguage(language)
    }
  }, [hasHydrated, language])

  return null
}

export function AppProviders({ children }: PropsWithChildren) {
  const resolvedTheme = useResolvedTheme()

  return (
    <TooltipProvider delayDuration={150}>
      <AppPreferencesEffect resolvedTheme={resolvedTheme} />
      <AppUpdaterEffect />
      {children}
      <AppToastProvider resolvedTheme={resolvedTheme} />
    </TooltipProvider>
  )
}
