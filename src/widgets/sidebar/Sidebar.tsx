import type { LucideIcon } from 'lucide-react'
import { Link, useRouterState } from '@tanstack/react-router'
import {
  Cpu,
  Globe,
  Home,
  Keyboard,
  MemoryStick,
  Palette,
  PanelLeftClose,
  PanelLeftOpen,
  Rocket,
  Settings,
  Shield,
  ShieldCheck,
  Zap,
} from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { cn } from '@/shared/lib/utils'
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/shared/ui/tooltip'

const categories = [
  { id: 'home', icon: Home, path: '/' },
  { id: 'system', icon: Zap, path: '/system' },
  { id: 'appearance', icon: Palette, path: '/appearance' },
  { id: 'privacy', icon: Shield, path: '/privacy' },
  { id: 'network', icon: Globe, path: '/network' },
  { id: 'input', icon: Keyboard, path: '/input' },
  { id: 'security', icon: ShieldCheck, path: '/security' },
  { id: 'hardware', icon: Cpu, path: '/hardware' },
  { id: 'memory', icon: MemoryStick, path: '/memory' },
]

interface SidebarLinkProps {
  to: string
  icon: LucideIcon
  label: string
  collapsed: boolean
  isActive: boolean
}

function SidebarLink({
  to,
  icon: Icon,
  label,
  collapsed,
  isActive,
}: SidebarLinkProps) {
  const link = (
    <Link
      to={to}
      className={cn(
        'flex w-full items-center rounded-lg py-2 text-sm transition-all duration-300',
        collapsed ? 'gap-0 px-2.5' : 'gap-3 px-2.5',
        isActive
          ? 'bg-primary text-primary-foreground'
          : 'text-muted-foreground hover:bg-accent hover:text-foreground',
      )}
    >
      <Icon className="h-5 w-5 shrink-0" />
      <span
        className={cn(
          'truncate whitespace-nowrap transition-all duration-300 ease-in-out',
          collapsed ? 'w-0 opacity-0' : 'w-32 opacity-100',
        )}
      >
        {label}
      </span>
    </Link>
  )

  if (!collapsed)
    return link

  return (
    <Tooltip>
      <TooltipTrigger asChild>{link}</TooltipTrigger>
      <TooltipContent side="right" sideOffset={8}>
        {label}
      </TooltipContent>
    </Tooltip>
  )
}

export function Sidebar() {
  const { t } = useTranslation()
  const [collapsed, setCollapsed] = useState(false)
  const currentPath = useRouterState({ select: s => s.location.pathname })

  return (
    <TooltipProvider delayDuration={0}>
      <aside
        className={cn(
          'flex h-full flex-col border-r border-border bg-card transition-all duration-300 ease-in-out',
          collapsed ? 'w-14' : 'w-56',
        )}
      >
        <div className="flex h-10 items-center justify-end border-b border-border px-3">
          <button
            type="button"
            onClick={() => setCollapsed(!collapsed)}
            className="rounded p-1.5 hover:bg-accent cursor-pointer"
          >
            {collapsed
              ? (
                  <PanelLeftOpen className="h-4 w-4" />
                )
              : (
                  <PanelLeftClose className="h-4 w-4" />
                )}
          </button>
        </div>

        <nav className="flex-1 overflow-y-auto overflow-x-hidden py-4">
          <ul className="space-y-1 px-2">
            {categories.map(cat => (
              <li key={cat.id}>
                <SidebarLink
                  to={cat.path}
                  icon={cat.icon}
                  label={t(`sidebar.categories.${cat.id}`)}
                  collapsed={collapsed}
                  isActive={currentPath === cat.path}
                />
              </li>
            ))}
          </ul>
        </nav>

        <div className="border-t border-border p-2 space-y-1">
          <SidebarLink
            to="/autostart"
            icon={Rocket}
            label={t('sidebar.autostart')}
            collapsed={collapsed}
            isActive={currentPath === '/autostart'}
          />
          <SidebarLink
            to="/settings"
            icon={Settings}
            label={t('sidebar.settings')}
            collapsed={collapsed}
            isActive={currentPath === '/settings'}
          />
        </div>
      </aside>
    </TooltipProvider>
  )
}
