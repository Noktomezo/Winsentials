import type { RefObject } from 'react'
import type { SmoothScrollAreaHandle } from '@/shared/ui/smooth-scroll-area'
import { Outlet } from '@tanstack/react-router'
import { useEffect, useRef } from 'react'
import { usePageHeader } from '@/app/use-page-header'
import { usePreferencesStore } from '@/entities/settings/model/preferences-store'
import { setDiscordPresenceActivity } from '@/features/discord-presence/api'
import { SidebarInset, SidebarProvider } from '@/shared/ui/sidebar'
import { SmoothScrollArea } from '@/shared/ui/smooth-scroll-area'
import { AppSidebar } from '@/widgets/sidebar/ui/app-sidebar'
import { AppTitlebar } from '@/widgets/titlebar/ui/app-titlebar'

export function AppShellLayout({
  pathname,
  scrollAreaRef,
}: {
  pathname: string
  scrollAreaRef: RefObject<SmoothScrollAreaHandle | null>
}) {
  const pageHeader = usePageHeader(pathname)
  const hasHydrated = usePreferencesStore(state => state.hasHydrated)
  const discordPresenceMode = usePreferencesStore(state => state.discordPresenceMode)
  const latestPresenceRequestId = useRef(0)
  const discordPresencePageLabel = typeof pageHeader.title === 'string'
    ? pageHeader.title
    : undefined

  useEffect(() => {
    if (!hasHydrated) {
      return
    }

    const requestId = latestPresenceRequestId.current + 1
    latestPresenceRequestId.current = requestId

    void setDiscordPresenceActivity({
      mode: discordPresenceMode,
      pageLabel: discordPresencePageLabel,
    }).catch((error) => {
      if (latestPresenceRequestId.current !== requestId) {
        return
      }
      console.warn('Failed to sync Discord Rich Presence', error)
    })

    return () => {
      if (latestPresenceRequestId.current === requestId) {
        latestPresenceRequestId.current += 1
      }
    }
  }, [discordPresenceMode, discordPresencePageLabel, hasHydrated])

  return (
    <SidebarProvider
      className="app-shell h-svh min-h-svh flex-col overflow-hidden"
      defaultOpen={false}
    >
      <AppTitlebar />
      <div className="flex min-h-0 flex-1 overflow-hidden bg-transparent">
        <AppSidebar />
        <SidebarInset className="min-h-0 overflow-hidden rounded-tl-[8px] border-t border-l border-border/70 bg-background">
          <SmoothScrollArea className="h-full" ref={scrollAreaRef}>
            <div key={pathname} className="page-shell-transition flex min-h-full flex-col">
              <header className="px-4 pt-4 pb-3 md:px-6 md:pt-4 md:pb-4">
                <div className="flex items-center justify-between gap-4">
                  <div className="min-w-0 space-y-0.5">
                    <h1 className="text-xl font-semibold tracking-tight text-foreground">
                      {pageHeader.title}
                    </h1>
                    {pageHeader.description && (
                      <p className="text-xs leading-5 text-muted-foreground">
                        {pageHeader.description}
                      </p>
                    )}
                  </div>
                  {pageHeader.actions && (
                    <div className="flex shrink-0 items-center gap-2">
                      {pageHeader.actions}
                    </div>
                  )}
                </div>
              </header>
              <Outlet />
            </div>
          </SmoothScrollArea>
        </SidebarInset>
      </div>
    </SidebarProvider>
  )
}
