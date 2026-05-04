import type { ReactNode } from 'react'
import { RefreshCw, Trash2 } from 'lucide-react'
import { useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useCleanupUiState } from '@/entities/cleanup/model/ui-state'
import { formatCpuModel } from '@/entities/system-info/lib/format-hardware-name'
import { useStaticInfo } from '@/entities/system-info/model/static-system-info'
import { formatBytesLocalized } from '@/shared/lib/format-size'
import { useMountEffect } from '@/shared/lib/hooks/use-mount-effect'
import { mountLabel, mountToParam } from '@/shared/lib/mount-utils'
import { safeDecodeSegment } from '@/shared/lib/safe-decode-segment'
import { cn } from '@/shared/lib/utils'
import { Button } from '@/shared/ui/button'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/shared/ui/tooltip'

export interface PageHeader {
  actions?: ReactNode
  title: ReactNode
  description: string
}

interface CleanupSummary {
  cleanableCount: number
  sizeBytes: number
  targetCount: number
}

const CLEANUP_SUMMARY_EVENT = 'winsentials:cleanup-summary'

export function usePageHeader(pathname: string): PageHeader {
  const { i18n, t } = useTranslation()
  const staticInfo = useStaticInfo()
  const cleanupUiState = useCleanupUiState()
  const [cleanupSummary, setCleanupSummary] = useState<CleanupSummary>({ cleanableCount: 0, sizeBytes: 0, targetCount: 0 })
  const cleanupBusy = cleanupUiState.busy
  const cleanupRefreshing = cleanupUiState.refreshingCategories.size > 0
  const cleanupCleanDisabled = cleanupBusy || cleanupRefreshing || cleanupSummary.cleanableCount === 0

  useMountEffect(() => {
    const handleCleanupSummary = (event: Event) => {
      const detail = (event as CustomEvent<CleanupSummary>).detail
      if (!detail) return

      setCleanupSummary({
        cleanableCount: detail.cleanableCount,
        sizeBytes: detail.sizeBytes,
        targetCount: detail.targetCount,
      })
    }

    window.addEventListener(CLEANUP_SUMMARY_EVENT, handleCleanupSummary)
    return () => window.removeEventListener(CLEANUP_SUMMARY_EVENT, handleCleanupSummary)
  })

  // ── Static pages ────────────────────────────────────────────────────────────
  const staticMap: Record<string, PageHeader> = useMemo(
    () => ({
      '/home': { title: t('app.title'), description: t('app.description') },
      '/ram': { title: t('home.ram'), description: t('ram.description') },
      '/gpu': { title: t('home.gpu'), description: t('gpu.description') },
      '/network-stats': {
        title: t('home.network'),
        description: t('networkStats.description'),
      },
      '/startup': {
        title: t('startup.title'),
        description: t('startup.description'),
      },
      '/behaviour': {
        title: t('behaviour.title'),
        description: t('behaviour.description'),
      },
      '/appearance': {
        title: t('appearance.title'),
        description: t('appearance.description'),
      },
      '/security': {
        title: t('security.title'),
        description: t('security.description'),
      },
      '/privacy': {
        title: t('privacy.title'),
        description: t('privacy.description'),
      },
      '/network': {
        title: t('network.title'),
        description: t('network.description'),
      },
      '/performance': {
        title: t('performance.title'),
        description: t('performance.description'),
      },
      '/memory': {
        title: t('memory.title'),
        description: t('memory.description'),
      },
      '/input': {
        title: t('input.title'),
        description: t('input.description'),
      },
      '/tools': {
        title: t('tools.title'),
        description: t('tools.description'),
      },
      '/cleanup': {
        title: (
          <span className="flex flex-wrap items-center gap-2">
            <span>{t('cleanup.title')}</span>
            <span className="rounded-md border border-border/60 bg-accent/45 px-1.5 py-0.5 text-xs font-normal text-muted-foreground">
              {t('cleanup.itemsCount', { count: cleanupSummary.targetCount })}
            </span>
            <span className="rounded-md border border-border/60 bg-accent/45 px-1.5 py-0.5 text-xs font-normal text-muted-foreground">
              {formatBytesLocalized(cleanupSummary.sizeBytes, { decimals: 1, locale: i18n.language, t })}
            </span>
          </span>
        ),
        description: t('cleanup.description'),
        actions: (
          <div className="flex items-center gap-2">
            <Button
              disabled={cleanupCleanDisabled}
              onClick={() => window.dispatchEvent(new Event('winsentials:cleanup-clean-all'))}
              size="sm"
              type="button"
            >
              <Trash2 className="size-4" />
              {t('cleanup.cleanAll')}
            </Button>
            <Tooltip>
              <TooltipTrigger asChild>
                <Button
                  aria-label={t('cleanup.refreshAll')}
                  disabled={cleanupBusy || cleanupRefreshing}
                  onClick={() => window.dispatchEvent(new Event('winsentials:cleanup-refresh-all'))}
                  size="icon-sm"
                  type="button"
                  variant="outline"
                >
                  <RefreshCw className={cn('size-4', cleanupRefreshing && 'animate-spin')} />
                </Button>
              </TooltipTrigger>
              <TooltipContent sideOffset={8}>{t('cleanup.refreshAll')}</TooltipContent>
            </Tooltip>
          </div>
        ),
      },
      '/settings': {
        title: t('settings.title'),
        description: t('settings.description'),
      },
      '/backup': {
        title: t('backup.title'),
        description: t('backup.description'),
      },
    }),
    [cleanupBusy, cleanupCleanDisabled, cleanupRefreshing, cleanupSummary.sizeBytes, cleanupSummary.targetCount, i18n.language, t],
  )
  if (staticMap[pathname]) return staticMap[pathname]

  // ── CPU: /cpu ────────────────────────────────────────────────────────────────
  if (pathname === '/cpu') {
    const model = staticInfo ? formatCpuModel(staticInfo.cpu.model) : null
    return {
      title: model
        ? (
            <span className="flex items-baseline gap-1.5">
              <span>{t('home.cpu')}</span>
              <span className="text-base font-normal text-muted-foreground">
                {`(${model})`}
              </span>
            </span>
          )
        : (
            t('home.cpu')
          ),
      description: t('cpu.description'),
    }
  }

  // ── GPU detail: /gpu/0, /gpu/1 … ────────────────────────────────────────────
  if (pathname.startsWith('/gpu/')) {
    const idx = Number(pathname.replace('/gpu/', ''))
    if (!Number.isInteger(idx) || idx < 0) {
      return { title: t('home.gpu'), description: t('gpu.description') }
    }
    const isValidIdx = idx < (staticInfo?.gpus.length ?? 0)
    if (!isValidIdx && staticInfo) {
      return { title: t('home.gpu'), description: t('gpu.description') }
    }
    const gpu = isValidIdx ? staticInfo?.gpus[idx] : null
    const label = t('gpu.gpuLabel', { index: idx })
    return {
      title: gpu
        ? (
            <span className="flex items-baseline gap-1.5">
              <span>{label}</span>
              <span className="text-base font-normal text-muted-foreground">
                (
                {gpu.name}
                )
              </span>
            </span>
          )
        : (
            label
          ),
      description: t('gpu.description'),
    }
  }

  // ── Disk detail: /storage/C … ────────────────────────────────────────────────
  if (pathname.startsWith('/storage/')) {
    const param = pathname.replace('/storage/', '')
    const idx
      = staticInfo?.disks.findIndex(
        d => mountToParam(d.mountPoint) === param,
      ) ?? -1
    const disk = idx >= 0 ? (staticInfo?.disks[idx] ?? null) : null
    const diskLabel
      = idx >= 0 ? t('storage.diskLabel', { index: idx }) : param.toUpperCase()
    const diskSub = disk
      ? disk.volumeLabel
        ? `${mountLabel(disk.mountPoint)} - ${disk.volumeLabel}`
        : mountLabel(disk.mountPoint)
      : null
    return {
      title: diskSub
        ? (
            <span className="flex items-baseline gap-1.5">
              <span>{diskLabel}</span>
              <span className="text-base font-normal text-muted-foreground">
                (
                {diskSub}
                )
              </span>
            </span>
          )
        : (
            diskLabel
          ),
      description: t('storage.description'),
    }
  }

  if (pathname.startsWith('/network-stats/')) {
    const adapterName = safeDecodeSegment(
      pathname.replace('/network-stats/', ''),
    )
    const adapter
      = staticInfo?.networkAdapters.find(entry => entry.name === adapterName)
        ?? null
    const adapterLabel = adapter?.adapterDescription || adapter?.name || null
    return {
      title: adapterLabel
        ? (
            <span className="flex items-baseline gap-1.5">
              <span>{t('home.network')}</span>
              <span className="text-base font-normal text-muted-foreground">
                (
                {adapterLabel}
                )
              </span>
            </span>
          )
        : (
            t('home.network')
          ),
      description: t('networkStats.description'),
    }
  }

  return { title: t('app.title'), description: '' }
}
