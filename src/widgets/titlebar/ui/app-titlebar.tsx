import type { ComponentProps } from 'react'
import { Link, useRouterState } from '@tanstack/react-router'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { Copy as CopyIcon, Minus, Square, X } from 'lucide-react'
import { Fragment, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useStaticInfo } from '@/entities/system-info/model/static-system-info'
import { useMountEffect } from '@/shared/lib/hooks/use-mount-effect'
import { mountToParam } from '@/shared/lib/mount-utils'
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
  Button,
  SidebarTrigger,
} from '@/shared/ui'

interface Crumb { label: string, href?: string }

function useBreadcrumbs(): Crumb[] {
  const pathname = useRouterState({ select: s => s.location.pathname })
  const { t } = useTranslation()
  const staticInfo = useStaticInfo()

  const home: Crumb = { label: t('home.title'), href: '/home' }

  if (pathname === '/home') return [{ label: t('home.title') }]

  const hardwareMap: Record<string, string> = {
    '/cpu': t('home.cpu'),
    '/ram': t('home.ram'),
    '/gpu': t('home.gpu'),
    '/network-stats': t('home.network'),
  }
  if (hardwareMap[pathname]) return [home, { label: hardwareMap[pathname] }]

  if (pathname.startsWith('/gpu/')) {
    const idx = Number(pathname.replace('/gpu/', ''))
    if (!Number.isInteger(idx) || idx < 0 || (staticInfo && idx >= staticInfo.gpus.length))
      return [home, { label: t('home.gpu') }]
    return [home, { label: t('gpu.gpuLabel', { index: idx }) }]
  }

  if (pathname.startsWith('/network-stats/')) {
    return [home, { label: t('home.network') }]
  }

  if (pathname.startsWith('/storage/')) {
    const param = pathname.replace('/storage/', '')
    const idx = staticInfo?.disks.findIndex(d => mountToParam(d.mountPoint) === param) ?? -1
    const label = idx >= 0 ? t('storage.diskLabel', { index: idx }) : param.toUpperCase()
    return [home, { label }]
  }

  const topLevel: Record<string, string> = {
    '/appearance': t('appearance.title'),
    '/behaviour': t('behaviour.title'),
    '/security': t('security.title'),
    '/privacy': t('privacy.title'),
    '/network': t('network.title'),
    '/performance': t('performance.title'),
    '/memory': t('memory.title'),
    '/input': t('input.title'),
    '/tools': t('tools.title'),
    '/settings': t('settings.title'),
  }
  if (topLevel[pathname]) return [{ label: topLevel[pathname] }]

  if (pathname === '/startup') {
    return [
      { label: t('tools.title'), href: '/tools' },
      { label: t('startup.title') },
    ]
  }

  if (pathname === '/backup') {
    return [
      { label: t('tools.title'), href: '/tools' },
      { label: t('backup.title') },
    ]
  }

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
  const { t } = useTranslation()
  const [isMaximized, setIsMaximized] = useState(false)

  const syncMaximizedState = async () => {
    setIsMaximized(await win.isMaximized())
  }

  useMountEffect(() => {
    let disposed = false

    const syncIfMounted = async () => {
      const maximized = await win.isMaximized()
      if (!disposed) {
        setIsMaximized(maximized)
      }
    }

    void syncIfMounted()

    const unlistenPromise = win.listen('tauri://resize', () => {
      void syncIfMounted()
    })

    return () => {
      disposed = true
      void unlistenPromise.then(unlisten => unlisten())
    }
  })

  const handleMinimize = async () => {
    await win.minimize()
  }

  const handleClose = async () => {
    await win.close()
  }

  const handleToggleMaximize = async () => {
    await win.toggleMaximize()
    await syncMaximizedState()
  }

  return (
    <header
      className="relative flex h-10 shrink-0 items-center bg-transparent px-2 text-sidebar-foreground"
      data-slot="app-titlebar"
    >
      <SidebarTrigger
        className="titlebar-control"
        iconClassName="size-3.5"
      />

      {/* Absolutely centred breadcrumb — independent of left/right button widths */}
      <div className="pointer-events-none absolute inset-0 z-10 flex items-center justify-center">
        <div className="pointer-events-auto max-w-[min(40rem,calc(100%-9rem))] overflow-hidden">
          <Breadcrumb>
            <BreadcrumbList className="max-w-full flex-nowrap gap-1 overflow-hidden text-xs sm:gap-1.5">
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
                              className="max-w-full truncate text-xs text-sidebar-foreground/60 hover:text-sidebar-foreground"
                            >
                              <Link
                                className="max-w-full cursor-pointer truncate"
                                to={crumb.href}
                              >
                                {crumb.label}
                              </Link>
                            </BreadcrumbLink>
                          )
                        : (
                            <BreadcrumbPage className="max-w-full truncate text-xs font-medium text-sidebar-foreground">
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
          aria-label={t('titlebar.minimize')}
          className="titlebar-control titlebar-control--warning"
          onClick={() => {
            void handleMinimize()
          }}
        >
          <Minus className="size-4" />
        </TitlebarButton>
        <TitlebarButton
          aria-label={isMaximized ? t('titlebar.restore') : t('titlebar.maximize')}
          className="titlebar-control titlebar-control--good"
          onClick={() => {
            void handleToggleMaximize()
          }}
        >
          {isMaximized ? <CopyIcon className="size-3.5" /> : <Square className="size-3.5" />}
        </TitlebarButton>
        <TitlebarButton
          aria-label={t('titlebar.close')}
          className="titlebar-control titlebar-control--danger"
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
