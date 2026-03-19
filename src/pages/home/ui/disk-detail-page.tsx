import type { DiskInfo, DiskLiveInfo, LiveSystemInfo, StaticSystemInfo } from '@/entities/system-info/model/types'
import type { ChartPoint } from '@/shared/ui/live-chart'
import { useParams } from '@tanstack/react-router'
import { useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { getLiveSystemInfo, getStaticSystemInfo } from '@/entities/system-info/api'
import { mountToParam } from '@/shared/lib/mount-utils'
import { LiveChart } from '@/shared/ui/live-chart'
import { Skeleton } from '@/shared/ui/skeleton'

function formatBytes(bytes: number, decimals = 1): string {
  if (bytes === 0) { return '0 B' }
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return `${Number.parseFloat((bytes / k ** i).toFixed(decimals))} ${sizes[i]}`
}

function formatRate(bytesPerSec: number): string {
  return `${formatBytes(bytesPerSec)}/s`
}

function pushHistory(
  ref: React.MutableRefObject<ChartPoint[]>,
  value: number,
  setter: (v: ChartPoint[]) => void,
): void {
  const next = [...ref.current, { value }]
  ref.current = next.length > 60 ? next.slice(-60) : next
  setter([...ref.current])
}

function Row({ label, value }: { label: string, value: React.ReactNode }) {
  return (
    <div className="flex items-center justify-between gap-4">
      <span className="text-xs text-muted-foreground">{label}</span>
      <span className="text-right text-xs font-medium text-foreground">{value}</span>
    </div>
  )
}

function getDiskLive(liveInfo: LiveSystemInfo | null, mountPoint: string): DiskLiveInfo | null {
  return liveInfo?.disks.find(disk => disk.mountPoint === mountPoint) ?? null
}

export function DiskDetailPage() {
  const { t } = useTranslation()
  const { disk: diskParam } = useParams({ from: '/storage/$disk' })
  const [staticInfo, setStaticInfo] = useState<StaticSystemInfo | null>(null)
  const [liveInfo, setLiveInfo] = useState<LiveSystemInfo | null>(null)
  const activeHistoryRef = useRef<ChartPoint[]>([])
  const [activeHistory, setActiveHistory] = useState<ChartPoint[]>([])

  useEffect(() => {
    getStaticSystemInfo().then(setStaticInfo).catch(console.error)
  }, [])

  useEffect(() => {
    if (!staticInfo) { return }

    activeHistoryRef.current = []
    setActiveHistory([])

    const tick = () => {
      getLiveSystemInfo()
        .then((live) => {
          setLiveInfo(live)

          const disk = staticInfo.disks.find(entry => mountToParam(entry.mountPoint) === diskParam)
          if (!disk) {
            return
          }

          const diskLive = getDiskLive(live, disk.mountPoint)
          pushHistory(activeHistoryRef, diskLive?.activeTimePercent ?? 0, setActiveHistory)
        })
        .catch(console.error)
    }

    tick()
    const id = setInterval(tick, 1000)
    return () => clearInterval(id)
  }, [diskParam, staticInfo])

  if (!staticInfo) {
    return (
      <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
        <section className="rounded-xl border border-border/70 bg-card p-4">
          <Skeleton className="mb-3 h-4 w-32" />
          <div className="space-y-2.5">
            <Skeleton className="h-3 w-full" />
            <Skeleton className="h-24 w-full" />
            <Skeleton className="h-3 w-full" />
          </div>
        </section>
      </section>
    )
  }

  const diskIndex = staticInfo.disks.findIndex(d => mountToParam(d.mountPoint) === diskParam)
  const disk: DiskInfo | undefined = staticInfo.disks[diskIndex]

  if (!disk) {
    return (
      <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
        <p className="text-xs text-muted-foreground">{t('storage.notFound')}</p>
      </section>
    )
  }

  const diskLive = getDiskLive(liveInfo, disk.mountPoint)

  return (
    <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
      <section className="flex flex-col gap-1 rounded-xl border border-border/70 bg-card p-4">
        <div className="flex items-baseline justify-between">
          <span className="text-xs font-medium text-foreground">{t('storage.activeTime')}</span>
          <span className="text-xs tabular-nums text-muted-foreground">100%</span>
        </div>
        <LiveChart data={activeHistory} height={96} unit="%" yDomain={[0, 100]} />
        <div className="flex items-baseline justify-between">
          <span className="text-xs text-muted-foreground">{t('ram.seconds', { n: 60 })}</span>
          <span className="text-xs tabular-nums text-muted-foreground">0</span>
        </div>
      </section>

      <section className="flex flex-col gap-3 rounded-xl border border-border/70 bg-card p-4">
        <h3 className="text-sm font-medium text-foreground">{t('storage.diskInfo')}</h3>
        <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
          <Row label={t('storage.activeTime')} value={`${diskLive?.activeTimePercent ?? 0}%`} />
          <Row label={t('storage.avgResponseTime')} value={`${(diskLive?.avgResponseMs ?? 0).toFixed(1)} ms`} />
          <Row label={t('storage.readSpeed')} value={formatRate(diskLive?.readBytesPerSec ?? 0)} />
          <Row label={t('storage.writeSpeed')} value={formatRate(diskLive?.writeBytesPerSec ?? 0)} />
          <Row label={t('storage.capacity')} value={formatBytes(disk.totalBytes)} />
          <Row label={t('storage.format')} value={disk.fileSystem || '-'} />
          <Row label={t('storage.systemDisk')} value={disk.isSystemDisk ? t('storage.yes') : t('storage.no')} />
          <Row label={t('storage.pagefile')} value={disk.hasPagefile ? t('storage.yes') : t('storage.no')} />
          <Row label={t('storage.type')} value={disk.typeLabel} />
        </div>
      </section>
    </section>
  )
}
