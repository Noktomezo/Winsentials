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

function Row({ label, value }: { label: string, value: React.ReactNode }) {
  return (
    <div className="flex items-center justify-between gap-4">
      <span className="text-xs text-muted-foreground">{label}</span>
      <span className="text-right text-xs font-medium text-foreground">{value}</span>
    </div>
  )
}

export function GpuPage() {
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
          const usage = live.gpuLive[0]?.usagePercent ?? 0
          const next = [...historyRef.current, { value: usage }]
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
          <section className="rounded-xl border border-border/70 bg-card p-4">
            <Skeleton className="mb-3 h-4 w-32" />
            <div className="space-y-2.5">
              <Skeleton className="h-3 w-full" />
              <Skeleton className="h-3 w-4/5" />
            </div>
          </section>
        </div>
      </section>
    )
  }

  const hasLiveData = (liveInfo?.gpuLive.length ?? 0) > 0
    && liveInfo?.gpuLive.some(g => g.usagePercent != null)

  return (
    <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
      <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
        {staticInfo.gpus.map((gpu, idx) => {
          const gpuLive = liveInfo?.gpuLive[idx]
          return (
            <section className="flex flex-col gap-3 rounded-xl border border-border/70 bg-card p-4" key={idx}>
              <h3 className="text-sm font-medium text-foreground">
                {t('gpu.adapter')}
                {staticInfo.gpus.length > 1 ? ` ${idx + 1}` : ''}
              </h3>
              <Row label={t('home.model')} value={gpu.model} />
              {gpu.vendor && gpu.vendor !== 'Unknown' && (
                <Row label={t('home.vendor')} value={gpu.vendor} />
              )}
              {gpu.vramBytes != null && gpu.vramBytes > 0 && (
                <Row label={t('home.vram')} value={formatBytes(gpu.vramBytes)} />
              )}
              {gpuLive?.usagePercent != null && (
                <div className="space-y-0.5">
                  <div className="flex justify-between">
                    <span className="text-xs text-muted-foreground">{t('gpu.load')}</span>
                    <span className="text-xs font-medium tabular-nums text-foreground">
                      {Math.round(gpuLive.usagePercent)}
                      %
                    </span>
                  </div>
                  <Progress className="h-1.5" value={gpuLive.usagePercent} />
                </div>
              )}
              {gpuLive?.temperatureCelsius != null && (
                <Row
                  label={t('gpu.temperature')}
                  value={`${Math.round(gpuLive.temperatureCelsius)} °C`}
                />
              )}
            </section>
          )
        })}

        {/* Chart — only shown if NVML data is available */}
        {hasLiveData && (
          <section className="flex flex-col gap-3 rounded-xl border border-border/70 bg-card p-4 md:col-span-2">
            <div className="flex items-center justify-between">
              <h3 className="text-sm font-medium text-foreground">{t('gpu.load')}</h3>
              <span className="text-sm font-semibold tabular-nums text-foreground">
                {Math.round(liveInfo?.gpuLive[0]?.usagePercent ?? 0)}
                %
              </span>
            </div>
            <LiveChart data={history} height={96} unit="%" yDomain={[0, 100]} />
          </section>
        )}

        {!hasLiveData && liveInfo && (
          <p className="col-span-2 text-xs text-muted-foreground">{t('gpu.noLiveData')}</p>
        )}
      </div>
    </section>
  )
}
