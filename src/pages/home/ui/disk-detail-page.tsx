import type { ReactNode } from 'react'
import type { DiskInfo, DiskLiveInfo } from '@/entities/system-info/model/types'
import type { ChartPoint } from '@/shared/ui/live-chart'
import { useParams } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'
import { useDeviceInventory, useLiveDisks } from '@/entities/system-info/model/live-system-store'
import { formatBytesLocalized, formatRateLocalized } from '@/shared/lib/format-size'
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
  const { data: liveInfo, activeHistory: storeActiveHistory } = useLiveDisks()
  const {
    data: deviceInventory,
    error: inventoryError,
    isFetching: inventoryFetching,
    retry: retryInventory,
  } = useDeviceInventory()

  if (deviceInventory === null) {
    if (inventoryError) {
      return (
        <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
          <section className="flex flex-col gap-3 rounded-xl border border-border/70 bg-card p-4">
            <p className="text-sm text-muted-foreground">{t('storage.loadError')}</p>
            <div>
              <Button onClick={retryInventory} size="sm" type="button" variant="outline">
                {t('tweaks.actions.retry')}
              </Button>
            </div>
          </section>
        </section>
      )
    }

    if (inventoryFetching) {
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
  }

  const disk: DiskInfo | undefined = deviceInventory?.disks.find(d => mountToParam(d.mountPoint) === diskParam)

  if (deviceInventory !== null && !disk) {
    return (
      <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
        <p className="text-xs text-muted-foreground">{t('storage.diskUnavailable')}</p>
      </section>
    )
  }

  if (!disk) {
    return null
  }

  const diskLive = getDiskLive(liveInfo, disk.mountPoint)
  const activeHistory: ChartPoint[] = (storeActiveHistory[disk.mountPoint] ?? []).map(v => ({ value: v }))
  const avgResponseTime = new Intl.NumberFormat(i18n.language, {
    minimumFractionDigits: 1,
    maximumFractionDigits: 1,
  }).format(diskLive?.avgResponseMs ?? 0)
  const diskType = t(`storage.types.${disk.kind.toLowerCase()}`, { defaultValue: disk.typeLabel })

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
          <Row label={t('storage.avgResponseTime')} value={`${avgResponseTime} ms`} />
          <Row label={t('storage.readSpeed')} value={formatRate(diskLive?.readBytesPerSec ?? 0, i18n.language, t)} />
          <Row label={t('storage.writeSpeed')} value={formatRate(diskLive?.writeBytesPerSec ?? 0, i18n.language, t)} />
          <Row label={t('storage.device')} value={disk.model ?? disk.name} />
          <Row label={t('storage.capacity')} value={formatBytes(disk.totalBytes, i18n.language, t)} />
          <Row label={t('storage.format')} value={disk.fileSystem || '-'} />
          <Row label={t('storage.systemDisk')} value={disk.isSystemDisk ? t('storage.yes') : t('storage.no')} />
          <Row label={t('storage.pagefile')} value={disk.hasPagefile ? t('storage.yes') : t('storage.no')} />
          <Row label={t('storage.type')} value={diskType} />
        </div>
      </section>
    </section>
  )
}
