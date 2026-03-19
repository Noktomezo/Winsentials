import type { ReactNode } from 'react'
import type { DiskInfo, DiskLiveInfo, StaticSystemInfo } from '@/entities/system-info/model/types'
import type { ChartPoint } from '@/shared/ui/live-chart'
import { useParams } from '@tanstack/react-router'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { getStaticSystemInfo } from '@/entities/system-info/api'
import { useLiveDisks } from '@/entities/system-info/model/live-system-store'
import { formatBytesLocalized, formatRateLocalized } from '@/shared/lib/format-size'
import { useMountEffect } from '@/shared/lib/hooks/use-mount-effect'
import { mountToParam } from '@/shared/lib/mount-utils'
import { Button } from '@/shared/ui/button'
import { LiveChart } from '@/shared/ui/live-chart'
import { Skeleton } from '@/shared/ui/skeleton'

function formatBytes(bytes: number, locale: string, t: ReturnType<typeof useTranslation>['t'], decimals = 1): string {
  return formatBytesLocalized(bytes, { decimals, locale, t })
}

function formatRate(bytesPerSec: number, locale: string, t: ReturnType<typeof useTranslation>['t']): string {
  return formatRateLocalized(bytesPerSec, { locale, t })
}

interface RowProps {
  label: string
  value: ReactNode
}

function Row({ label, value }: RowProps) {
  return (
    <div className="flex items-center justify-between gap-4">
      <span className="text-xs text-muted-foreground">{label}</span>
      <span className="text-right text-xs font-medium text-foreground">{value}</span>
    </div>
  )
}

function getDiskLive(liveInfo: DiskLiveInfo[] | null, mountPoint: string): DiskLiveInfo | null {
  return liveInfo?.find(disk => disk.mountPoint === mountPoint) ?? null
}

export function DiskDetailPage() {
  const { t, i18n } = useTranslation()
  const { disk: diskParam } = useParams({ from: '/storage/$disk' })
  const [staticInfo, setStaticInfo] = useState<StaticSystemInfo | null>(null)
  const [staticInfoError, setStaticInfoError] = useState(false)
  const { data: liveInfo, activeHistory: storeActiveHistory } = useLiveDisks()

  const loadStaticInfo = () => {
    setStaticInfoError(false)
    getStaticSystemInfo()
      .then(setStaticInfo)
      .catch((error) => {
        console.error(error)
        setStaticInfoError(true)
      })
  }

  useMountEffect(() => {
    loadStaticInfo()
  })

  if (!staticInfo) {
    if (staticInfoError) {
      return (
        <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
          <section className="flex flex-col gap-3 rounded-xl border border-border/70 bg-card p-4">
            <p className="text-sm text-muted-foreground">{t('storage.loadError')}</p>
            <div>
              <Button onClick={loadStaticInfo} size="sm" type="button" variant="outline">
                {t('tweaks.actions.retry')}
              </Button>
            </div>
          </section>
        </section>
      )
    }

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
  const activeHistory: ChartPoint[] = (storeActiveHistory[disk.mountPoint] ?? []).map(v => ({ value: v }))

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
          <Row label={t('storage.readSpeed')} value={formatRate(diskLive?.readBytesPerSec ?? 0, i18n.language, t)} />
          <Row label={t('storage.writeSpeed')} value={formatRate(diskLive?.writeBytesPerSec ?? 0, i18n.language, t)} />
          <Row label={t('storage.capacity')} value={formatBytes(disk.totalBytes, i18n.language, t)} />
          <Row label={t('storage.format')} value={disk.fileSystem || '-'} />
          <Row label={t('storage.systemDisk')} value={disk.isSystemDisk ? t('storage.yes') : t('storage.no')} />
          <Row label={t('storage.pagefile')} value={disk.hasPagefile ? t('storage.yes') : t('storage.no')} />
          <Row label={t('storage.type')} value={disk.typeLabel} />
        </div>
      </section>
    </section>
  )
}
