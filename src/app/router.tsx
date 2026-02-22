import { createRootRoute, createRoute, createRouter, Outlet } from '@tanstack/react-router'
import { Layout } from '@/app/Layout'
import { AppearancePage } from '@/pages/appearance'
import { HardwarePage } from '@/pages/hardware'
import { HomePage } from '@/pages/home'
import { InputPage } from '@/pages/input'
import { MemoryPage } from '@/pages/memory'
import { NetworkPage } from '@/pages/network'
import { PrivacyPage } from '@/pages/privacy'
import { SecurityPage } from '@/pages/security'
import { SettingsPage } from '@/pages/settings'
import { SystemPage } from '@/pages/system'

const rootRoute = createRootRoute({
  component: () => <Layout><Outlet /></Layout>,
})

const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  component: HomePage,
})

const systemRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/system',
  component: SystemPage,
})

const appearanceRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/appearance',
  component: AppearancePage,
})

const privacyRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/privacy',
  component: PrivacyPage,
})

const networkRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/network',
  component: NetworkPage,
})

const inputRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/input',
  component: InputPage,
})

const securityRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/security',
  component: SecurityPage,
})

const hardwareRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/hardware',
  component: HardwarePage,
})

const memoryRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/memory',
  component: MemoryPage,
})

const settingsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/settings',
  component: SettingsPage,
})

const routeTree = rootRoute.addChildren([
  indexRoute,
  systemRoute,
  appearanceRoute,
  privacyRoute,
  networkRoute,
  inputRoute,
  securityRoute,
  hardwareRoute,
  memoryRoute,
  settingsRoute,
])

export const router = createRouter({ routeTree })

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router
  }
}
