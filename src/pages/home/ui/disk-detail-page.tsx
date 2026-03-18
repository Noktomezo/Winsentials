import type { DiskInfo, StaticSystemInfo } from '@/entities/system-info/model/types'
import { useParams } from '@tanstack/react-router'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { getStaticSystemInfo } from '@/entities/system-info/api'
import { mountLabel, mountToParam } from '@/shared/lib/mount-utils'
import { Progress } from '@/shared/ui/progress'
import { Skeleton } from '@/shared/ui/skeleton'

function formatBytes(bytes: number, decimals = 1): string {
  if (bytes === 0) { return '0 B' }
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return `${Number.parseFloat((bytes / k ** i).toFixed(decimals))} ${sizes[i]}`
}

function usagePct(used: number, total: number): number {
  if (total === 0) { return 0 }
  return Math.round((used / total) * 100)
}

function Row({ label, value }: { label: string, value: React.ReactNode }) {
  return (
    <div className="flex items-center justify-between gap-4">
      <span className="text-xs text-muted-foreground">{label}</span>
      <span className="text-right text-xs font-medium text-foreground">{value}</span>
    </div>
  )
}

export function DiskDetailPage() {
  const { t } = useTranslation()
  const { disk: diskParam } = useParams({ from: '/storage/$disk' })
  const [staticInfo, setStaticInfo] = useState<StaticSystemInfo | null>(null)

  useEffect(() => {
    getStaticSystemInfo().then(setStaticInfo).catch(console.error)
  }, [])

  if (!staticInfo) {
    return (
      <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
        <section className="rounded-xl border border-border/70 bg-card p-4">
          <Skeleton className="mb-3 h-4 w-32" />
          <div className="space-y-2.5">
            <Skeleton className="h-3 w-full" />
            <Skeleton className="h-3 w-4/5" />
            <Skeleton className="h-3 w-3/5" />
            <Skeleton className="h-2 w-full rounded-full" />
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

  const used = disk.totalBytes - disk.availableBytes
  const pct = usagePct(used, disk.totalBytes)

  return (
    <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
      <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
        <section className="flex flex-col gap-3 rounded-xl border border-border/70 bg-card p-4">
          <h3 className="text-sm font-medium text-foreground">{t('storage.diskInfo')}</h3>
          <Row label={t('storage.mountPoint')} value={mountLabel(disk.mountPoint)} />
          {disk.volumeLabel && (
            <Row label={t('storage.volumeLabel')} value={disk.volumeLabel} />
          )}
          {disk.name && disk.name !== disk.mountPoint && (
            <Row label={t('storage.device')} value={disk.name} />
          )}
          <Row label={t('storage.type')} value={disk.kind} />
          <Row label={t('storage.fileSystem')} value={disk.fileSystem} />
          <Row label={t('home.total')} value={formatBytes(disk.totalBytes)} />
          <Row label={t('storage.used')} value={formatBytes(used)} />
          <Row label={t('storage.available')} value={formatBytes(disk.availableBytes)} />
        </section>

        <section className="flex flex-col gap-3 rounded-xl border border-border/70 bg-card p-4">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-medium text-foreground">{t('home.usage')}</h3>
            <span className="text-sm font-semibold tabular-nums text-foreground">
              {pct}
              %
            </span>
          </div>
          <Progress className="h-1.5" value={pct} />
          <p className="text-xs text-muted-foreground">
            {t('home.usedOf', { used: formatBytes(used), total: formatBytes(disk.totalBytes) })}
          </p>
        </section>
      </div>
    </section>
  )
}
