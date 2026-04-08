import type { ReactNode } from 'react'
import type { StaticSystemInfo } from '@/entities/system-info/model/types'
import type { ChartPoint } from '@/shared/ui/live-chart'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { getStaticSystemInfo } from '@/entities/system-info/api'
import { useLiveCpu } from '@/entities/system-info/model/live-system-store'
import { useMountEffect } from '@/shared/lib/hooks/use-mount-effect'
import { Button } from '@/shared/ui/button'
import { LiveChart } from '@/shared/ui/live-chart'
import { Progress } from '@/shared/ui/progress'
import { Skeleton } from '@/shared/ui/skeleton'

function formatMhz(mhz: number): string {
  return mhz >= 1000 ? `${(mhz / 1000).toFixed(2)} GHz` : `${mhz} MHz`
}

function formatCache(kb: number): string {
  return kb >= 1024 ? `${(kb / 1024).toFixed(0)} MB` : `${kb} KB`
}

function formatUptime(secs: number): string {
  const d = Math.floor(secs / 86400)
  const h = Math.floor((secs % 86400) / 3600)
  const m = Math.floor((secs % 3600) / 60)
  const s = secs % 60
  const dd = String(d).padStart(2, '0')
  const hh = String(h).padStart(2, '0')
  const mm = String(m).padStart(2, '0')
  const ss = String(s).padStart(2, '0')
  return `${dd}:${hh}:${mm}:${ss}`
}

interface RowProps {
  label: string
  value: ReactNode
}

function Row({ label, value }: RowProps) {
  return (
    <div className="flex items-center justify-between gap-4">
      <span className="text-xs text-muted-foreground">{label}</span>
      <span className="text-right text-xs font-medium">{value}</span>
    </div>
  )
}

export function CpuPage() {
  const { t } = useTranslation()
  const [staticInfo, setStaticInfo] = useState<StaticSystemInfo | null>(null)
  const [staticInfoError, setStaticInfoError] = useState(false)
  const { data: liveInfo, history: rawHistory } = useLiveCpu()
  const history: ChartPoint[] = rawHistory.map(v => ({ value: v }))

  const loadStaticInfo = () => {
    setStaticInfoError(false)
    getStaticSystemInfo()
      .then((info) => {
        setStaticInfo(info)
      })
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
          <section className="flex flex-col gap-3 rounded-lg border border-border/70 bg-card p-4">
            <p className="text-sm text-muted-foreground">{t('cpu.loadError')}</p>
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
        {Array.from({ length: 3 }).map((_, i) => (
          <section className="rounded-lg border border-border/70 bg-card p-4" key={i}>
            <Skeleton className="mb-3 h-4 w-32" />
            <div className="space-y-2.5">
              <Skeleton className="h-3 w-full" />
              <Skeleton className="h-3 w-4/5" />
              <Skeleton className="h-3 w-3/5" />
            </div>
          </section>
        ))}
      </section>
    )
  }

  const cpu = staticInfo.cpu

  return (
    <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
      {/* Live chart */}
      <section className="flex flex-col gap-1 rounded-lg border border-border/70 bg-card p-4">
        <div className="flex items-baseline justify-between">
          <span className="text-xs font-medium text-foreground">{t('home.usage')}</span>
          <span className="text-xs tabular-nums text-muted-foreground">100%</span>
        </div>
        <LiveChart data={history} height={96} unit="%" yDomain={[0, 100]} />
        <div className="flex items-baseline justify-between">
          <span className="text-xs text-muted-foreground">{t('ram.seconds', { n: 60 })}</span>
          <span className="text-xs tabular-nums text-muted-foreground">0</span>
        </div>
      </section>

      {/* CPU info */}
      <section className="flex flex-col gap-3 rounded-lg border border-border/70 bg-card p-4">
        <h3 className="text-sm font-medium text-foreground">{t('cpu.info')}</h3>
        <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
          <Row
            label={t('home.model')}
            value={<span className="text-foreground">{cpu.model}</span>}
          />
          <Row
            label={t('cpu.currentFreq')}
            value={<span className="tabular-nums text-[var(--metric-accent)]">{liveInfo ? formatMhz(liveInfo.cpuCurrentFreqMhz) : '—'}</span>}
          />
          <Row
            label={t('cpu.processes')}
            value={<span className="tabular-nums text-[var(--metric-accent)]">{liveInfo ? liveInfo.cpuProcessCount.toLocaleString() : '—'}</span>}
          />
          <Row
            label={t('cpu.threads')}
            value={<span className="tabular-nums text-[var(--metric-accent)]">{liveInfo ? liveInfo.cpuThreadCount.toLocaleString() : '—'}</span>}
          />
          <Row
            label={t('cpu.handles')}
            value={<span className="tabular-nums text-[var(--metric-accent)]">{liveInfo ? liveInfo.cpuHandleCount.toLocaleString() : '—'}</span>}
          />
          <Row
            label={t('cpu.uptime')}
            value={<span className="tabular-nums text-[var(--metric-accent)]">{liveInfo ? formatUptime(liveInfo.cpuUptimeSecs) : '—'}</span>}
          />
          <Row
            label={t('home.baseFreq')}
            value={<span className="text-foreground">{formatMhz(cpu.baseFreqMhz)}</span>}
          />
          <Row
            label={t('cpu.sockets')}
            value={<span className="text-foreground">{cpu.sockets}</span>}
          />
          <Row
            label={t('home.cores')}
            value={<span className="text-foreground">{cpu.physicalCores}</span>}
          />
          <Row
            label={t('cpu.logicalCores')}
            value={<span className="text-foreground">{cpu.logicalCores}</span>}
          />
          <Row
            label={t('cpu.virtualization')}
            value={(
              <span className={cpu.virtualization ? 'text-[var(--metric-good)]' : 'text-muted-foreground'}>
                {cpu.virtualization ? t('cpu.virtualizationOn') : t('cpu.virtualizationOff')}
              </span>
            )}
          />
          {cpu.l1CacheKb != null && (
            <Row
              label={t('cpu.l1Cache')}
              value={<span className="text-foreground">{formatCache(cpu.l1CacheKb)}</span>}
            />
          )}
          {cpu.l2CacheKb != null && (
            <Row
              label={t('cpu.l2Cache')}
              value={<span className="text-foreground">{formatCache(cpu.l2CacheKb)}</span>}
            />
          )}
          {cpu.l3CacheKb != null && (
            <Row
              label={t('cpu.l3Cache')}
              value={<span className="text-foreground">{formatCache(cpu.l3CacheKb)}</span>}
            />
          )}
        </div>
      </section>

      {/* Per-core usage */}
      {liveInfo && (
        <section className="flex flex-col gap-3 rounded-lg border border-border/70 bg-card p-4">
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
    </section>
  )
}
