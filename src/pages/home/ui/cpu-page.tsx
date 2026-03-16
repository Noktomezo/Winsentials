import type { LiveSystemInfo, StaticSystemInfo } from '@/entities/system-info/model/types'
import type { ChartPoint } from '@/shared/ui/live-chart'
import { useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { getLiveSystemInfo, getStaticSystemInfo } from '@/entities/system-info/api'
import { LiveChart } from '@/shared/ui/live-chart'
import { Progress } from '@/shared/ui/progress'
import { Skeleton } from '@/shared/ui/skeleton'

function formatMhz(mhz: number): string {
  return mhz >= 1000 ? `${(mhz / 1000).toFixed(2)} GHz` : `${mhz} MHz`
}

function Row({ label, value }: { label: string, value: React.ReactNode }) {
  return (
    <div className="flex items-center justify-between gap-4">
      <span className="text-xs text-muted-foreground">{label}</span>
      <span className="text-right text-xs font-medium text-foreground">{value}</span>
    </div>
  )
}

export function CpuPage() {
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
          const next = [...historyRef.current, { value: live.cpuUsagePercent }]
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
                <Skeleton className="h-3 w-3/5" />
              </div>
            </section>
          ))}
        </div>
      </section>
    )
  }

  const cpu = staticInfo.cpu
  const pct = liveInfo ? Math.round(liveInfo.cpuUsagePercent) : 0

  return (
    <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
      <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
        {/* Static info */}
        <section className="flex flex-col gap-3 rounded-xl border border-border/70 bg-card p-4">
          <h3 className="text-sm font-medium text-foreground">{t('cpu.info')}</h3>
          <Row label={t('home.model')} value={cpu.model} />
          <Row
            label={t('home.cores')}
            value={t('home.coresValue', { physical: cpu.physicalCores, logical: cpu.logicalCores })}
          />
          <Row label={t('home.baseFreq')} value={formatMhz(cpu.baseFreqMhz)} />
        </section>

        {/* Live usage + chart */}
        <section className="flex flex-col gap-3 rounded-xl border border-border/70 bg-card p-4">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-medium text-foreground">{t('home.usage')}</h3>
            <span className="text-sm font-semibold tabular-nums text-foreground">
              {pct}
              %
            </span>
          </div>
          <Progress className="h-1.5" value={pct} />
          <LiveChart data={history} height={96} unit="%" yDomain={[0, 100]} />
        </section>

        {/* Per-core usage */}
        {liveInfo && (
          <section className="flex flex-col gap-3 rounded-xl border border-border/70 bg-card p-4 md:col-span-2">
            <h3 className="text-sm font-medium text-foreground">{t('cpu.perCore')}</h3>
            <div className="grid grid-cols-2 gap-x-6 gap-y-2 sm:grid-cols-4">
              {liveInfo.cpuPerCore.map((usage, i) => (
                <div className="space-y-0.5" key={i}>
                  <div className="flex justify-between">
                    <span className="text-xs text-muted-foreground">
                      {t('cpu.core')}
                      {' '}
                      {i + 1}
                    </span>
                    <span className="text-xs tabular-nums text-foreground">
                      {Math.round(usage)}
                      %
                    </span>
                  </div>
                  <Progress className="h-1" value={usage} />
                </div>
              ))}
            </div>
          </section>
        )}
      </div>
    </section>
  )
}
