import type { MouseEvent } from 'react'
import { useNavigate, useRouter, useRouterState } from '@tanstack/react-router'
import { FolderCog, Network, Palette, Settings2, Shield } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from '@/shared/ui/sidebar'

type SidebarRoute = '/appearance' | '/behaviour' | '/security' | '/network' | '/settings'

export function AppSidebar() {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const router = useRouter()
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
    router.preloadRoute({ to }).catch((error) => {
      console.warn('Failed to preload route', error)
    })
  }

  function handleMenuClick(
    event: MouseEvent<HTMLButtonElement>,
    to: SidebarRoute,
  ) {
    event.preventDefault()
    handleNavigate(to)
  }

  return (
    <Sidebar
      className="h-full min-h-0 shrink-0 [&>[data-slot=sidebar-inner]]:bg-transparent"
      collapsible="icon"
      style={
        {
          '--sidebar-width': '12rem',
          '--sidebar-width-icon': 'calc(var(--spacing) * 12)',
        } as React.CSSProperties
      }
    >
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
              <span>{t('navigation.security')}</span>
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
              <span>{t('navigation.behaviour')}</span>
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
              <span>{t('navigation.appearance')}</span>
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
              <span>{t('navigation.network')}</span>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarContent>
      <SidebarFooter className="border-t border-sidebar-border/70 p-2">
        <SidebarMenu>
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
              <span>{t('navigation.settings')}</span>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarFooter>
    </Sidebar>
  )
}
