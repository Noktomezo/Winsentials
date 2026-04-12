import type { MouseEvent } from 'react'
import { useNavigate, useRouter, useRouterState } from '@tanstack/react-router'
import { EyeOff, FolderCog, Gauge, House, Keyboard, MemoryStick, Network, Palette, Settings2, Shield, Wrench } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { useRouteIntentPreload } from '@/shared/lib/hooks/use-route-intent-preload'
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from '@/shared/ui'

type SidebarRoute = '/home' | '/appearance' | '/backup' | '/behaviour' | '/security' | '/privacy' | '/network' | '/performance' | '/memory' | '/input' | '/startup' | '/tools' | '/settings'

export function AppSidebar() {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const router = useRouter()
  const preloadRouteIntent = useRouteIntentPreload()
  const pathname = useRouterState({
    select: state => state.location.pathname,
  })

  function handleNavigate(to: SidebarRoute) {
    if (pathname === to) {
      return
    }

    void navigate({ to })
  }

  function handlePointerIntent(to: SidebarRoute) {
    preloadRouteIntent(() => router.preloadRoute({ to }))
  }

  function handleMenuClick(
    event: MouseEvent<HTMLButtonElement>,
    to: SidebarRoute,
  ) {
    event.preventDefault()
    handleNavigate(to)
  }

  const isHomeRoute = pathname === '/home'
    || pathname === '/cpu'
    || pathname === '/ram'
    || pathname.startsWith('/gpu')
    || pathname.startsWith('/storage')
    || pathname.startsWith('/network-stats')
  const isToolsRoute = pathname === '/tools' || pathname === '/startup' || pathname === '/backup'

  return (
    <Sidebar
      className="h-full min-h-0 shrink-0"
      collapsible="icon"
      style={
        {
          '--sidebar-width': '13.44rem',
          '--sidebar-width-icon': 'calc(var(--spacing) * 12)',
        } as React.CSSProperties
      }
    >
      <SidebarHeader className="border-b border-sidebar-border/70 p-2">
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton
              className="cursor-pointer"
              isActive={isHomeRoute}
              onClick={event => handleMenuClick(event, '/home')}
              onFocus={() => handlePointerIntent('/home')}
              onMouseEnter={() => handlePointerIntent('/home')}
              tooltip={t('navigation.home')}
              type="button"
            >
              <House />
              <span data-sidebar-label>{t('navigation.home')}</span>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarHeader>
      <SidebarContent className="p-2">
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton
              className="cursor-pointer"
              isActive={pathname === '/security'}
              onClick={event => handleMenuClick(event, '/security')}
              onFocus={() => handlePointerIntent('/security')}
              onMouseEnter={() => handlePointerIntent('/security')}
              tooltip={t('navigation.security')}
              type="button"
            >
              <Shield />
              <span data-sidebar-label>{t('navigation.security')}</span>
            </SidebarMenuButton>
          </SidebarMenuItem>
          <SidebarMenuItem>
            <SidebarMenuButton
              className="cursor-pointer"
              isActive={pathname === '/privacy'}
              onClick={event => handleMenuClick(event, '/privacy')}
              onFocus={() => handlePointerIntent('/privacy')}
              onMouseEnter={() => handlePointerIntent('/privacy')}
              tooltip={t('navigation.privacy')}
              type="button"
            >
              <EyeOff />
              <span data-sidebar-label>{t('navigation.privacy')}</span>
            </SidebarMenuButton>
          </SidebarMenuItem>
          <SidebarMenuItem>
            <SidebarMenuButton
              className="cursor-pointer"
              isActive={pathname === '/behaviour'}
              onClick={event => handleMenuClick(event, '/behaviour')}
              onFocus={() => handlePointerIntent('/behaviour')}
              onMouseEnter={() => handlePointerIntent('/behaviour')}
              tooltip={t('navigation.behaviour')}
              type="button"
            >
              <FolderCog />
              <span data-sidebar-label>{t('navigation.behaviour')}</span>
            </SidebarMenuButton>
          </SidebarMenuItem>
          <SidebarMenuItem>
            <SidebarMenuButton
              className="cursor-pointer"
              isActive={pathname === '/appearance'}
              onClick={event => handleMenuClick(event, '/appearance')}
              onFocus={() => handlePointerIntent('/appearance')}
              onMouseEnter={() => handlePointerIntent('/appearance')}
              tooltip={t('navigation.appearance')}
              type="button"
            >
              <Palette />
              <span data-sidebar-label>{t('navigation.appearance')}</span>
            </SidebarMenuButton>
          </SidebarMenuItem>
          <SidebarMenuItem>
            <SidebarMenuButton
              className="cursor-pointer"
              isActive={pathname === '/performance'}
              onClick={event => handleMenuClick(event, '/performance')}
              onFocus={() => handlePointerIntent('/performance')}
              onMouseEnter={() => handlePointerIntent('/performance')}
              tooltip={t('navigation.performance')}
              type="button"
            >
              <Gauge />
              <span data-sidebar-label>{t('navigation.performance')}</span>
            </SidebarMenuButton>
          </SidebarMenuItem>
          <SidebarMenuItem>
            <SidebarMenuButton
              className="cursor-pointer"
              isActive={pathname === '/memory'}
              onClick={event => handleMenuClick(event, '/memory')}
              onFocus={() => handlePointerIntent('/memory')}
              onMouseEnter={() => handlePointerIntent('/memory')}
              tooltip={t('navigation.memory')}
              type="button"
            >
              <MemoryStick />
              <span data-sidebar-label>{t('navigation.memory')}</span>
            </SidebarMenuButton>
          </SidebarMenuItem>
          <SidebarMenuItem>
            <SidebarMenuButton
              className="cursor-pointer"
              isActive={pathname === '/input'}
              onClick={event => handleMenuClick(event, '/input')}
              onFocus={() => handlePointerIntent('/input')}
              onMouseEnter={() => handlePointerIntent('/input')}
              tooltip={t('navigation.input')}
              type="button"
            >
              <Keyboard />
              <span data-sidebar-label>{t('navigation.input')}</span>
            </SidebarMenuButton>
          </SidebarMenuItem>
          <SidebarMenuItem>
            <SidebarMenuButton
              className="cursor-pointer"
              isActive={pathname === '/network'}
              onClick={event => handleMenuClick(event, '/network')}
              onFocus={() => handlePointerIntent('/network')}
              onMouseEnter={() => handlePointerIntent('/network')}
              tooltip={t('navigation.network')}
              type="button"
            >
              <Network />
              <span data-sidebar-label>{t('navigation.network')}</span>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarContent>
      <SidebarFooter className="border-t border-sidebar-border/70 p-2">
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton
              className="cursor-pointer"
              isActive={isToolsRoute}
              onClick={event => handleMenuClick(event, '/tools')}
              onFocus={() => handlePointerIntent('/tools')}
              onMouseEnter={() => handlePointerIntent('/tools')}
              tooltip={t('navigation.tools')}
              type="button"
            >
              <Wrench />
              <span data-sidebar-label>{t('navigation.tools')}</span>
            </SidebarMenuButton>
          </SidebarMenuItem>
          <SidebarMenuItem>
            <SidebarMenuButton
              className="cursor-pointer"
              isActive={pathname === '/settings'}
              onClick={event => handleMenuClick(event, '/settings')}
              onFocus={() => handlePointerIntent('/settings')}
              onMouseEnter={() => handlePointerIntent('/settings')}
              tooltip={t('navigation.settings')}
              type="button"
            >
              <Settings2 />
              <span data-sidebar-label>{t('navigation.settings')}</span>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarFooter>
    </Sidebar>
  )
}
