import type { MouseEvent } from 'react'
import { useNavigate, useRouter, useRouterState } from '@tanstack/react-router'
import { ArchiveRestore, ChevronDown, FolderCog, House, Network, Palette, Rocket, Settings2, Shield, Wrench } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useRouteIntentPreload } from '@/shared/lib/hooks/use-route-intent-preload'
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  useSidebar,
} from '@/shared/ui'

type SidebarRoute = '/home' | '/appearance' | '/backup' | '/behaviour' | '/security' | '/network' | '/startup' | '/settings'

export function AppSidebar() {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const router = useRouter()
  const preloadRouteIntent = useRouteIntentPreload()
  const [isToolsOpen, setIsToolsOpen] = useState(false)
  const { state: sidebarState } = useSidebar()
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

  function handleToolsPointerIntent() {
    handlePointerIntent('/startup')
    handlePointerIntent('/backup')
  }

  const isHomeRoute = pathname === '/home'
    || pathname === '/cpu'
    || pathname === '/ram'
    || pathname.startsWith('/gpu')
    || pathname.startsWith('/storage')
    || pathname.startsWith('/network-stats')
  const isToolsRoute = pathname === '/startup' || pathname === '/backup'

  return (
    <Sidebar
      className="h-full min-h-0 shrink-0 [&>[data-slot=sidebar-inner]]:bg-transparent"
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
              <span>{t('navigation.home')}</span>
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
            <DropdownMenu onOpenChange={setIsToolsOpen} open={isToolsOpen}>
              <DropdownMenuTrigger asChild>
                <SidebarMenuButton
                  className="cursor-pointer"
                  isActive={isToolsRoute}
                  onFocus={handleToolsPointerIntent}
                  onMouseEnter={handleToolsPointerIntent}
                  tooltip={t('navigation.tools')}
                  type="button"
                >
                  <Wrench />
                  <span>{t('navigation.tools')}</span>
                  <ChevronDown className={`ml-auto size-4 opacity-70 transition-transform duration-200 hidden group-data-[state=expanded]:block ${isToolsOpen ? 'rotate-180' : 'rotate-0'}`} />
                </SidebarMenuButton>
              </DropdownMenuTrigger>
              <DropdownMenuContent
                align={sidebarState === 'collapsed' ? 'center' : 'start'}
                className="min-w-0"
                side={sidebarState === 'collapsed' ? 'right' : 'top'}
                sideOffset={6}
                style={sidebarState === 'collapsed'
                  ? undefined
                  : { width: 'var(--radix-dropdown-menu-trigger-width)' }}
              >
                <DropdownMenuItem
                  onFocus={() => handlePointerIntent('/startup')}
                  onMouseEnter={() => handlePointerIntent('/startup')}
                  onSelect={() => handleNavigate('/startup')}
                >
                  <Rocket className="size-4" />
                  {t('navigation.startup')}
                </DropdownMenuItem>
                <DropdownMenuItem
                  onFocus={() => handlePointerIntent('/backup')}
                  onMouseEnter={() => handlePointerIntent('/backup')}
                  onSelect={() => handleNavigate('/backup')}
                >
                  <ArchiveRestore className="size-4" />
                  {t('navigation.backup')}
                </DropdownMenuItem>
              </DropdownMenuContent>
            </DropdownMenu>
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
              <span>{t('navigation.settings')}</span>
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarFooter>
    </Sidebar>
  )
}
