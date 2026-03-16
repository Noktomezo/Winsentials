import type { LiveSystemInfo, StaticSystemInfo } from '@/entities/system-info/model/types'
import type { ChartPoint } from '@/shared/ui/live-chart'
import { useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { getLiveSystemInfo, getStaticSystemInfo } from '@/entities/system-info/api'
import { LiveChart } from '@/shared/ui/live-chart'
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

export function RamPage() {
  const { t } = useTranslation()
  const [staticInfo, setStaticInfo] = useState<StaticSystemInfo | null>(null)
  const [liveInfo, setLiveInfo] = useState<LiveSystemInfo | null>(null)
  const [history, setHistory] = useState<ChartPoint[]>([])
  const historyRef = useRef<ChartPoint[]>([])

  useEffect(() => {
    getStaticSystemInfo().then(setStaticInfo).catch(console.error)
  }, [])

  useEffect(() => {
    if (!staticInfo) { return }
    const tick = () => {
      getLiveSystemInfo()
        .then((live) => {
          setLiveInfo(live)
          const pct = usagePct(live.ramUsedBytes, staticInfo.ram.totalBytes)
          const next = [...historyRef.current, { value: pct }]
          historyRef.current = next.length > 60 ? next.slice(-60) : next
          setHistory([...historyRef.current])
        })
        .catch(console.error)
    }
    tick()
    const id = setInterval(tick, 1000)
    return () => clearInterval(id)
  }, [staticInfo])

  if (!staticInfo) {
    return (
      <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
        <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
          {Array.from({ length: 2 }).map((_, i) => (
            <section className="rounded-xl border border-border/70 bg-card p-4" key={i}>
              <Skeleton className="mb-3 h-4 w-32" />
              <div className="space-y-2.5">
                <Skeleton className="h-3 w-full" />
                <Skeleton className="h-3 w-4/5" />
              </div>
            </section>
          ))}
        </div>
      </section>
    )
  }

  const { ram } = staticInfo
  const total = ram.totalBytes
  const used = liveInfo?.ramUsedBytes ?? 0
  const pct = usagePct(used, total)

  return (
    <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
      <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
        {/* Static info */}
        <section className="flex flex-col gap-3 rounded-xl border border-border/70 bg-card p-4">
          <h3 className="text-sm font-medium text-foreground">{t('ram.info')}</h3>
          <Row label={t('home.total')} value={formatBytes(total)} />
          {ram.speedMhz != null && (
            <Row label={t('home.speed')} value={`${ram.speedMhz} MHz`} />
          )}
          {ram.totalSlots > 0 && (
            <Row label={t('home.slots')} value={`${ram.usedSlots} / ${ram.totalSlots}`} />
          )}
        </section>

        {/* Live usage + chart */}
        <section className="flex flex-col gap-3 rounded-xl border border-border/70 bg-card p-4">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-medium text-foreground">{t('home.usage')}</h3>
            <span className="text-sm font-semibold tabular-nums text-foreground">
              {t('home.usedOf', { used: formatBytes(used), total: formatBytes(total) })}
            </span>
          </div>
          <Progress className="h-1.5" value={pct} />
          <LiveChart data={history} height={96} unit="%" yDomain={[0, 100]} />
        </section>
      </div>
    </section>
  )
}
