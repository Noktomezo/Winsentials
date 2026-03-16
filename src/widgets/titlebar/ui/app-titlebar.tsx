import type { ComponentProps } from 'react'
import { Link, useRouterState } from '@tanstack/react-router'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { Minus, X } from 'lucide-react'
import { Fragment } from 'react'
import { useTranslation } from 'react-i18next'
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from '@/shared/ui/breadcrumb'
import { Button } from '@/shared/ui/button'
import { SidebarTrigger } from '@/shared/ui/sidebar'

interface Crumb { label: string, href?: string }

function useBreadcrumbs(): Crumb[] {
  const pathname = useRouterState({ select: s => s.location.pathname })
  const { t } = useTranslation()

  const home: Crumb = { label: t('home.title'), href: '/home' }

  if (pathname === '/home') { return [{ label: t('home.title') }] }

  const hardwareMap: Record<string, string> = {
    '/cpu': t('cpu.title'),
    '/ram': t('ram.title'),
    '/gpu': t('gpu.title'),
    '/storage': t('storage.title'),
    '/network-stats': t('networkStats.title'),
  }
  if (hardwareMap[pathname]) { return [home, { label: hardwareMap[pathname] }] }

  if (pathname.startsWith('/storage/')) {
    const disk = `${pathname.replace('/storage/', '').toUpperCase()}:`
    return [home, { label: t('storage.title'), href: '/storage' }, { label: disk }]
  }

  const topLevel: Record<string, string> = {
    '/appearance': t('appearance.title'),
    '/behaviour': t('behaviour.title'),
    '/security': t('security.title'),
    '/network': t('network.title'),
    '/settings': t('settings.title'),
  }
  if (topLevel[pathname]) { return [{ label: topLevel[pathname] }] }

  return [{ label: t('app.title') }]
}

function TitlebarButton({
  className,
  ...props
}: ComponentProps<typeof Button>) {
  return (
    <Button
      className={className}
      size="icon-xs"
      type="button"
      variant="ghost"
      {...props}
    />
  )
}

export function AppTitlebar() {
  const win = getCurrentWindow()
  const crumbs = useBreadcrumbs()

  const handleMinimize = async () => {
    await win.minimize()
  }

  const handleClose = async () => {
    await win.close()
  }

  return (
    <header className="relative flex h-10 shrink-0 items-center bg-sidebar px-2 text-sidebar-foreground">
      <SidebarTrigger
        className="size-8 cursor-pointer rounded-md text-sidebar-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground"
        iconClassName="size-3.5"
      />

      {/* Absolutely centred breadcrumb — independent of left/right button widths */}
      <div className="pointer-events-none absolute inset-0 flex items-center justify-center">
        <div className="pointer-events-auto">
          <Breadcrumb>
            <BreadcrumbList className="gap-1 text-xs sm:gap-1.5 flex-nowrap">
              {crumbs.map((crumb, i) => {
                const isLast = i === crumbs.length - 1
                return (
                  <Fragment key={crumb.label}>
                    {i > 0 && (
                      <BreadcrumbSeparator className="text-sidebar-foreground/40 [&>svg]:size-3" />
                    )}
                    <BreadcrumbItem>
                      {!isLast && crumb.href
                        ? (
                            <BreadcrumbLink
                              asChild
                              className="text-sidebar-foreground/60 hover:text-sidebar-foreground text-xs"
                            >
                              <Link to={crumb.href}>{crumb.label}</Link>
                            </BreadcrumbLink>
                          )
                        : (
                            <BreadcrumbPage className="text-sidebar-foreground text-xs font-medium">
                              {crumb.label}
                            </BreadcrumbPage>
                          )}
                    </BreadcrumbItem>
                  </Fragment>
                )
              })}
            </BreadcrumbList>
          </Breadcrumb>
        </div>
      </div>

      {/* Drag region fills remaining space */}
      <div className="min-w-0 flex-1 self-stretch" data-tauri-drag-region />

      <div className="flex items-center gap-1">
        <TitlebarButton
          aria-label="Minimize window"
          className="h-8 w-8 cursor-pointer rounded-md p-0 text-sidebar-foreground hover:bg-sidebar-accent hover:text-sidebar-accent-foreground"
          onClick={() => {
            void handleMinimize()
          }}
        >
          <Minus className="size-4" />
        </TitlebarButton>
        <TitlebarButton
          aria-label="Close window"
          className="h-8 w-8 cursor-pointer rounded-md p-0 text-sidebar-foreground hover:bg-destructive hover:text-white focus-visible:ring-destructive/20 dark:hover:bg-destructive/60 dark:focus-visible:ring-destructive/40"
          onClick={() => {
            void handleClose()
          }}
        >
          <X className="size-4" />
        </TitlebarButton>
      </div>
    </header>
  )
}
