import {
  createRootRoute,
  createRoute,
  createRouter,
  lazyRouteComponent,
} from '@tanstack/react-router'
import { AppShell } from '@/app/ui/app-shell'
import { IndexRedirect } from '@/app/ui/index-redirect'

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
  ),
})

const appearanceRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'appearance',
  component: lazyRouteComponent(
    () => import('@/pages/appearance/ui/appearance-page'),
  ),
})

const behaviourRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'behaviour',
  component: lazyRouteComponent(
    () => import('@/pages/behaviour/ui/behaviour-page'),
  ),
})

const debloatRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'debloat',
  component: lazyRouteComponent(
    () => import('@/pages/debloat/ui/debloat-page'),
  ),
})

const contextMenuRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'context-menu',
  component: lazyRouteComponent(
    () => import('@/pages/context-menu/ui/context-menu-page'),
  ),
})

const settingsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'settings',
  component: lazyRouteComponent(
    () => import('@/pages/settings/ui/settings-page'),
  ),
})

const securityRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'security',
  component: lazyRouteComponent(
    () => import('@/pages/security/ui/security-page'),
  ),
})

const privacyRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'privacy',
  component: lazyRouteComponent(
    () => import('@/pages/privacy/ui/privacy-page'),
  ),
})

const networkRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'network',
  component: lazyRouteComponent(
    () => import('@/pages/network/ui/network-page'),
  ),
})

const performanceRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'performance',
  component: lazyRouteComponent(
    () => import('@/pages/performance/ui/performance-page'),
  ),
})

const memoryRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'memory',
  component: lazyRouteComponent(
    () => import('@/pages/memory/ui/memory-page'),
  ),
})

const inputRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'input',
  component: lazyRouteComponent(
    () => import('@/pages/input/ui/input-page'),
  ),
})

const toolsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'tools',
  component: lazyRouteComponent(
    () => import('@/pages/tools/ui/tools-page'),
  ),
})

const cleanupRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'cleanup',
  component: lazyRouteComponent(
    () => import('@/pages/cleanup/ui/cleanup-page'),
  ),
})

const cpuRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'cpu',
  component: lazyRouteComponent(
    () => import('@/pages/home/ui/cpu-page'),
  ),
})

const ramRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'ram',
  component: lazyRouteComponent(
    () => import('@/pages/home/ui/ram-page'),
  ),
})

const gpuRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'gpu',
  component: lazyRouteComponent(
    () => import('@/pages/home/ui/gpu-page'),
  ),
})

const gpuDetailRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'gpu/$gpuIndex',
  component: lazyRouteComponent(
    () => import('@/pages/home/ui/gpu-page'),
  ),
})

const diskRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'storage/$disk',
  component: lazyRouteComponent(
    () => import('@/pages/home/ui/disk-detail-page'),
  ),
})

const networkStatsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'network-stats',
  component: lazyRouteComponent(
    () => import('@/pages/home/ui/network-stats-page'),
  ),
})

const startupRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'startup',
  component: lazyRouteComponent(
    () => import('@/pages/startup/ui/startup-page'),
  ),
})

const backupRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'backup',
  component: lazyRouteComponent(
    () => import('@/pages/backup/ui/backup-page'),
  ),
})

const networkAdapterRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: 'network-stats/$adapterName',
  component: lazyRouteComponent(
    () => import('@/pages/home/ui/network-stats-page'),
  ),
})

const routeTree = rootRoute.addChildren([
  indexRoute,
  homeRoute,
  behaviourRoute,
  contextMenuRoute,
  debloatRoute,
  appearanceRoute,
  securityRoute,
  privacyRoute,
  networkRoute,
  performanceRoute,
  memoryRoute,
  inputRoute,
  toolsRoute,
  cleanupRoute,
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
