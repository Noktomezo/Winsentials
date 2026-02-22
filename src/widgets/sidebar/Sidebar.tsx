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
  Settings,
  Shield,
  ShieldCheck,
  Zap,
} from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { cn } from '@/lib/utils'

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

export function Sidebar() {
  const { t } = useTranslation()
  const [collapsed, setCollapsed] = useState(false)
  const currentPath = useRouterState({ select: s => s.location.pathname })

  return (
    <aside
      className={cn(
        'flex h-full flex-col border-r border-border bg-card transition-all duration-300',
        collapsed ? 'w-14' : 'w-56',
      )}
    >
      <div
        className={cn(
          'flex h-10 items-center border-b border-border px-3',
          collapsed ? 'justify-center' : 'justify-end',
        )}
      >
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

      <nav className="flex-1 py-4">
        <ul className="space-y-1 px-2">
          {categories.map((cat) => {
            const Icon = cat.icon
            const isActive = currentPath === cat.path
            return (
              <li key={cat.id}>
                <Link
                  to={cat.path}
                  className={cn(
                    'flex items-center gap-3 rounded-lg px-3 py-2 text-sm transition-colors',
                    collapsed && 'justify-center px-0',
                    isActive
                      ? 'bg-primary text-primary-foreground'
                      : 'text-muted-foreground hover:bg-accent hover:text-foreground',
                  )}
                >
                  <Icon className="h-5 w-5 shrink-0" />
                  <span className={cn('truncate', collapsed && 'hidden')}>
                    {t(`sidebar.categories.${cat.id}`)}
                  </span>
                </Link>
              </li>
            )
          })}
        </ul>
      </nav>

      <div className="border-t border-border p-2">
        <Link
          to="/settings"
          className={cn(
            'flex w-full items-center gap-3 rounded-lg px-3 py-2 text-sm text-muted-foreground hover:bg-accent hover:text-foreground',
            collapsed && 'justify-center px-0',
            currentPath === '/settings' && 'bg-primary text-primary-foreground',
          )}
        >
          <Settings className="h-5 w-5 shrink-0" />
          <span className={cn('truncate', collapsed && 'hidden')}>
            {t('sidebar.settings')}
          </span>
        </Link>
      </div>
    </aside>
  )
}
