import type { LiveSystemInfo } from '@/entities/system-info/model/types'
import type { ChartPoint } from '@/shared/ui/live-chart'
import { useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { getLiveSystemInfo, getStaticSystemInfo } from '@/entities/system-info/api'
import { LiveChart } from '@/shared/ui/live-chart'
import { Skeleton } from '@/shared/ui/skeleton'

function formatBytes(bytes: number, decimals = 1): string {
  if (bytes === 0) { return '0 B' }
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return `${Number.parseFloat((bytes / k ** i).toFixed(decimals))} ${sizes[i]}`
}

export function NetworkStatsPage() {
  const { t } = useTranslation()
  const [ready, setReady] = useState(false)
  const [liveInfo, setLiveInfo] = useState<LiveSystemInfo | null>(null)
  const [rxHistory, setRxHistory] = useState<ChartPoint[]>([])
  const [txHistory, setTxHistory] = useState<ChartPoint[]>([])
  const rxRef = useRef<ChartPoint[]>([])
  const txRef = useRef<ChartPoint[]>([])

  useEffect(() => {
    // Just need static info to prime the backend cache before live polling
    getStaticSystemInfo()
      .then(() => setReady(true))
      .catch(console.error)
  }, [])

  useEffect(() => {
    if (!ready) { return }
    const tick = () => {
      getLiveSystemInfo()
        .then((live) => {
          setLiveInfo(live)
          const totalRx = live.network.reduce((s, i) => s + i.rxBytesPerSec, 0)
          const totalTx = live.network.reduce((s, i) => s + i.txBytesPerSec, 0)

          const nextRx = [...rxRef.current, { value: totalRx }]
          rxRef.current = nextRx.length > 60 ? nextRx.slice(-60) : nextRx
          setRxHistory([...rxRef.current])

          const nextTx = [...txRef.current, { value: totalTx }]
          txRef.current = nextTx.length > 60 ? nextTx.slice(-60) : nextTx
          setTxHistory([...txRef.current])
        })
        .catch(console.error)
    }
    tick()
    const id = setInterval(tick, 1000)
    return () => clearInterval(id)
  }, [ready])

  const ifaces = liveInfo?.network ?? []

  if (!ready) {
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

  return (
    <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
      <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
        {/* Download chart */}
        <section className="flex flex-col gap-3 rounded-xl border border-border/70 bg-card p-4">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-medium text-foreground">{t('networkStats.download')}</h3>
            <span className="text-sm font-semibold tabular-nums text-foreground">
              {formatBytes(liveInfo?.network.reduce((s, i) => s + i.rxBytesPerSec, 0) ?? 0)}
              /s
            </span>
          </div>
          <LiveChart data={rxHistory} height={96} />
        </section>

        {/* Upload chart */}
        <section className="flex flex-col gap-3 rounded-xl border border-border/70 bg-card p-4">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-medium text-foreground">{t('networkStats.upload')}</h3>
            <span className="text-sm font-semibold tabular-nums text-foreground">
              {formatBytes(liveInfo?.network.reduce((s, i) => s + i.txBytesPerSec, 0) ?? 0)}
              /s
            </span>
          </div>
          <LiveChart data={txHistory} height={96} />
        </section>

        {/* Per-adapter table */}
        <section className="flex flex-col gap-3 rounded-xl border border-border/70 bg-card p-4 md:col-span-2">
          <h3 className="text-sm font-medium text-foreground">{t('networkStats.adapters')}</h3>
          {ifaces.length === 0
            ? <span className="text-xs text-muted-foreground">{t('home.noActivity')}</span>
            : (
                <div className="space-y-3">
                  {ifaces.map(iface => (
                    <div className="space-y-0.5" key={iface.name}>
                      <span className="text-xs font-medium text-foreground">{iface.name}</span>
                      <div className="flex gap-6">
                        <span className="text-xs tabular-nums text-muted-foreground">
                          ↓
                          {' '}
                          {formatBytes(iface.rxBytesPerSec)}
                          /s
                        </span>
                        <span className="text-xs tabular-nums text-muted-foreground">
                          ↑
                          {' '}
                          {formatBytes(iface.txBytesPerSec)}
                          /s
                        </span>
                      </div>
                    </div>
                  ))}
                </div>
              )}
        </section>
      </div>
    </section>
  )
}
