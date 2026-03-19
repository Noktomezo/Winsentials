import type { ReactNode } from 'react'
import type { StaticSystemInfo } from '@/entities/system-info/model/types'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { getStaticSystemInfo } from '@/entities/system-info/api'
import { mountLabel, mountToParam } from '@/shared/lib/mount-utils'

export interface PageHeader {
  title: ReactNode
  description: string
}

// Module-level cache so the IPC call fires only once per app session.
let staticInfoCache: StaticSystemInfo | null = null
let staticInfoPromise: Promise<StaticSystemInfo> | null = null
const listeners: Array<(info: StaticSystemInfo) => void> = []

function loadStaticInfo(): Promise<StaticSystemInfo> {
  if (staticInfoCache) {
    return Promise.resolve(staticInfoCache)
  }

  if (staticInfoPromise) {
    return staticInfoPromise
  }

  staticInfoPromise = getStaticSystemInfo()
    .then((info) => {
      staticInfoCache = info
      for (const fn of listeners) { fn(info) }
      listeners.length = 0
      return info
    })
    .finally(() => {
      staticInfoPromise = null
    })

  return staticInfoPromise
}

export function useStaticInfo(): StaticSystemInfo | null {
  const [info, setInfo] = useState<StaticSystemInfo | null>(staticInfoCache)

  useEffect(() => {
    if (staticInfoCache) {
      setInfo(staticInfoCache)
      return
    }
    listeners.push(setInfo)
    loadStaticInfo().catch(console.error)
    return () => {
      const idx = listeners.indexOf(setInfo)
      if (idx !== -1) { listeners.splice(idx, 1) }
    }
  }, [])

  return info
}

export function usePageHeader(pathname: string): PageHeader {
  const { t } = useTranslation()
  const staticInfo = useStaticInfo()

  // ── Static pages ────────────────────────────────────────────────────────────
  const staticMap: Record<string, PageHeader> = {
    '/home': { title: t('home.title'), description: t('home.description') },
    '/ram': { title: t('home.ram'), description: t('ram.description') },
    '/gpu': { title: t('home.gpu'), description: t('gpu.description') },
    '/network-stats': { title: t('home.network'), description: t('networkStats.description') },
    '/behaviour': { title: t('behaviour.title'), description: t('behaviour.description') },
    '/appearance': { title: t('appearance.title'), description: t('appearance.description') },
    '/security': { title: t('security.title'), description: t('security.description') },
    '/network': { title: t('network.title'), description: t('network.description') },
    '/settings': { title: t('settings.title'), description: t('settings.description') },
  }
  if (staticMap[pathname]) { return staticMap[pathname] }

  // ── CPU: /cpu ────────────────────────────────────────────────────────────────
  if (pathname === '/cpu') {
    const model = staticInfo?.cpu.model
    return {
      title: model
        ? (
            <span className="flex items-baseline gap-1.5">
              <span>{t('home.cpu')}</span>
              <span className="text-base font-normal text-muted-foreground">
                (
                {model}
                )
              </span>
            </span>
          )
        : t('home.cpu'),
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
        : label,
      description: t('gpu.description'),
    }
  }

  // ── Disk detail: /storage/C … ────────────────────────────────────────────────
  if (pathname.startsWith('/storage/')) {
    const param = pathname.replace('/storage/', '')
    const disk = staticInfo?.disks.find(d => mountToParam(d.mountPoint) === param)
    const idx = staticInfo?.disks.findIndex(d => mountToParam(d.mountPoint) === param) ?? -1
    const diskLabel = idx >= 0 ? t('storage.diskLabel', { index: idx }) : param.toUpperCase()
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
        : diskLabel,
      description: t('storage.description'),
    }
  }

  if (pathname.startsWith('/network-stats/')) {
    const adapterName = decodeURIComponent(pathname.replace('/network-stats/', ''))
    const adapter = staticInfo?.networkAdapters.find(entry => entry.name === adapterName) ?? null
    return {
      title: adapter
        ? (
            <span className="flex items-baseline gap-1.5">
              <span>{t('home.network')}</span>
              <span className="text-base font-normal text-muted-foreground">
                (
                {adapter.name}
                )
              </span>
            </span>
          )
        : t('home.network'),
      description: t('networkStats.description'),
    }
  }

  return { title: t('app.title'), description: '' }
}
