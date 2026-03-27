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
import { useRef } from 'react'
import { useMountEffect } from '@/shared/lib/hooks/use-mount-effect'
import { SidebarInset, SidebarProvider } from '@/shared/ui/sidebar'
import { SmoothScrollArea } from '@/shared/ui/smooth-scroll-area'
import { AppSidebar } from '@/widgets/sidebar/ui/app-sidebar'
import { AppTitlebar } from '@/widgets/titlebar/ui/app-titlebar'
import { usePageHeader } from './use-page-header'

function ScrollReset({ scrollAreaRef }: { scrollAreaRef: RefObject<SmoothScrollAreaHandle | null> }) {
  useMountEffect(() => {
    scrollAreaRef.current?.scrollToTop(true)
  })

  return null
}

function AppShellLayout({
  pathname,
  scrollAreaRef,
}: {
  pathname: string
  scrollAreaRef: RefObject<SmoothScrollAreaHandle | null>
}) {
  const pageHeader = usePageHeader(pathname)

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
              <header className="px-4 pt-4 pb-3 md:px-6 md:pt-4 md:pb-4">
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

  return (
    <>
      <ScrollReset key={pathname} scrollAreaRef={scrollAreaRef} />
      <AppShellLayout pathname={pathname} scrollAreaRef={scrollAreaRef} />
    </>
  )
}

function IndexRedirect() {
  return <Navigate to="/home" replace />
}

const rootRoute = createRootRoute({
  component: AppShell,
})

const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  component: IndexRedirect,
})

const homeRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'home',
  component: lazyRouteComponent(
    () => import('@/pages/home/ui/home-page'),
    'HomePage',
  ),
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

const cpuRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'cpu',
  component: lazyRouteComponent(
    () => import('@/pages/home/ui/cpu-page'),
    'CpuPage',
  ),
})

const ramRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'ram',
  component: lazyRouteComponent(
    () => import('@/pages/home/ui/ram-page'),
    'RamPage',
  ),
})

const gpuRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'gpu',
  component: lazyRouteComponent(
    () => import('@/pages/home/ui/gpu-page'),
    'GpuPage',
  ),
})

const gpuDetailRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'gpu/$gpuIndex',
  component: lazyRouteComponent(
    () => import('@/pages/home/ui/gpu-page'),
    'GpuPage',
  ),
})

const diskRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'storage/$disk',
  component: lazyRouteComponent(
    () => import('@/pages/home/ui/disk-detail-page'),
    'DiskDetailPage',
  ),
})

const networkStatsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'network-stats',
  component: lazyRouteComponent(
    () => import('@/pages/home/ui/network-stats-page'),
    'NetworkStatsPage',
  ),
})

const startupRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'startup',
  component: lazyRouteComponent(
    () => import('@/pages/startup/ui/startup-page'),
    'StartupPage',
  ),
})

const backupRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'backup',
  component: lazyRouteComponent(
    () => import('@/pages/backup/ui/backup-page'),
    'BackupPage',
  ),
})

const networkAdapterRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'network-stats/$adapterName',
  component: lazyRouteComponent(
    () => import('@/pages/home/ui/network-stats-page'),
    'NetworkStatsPage',
  ),
})

const routeTree = rootRoute.addChildren([
  indexRoute,
  homeRoute,
  behaviourRoute,
  appearanceRoute,
  securityRoute,
  networkRoute,
  startupRoute,
  backupRoute,
  settingsRoute,
  cpuRoute,
  ramRoute,
  gpuRoute,
  gpuDetailRoute,
  diskRoute,
  networkStatsRoute,
  networkAdapterRoute,
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
