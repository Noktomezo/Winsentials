import type { DiskInfo, StaticSystemInfo } from '@/entities/system-info/model/types'
import { useNavigate } from '@tanstack/react-router'
import { ChevronRight, HardDrive } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { getStaticSystemInfo } from '@/entities/system-info/api'
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

export function mountToParam(mountPoint: string): string {
  return mountPoint.replace(/[:\\/]/g, '')
}

function DiskCard({ disk, index }: { disk: DiskInfo, index: number }) {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const used = disk.totalBytes - disk.availableBytes
  const pct = usagePct(used, disk.totalBytes)

  return (
    <button
      className="group flex flex-col gap-3 rounded-xl border border-border/70 bg-card p-4 text-left transition-colors hover:bg-accent/20 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
      onClick={() => void navigate({ to: '/storage/$disk', params: { disk: mountToParam(disk.mountPoint) } })}
      type="button"
    >
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <span className="flex size-7 shrink-0 items-center justify-center rounded-md bg-accent/60 text-accent-foreground">
            <HardDrive className="size-3.5" />
          </span>
          <div>
            <h2 className="text-sm font-medium text-foreground">
              {t('storage.diskLabel', { index })}
            </h2>
            <p className="text-xs text-muted-foreground">{disk.mountPoint}</p>
          </div>
        </div>
        <ChevronRight className="size-3.5 text-muted-foreground transition-transform group-hover:translate-x-0.5" />
      </div>
      <div className="flex justify-between text-xs text-muted-foreground">
        <span>{disk.kind}</span>
        <span>{disk.fileSystem}</span>
      </div>
      <div className="space-y-0.5">
        <div className="flex justify-between">
          <span className="text-xs text-muted-foreground">{t('home.usage')}</span>
          <span className="text-xs font-medium tabular-nums text-foreground">
            {t('home.usedOf', { used: formatBytes(used), total: formatBytes(disk.totalBytes) })}
          </span>
        </div>
        <Progress className="h-1.5" value={pct} />
      </div>
    </button>
  )
}

export function StoragePage() {
  const [staticInfo, setStaticInfo] = useState<StaticSystemInfo | null>(null)

  useEffect(() => {
    getStaticSystemInfo().then(setStaticInfo).catch(console.error)
  }, [])

  if (!staticInfo) {
    return (
      <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
          {Array.from({ length: 2 }).map((_, i) => (
            <section className="rounded-xl border border-border/70 bg-card p-4" key={i}>
              <Skeleton className="mb-3 h-4 w-32" />
              <div className="space-y-2.5">
                <Skeleton className="h-3 w-full" />
                <Skeleton className="h-2 w-full rounded-full" />
              </div>
            </section>
          ))}
        </div>
      </section>
    )
  }

  return (
    <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
      <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
        {staticInfo.disks.map((disk, i) => (
          <DiskCard disk={disk} index={i} key={disk.mountPoint} />
        ))}
      </div>
    </section>
  )
}
