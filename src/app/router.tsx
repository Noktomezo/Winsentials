import type { RefObject } from 'react'
import type { SmoothScrollAreaHandle } from '@/shared/ui/smooth-scroll-area'
import {
  createRootRoute,
  createRoute,
  createRouter,
  lazyRouteComponent,
  Navigate,
  Outlet,
  useRouterState,
} from '@tanstack/react-router'
import { useEffect, useRef } from 'react'
import { useTranslation } from 'react-i18next'
import { SidebarInset, SidebarProvider } from '@/shared/ui/sidebar'
import { SmoothScrollArea } from '@/shared/ui/smooth-scroll-area'
import { AppSidebar } from '@/widgets/sidebar/ui/app-sidebar'
import { AppTitlebar } from '@/widgets/titlebar/ui/app-titlebar'

function AppShellLayout({
  pathname,
  scrollAreaRef,
}: {
  pathname: string
  scrollAreaRef: RefObject<SmoothScrollAreaHandle | null>
}) {
  const { t } = useTranslation()
  const pageHeader = {
    '/behaviour': {
      description: t('behaviour.description'),
      title: t('behaviour.title'),
    },
    '/appearance': {
      description: t('appearance.description'),
      title: t('appearance.title'),
    },
    '/security': {
      description: t('security.description'),
      title: t('security.title'),
    },
    '/network': {
      description: t('network.description'),
      title: t('network.title'),
    },
    '/settings': {
      description: t('settings.description'),
      title: t('settings.title'),
    },
  }[pathname] ?? {
    description: '',
    title: t('app.title'),
  }

  return (
    <SidebarProvider
      className="h-svh min-h-svh flex-col overflow-hidden"
      defaultOpen={true}
    >
      <AppTitlebar />
      <div className="flex min-h-0 flex-1 overflow-hidden bg-sidebar">
        <AppSidebar />
        <SidebarInset className="min-h-0 overflow-hidden rounded-tl-[8px] border-t border-l border-border/70 bg-background">
          <SmoothScrollArea className="h-full" ref={scrollAreaRef}>
            <div key={pathname} className="page-shell-transition flex min-h-full flex-col">
              <header className="p-4 md:p-6">
                <div className="space-y-0.5">
                  <h1 className="text-xl font-semibold tracking-tight text-foreground">
                    {pageHeader.title}
                  </h1>
                  {pageHeader.description && (
                    <p className="text-xs leading-5 text-muted-foreground">
                      {pageHeader.description}
                    </p>
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

function AppShell() {
  const pathname = useRouterState({
    select: state => state.location.pathname,
  })
  const scrollAreaRef = useRef<SmoothScrollAreaHandle>(null)

  useEffect(() => {
    scrollAreaRef.current?.scrollToTop(true)
  }, [pathname])

  return <AppShellLayout pathname={pathname} scrollAreaRef={scrollAreaRef} />
}

function IndexRedirect() {
  return <Navigate to="/appearance" replace />
}

const rootRoute = createRootRoute({
  component: AppShell,
})

const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  component: IndexRedirect,
})

const appearanceRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'appearance',
  component: lazyRouteComponent(
    () => import('@/pages/appearance/ui/appearance-page'),
    'AppearancePage',
  ),
})

const behaviourRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'behaviour',
  component: lazyRouteComponent(
    () => import('@/pages/behaviour/ui/behaviour-page'),
    'BehaviourPage',
  ),
})

const settingsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'settings',
  component: lazyRouteComponent(
    () => import('@/pages/settings/ui/settings-page'),
    'SettingsPage',
  ),
})

const securityRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'security',
  component: lazyRouteComponent(
    () => import('@/pages/security/ui/security-page'),
    'SecurityPage',
  ),
})

const networkRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'network',
  component: lazyRouteComponent(
    () => import('@/pages/network/ui/network-page'),
    'NetworkPage',
  ),
})

const routeTree = rootRoute.addChildren([
  indexRoute,
  behaviourRoute,
  appearanceRoute,
  securityRoute,
  networkRoute,
  settingsRoute,
])

export const router = createRouter({
  routeTree,
  defaultPreload: 'intent',
})

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router
  }
}
