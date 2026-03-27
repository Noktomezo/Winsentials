import type { ReactNode } from 'react'
import type { GpuInfo, LiveGpuInfo, StaticSystemInfo } from '@/entities/system-info/model/types'
import type { ChartPoint } from '@/shared/ui/live-chart'
import { Navigate, useParams } from '@tanstack/react-router'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { getStaticSystemInfo } from '@/entities/system-info/api'
import { useLiveGpu } from '@/entities/system-info/model/live-system-store'
import { useMountEffect } from '@/shared/lib/hooks/use-mount-effect'
import { Button, Skeleton } from '@/shared/ui'
import { LiveChart } from '@/shared/ui/live-chart'

interface EngineChartProps {
  label: string
  value: number
  data: ChartPoint[]
}

interface MemChartProps {
  label: string
  valueLabel: string
  data: ChartPoint[]
  unit?: string
  yDomain?: [number, number]
}

interface LiveGpuErrorStateProps {
  message: string
  onRetry: () => void
}

function gpuUsage(gpu: Pick<LiveGpuInfo, 'util3d' | 'utilCopy' | 'utilEncode' | 'utilDecode' | 'utilHighPriority3d' | 'utilHighPriorityCompute'>): number {
  return Math.max(
    gpu.util3d,
    gpu.utilCopy,
    gpu.utilEncode,
    gpu.utilDecode,
    gpu.utilHighPriority3d,
    gpu.utilHighPriorityCompute,
  )
}

function formatMb(mb: number, t: ReturnType<typeof useTranslation>['t']): string {
  if (mb === 0) { return `0 ${t('format.megabyte')}` }
  if (mb >= 1024) { return `${(mb / 1024).toFixed(1)} ${t('format.gigabyte')}` }
  return `${mb} ${t('format.megabyte')}`
}

function formatMbPair(used: number, total: number, t: ReturnType<typeof useTranslation>['t']): string {
  const useGb = total >= 1024
  if (useGb) {
    return `${(used / 1024).toFixed(1)} / ${(total / 1024).toFixed(1)} ${t('format.gigabyte')}`
  }
  return `${used} / ${total} ${t('format.megabyte')}`
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

function loadColor(pct: number): string {
  if (pct >= 85) { return 'text-destructive' }
  if (pct >= 60) { return 'text-warning' }
  return 'text-success'
}

function tempColorClass(temp: number): string {
  if (temp >= 80) { return 'text-destructive' }
  if (temp >= 60) { return 'text-warning' }
  return 'text-success'
}

function EngineChart({ label, value, data }: EngineChartProps) {
  const { t } = useTranslation()

  return (
    <section className="flex flex-col gap-2 rounded-xl border border-border/70 bg-card p-4">
      <div className="flex items-center justify-between">
        <h3 className="text-xs font-medium text-muted-foreground">{label}</h3>
        <span className="text-xs font-semibold tabular-nums text-foreground">
          {value}
          %
        </span>
      </div>
      <LiveChart data={data} height={64} unit="%" yDomain={[0, 100]} />
      <div className="flex items-baseline justify-between">
        <span className="text-xs text-muted-foreground">{t('ram.seconds', { n: 60 })}</span>
        <span className="text-xs tabular-nums text-muted-foreground">0</span>
      </div>
    </section>
  )
}

function MemChart({ label, valueLabel, data, unit = '%', yDomain }: MemChartProps) {
  const { t } = useTranslation()

  return (
    <section className="col-span-2 flex flex-col gap-2 rounded-xl border border-border/70 bg-card p-4">
      <div className="flex items-center justify-between">
        <h3 className="text-xs font-medium text-muted-foreground">{label}</h3>
        <span className="text-xs font-semibold tabular-nums text-foreground">{valueLabel}</span>
      </div>
      <LiveChart data={data} height={64} unit={unit} yDomain={yDomain} />
      <div className="flex items-baseline justify-between">
        <span className="text-xs text-muted-foreground">{t('ram.seconds', { n: 60 })}</span>
        <span className="text-xs tabular-nums text-muted-foreground">0</span>
      </div>
    </section>
  )
}

function getEngineCharts(
  gpu: GpuInfo,
  live: LiveGpuInfo | undefined,
  history: {
    threeD: ChartPoint[]
    copy: ChartPoint[]
    encode: ChartPoint[]
    decode: ChartPoint[]
    highPriority3d: ChartPoint[]
    highPriorityCompute: ChartPoint[]
  },
  t: (key: string) => string,
) {
  if (gpu.isIntegrated) {
    return [
      { key: '3d', label: t('gpu.engine3D'), value: live?.util3d ?? 0, data: history.threeD },
      { key: 'copy', label: t('gpu.engineCopy'), value: live?.utilCopy ?? 0, data: history.copy },
      { key: 'hp3d', label: t('gpu.engineHP3D'), value: live?.utilHighPriority3d ?? 0, data: history.highPriority3d },
      { key: 'hpcompute', label: t('gpu.engineHPCompute'), value: live?.utilHighPriorityCompute ?? 0, data: history.highPriorityCompute },
    ]
  }

  return [
    { key: '3d', label: t('gpu.engine3D'), value: live?.util3d ?? 0, data: history.threeD },
    { key: 'copy', label: t('gpu.engineCopy'), value: live?.utilCopy ?? 0, data: history.copy },
    { key: 'encode', label: t('gpu.engineVideoEncode'), value: live?.utilEncode ?? 0, data: history.encode },
    { key: 'decode', label: t('gpu.engineVideoDecode'), value: live?.utilDecode ?? 0, data: history.decode },
  ]
}

function LiveGpuLoadingState() {
  return (
    <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
      <div className="grid grid-cols-2 gap-4">
        {Array.from({ length: 4 }).map((_, i) => (
          <section className="rounded-xl border border-border/70 bg-card p-4" key={i}>
            <Skeleton className="mb-3 h-3 w-24" />
            <Skeleton className="h-16 w-full" />
          </section>
        ))}
        <section className="col-span-2 rounded-xl border border-border/70 bg-card p-4">
          <div className="space-y-2.5">
            {Array.from({ length: 8 }).map((_, i) => (
              <Skeleton className="h-3 w-full" key={i} />
            ))}
          </div>
        </section>
      </div>
    </section>
  )
}

function LiveGpuErrorState({ message, onRetry }: LiveGpuErrorStateProps) {
  const { t } = useTranslation()

  return (
    <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
      <section className="flex flex-col gap-3 rounded-xl border border-border/70 bg-card p-4">
        <p className="text-sm text-muted-foreground">{message}</p>
        <div>
          <Button onClick={onRetry} size="sm" type="button" variant="outline">
            {t('tweaks.actions.retry')}
          </Button>
        </div>
      </section>
    </section>
  )
}

export function GpuPage() {
  const { t } = useTranslation()
  const params = useParams({ strict: false })
  const parsedGpuIndex = params.gpuIndex !== undefined ? Number(params.gpuIndex) : null

  const [staticInfo, setStaticInfo] = useState<StaticSystemInfo | null>(null)
  const [staticError, setStaticError] = useState(false)
  const { data: liveInfo, error: liveError, history: gpuHistory, isFetching, retry } = useLiveGpu()

  const loadStaticInfo = () => {
    setStaticError(false)
    getStaticSystemInfo()
      .then(setStaticInfo)
      .catch((error) => {
        console.error(error)
        setStaticError(true)
      })
  }

  useMountEffect(() => {
    loadStaticInfo()
  })

  const gpuIndex = staticInfo && parsedGpuIndex !== null && Number.isInteger(parsedGpuIndex) && parsedGpuIndex >= 0 && parsedGpuIndex < staticInfo.gpus.length
    ? parsedGpuIndex
    : null
  const gpu = staticInfo && gpuIndex !== null ? staticInfo.gpus[gpuIndex] : null
  const isDetailView = gpuIndex !== null && gpu != null
  const liveByIndex = Object.fromEntries((liveInfo ?? []).map(sample => [sample.index, sample]))
  const historyByIndex = gpuHistory

  if (staticInfo && params.gpuIndex !== undefined && gpuIndex === null) {
    return <Navigate replace to="/gpu" />
  }

  if (!staticInfo) {
    if (staticError) {
      return (
        <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
          <section className="flex flex-col gap-3 rounded-xl border border-border/70 bg-card p-4">
            <p className="text-sm text-muted-foreground">{t('gpu.loadError')}</p>
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
        <div className="grid grid-cols-2 gap-4">
          {Array.from({ length: 4 }).map((_, i) => (
            <section className="rounded-xl border border-border/70 bg-card p-4" key={i}>
              <Skeleton className="mb-3 h-3 w-24" />
              <Skeleton className="h-16 w-full" />
            </section>
          ))}
          <section className="col-span-2 rounded-xl border border-border/70 bg-card p-4">
            <div className="space-y-2.5">
              {Array.from({ length: 8 }).map((_, i) => (
                <Skeleton className="h-3 w-full" key={i} />
              ))}
            </div>
          </section>
        </div>
      </section>
    )
  }

  // ── Detail view ──────────────────────────────────────────────────────────────
  if (isDetailView && gpu && gpuIndex !== null) {
    if (liveInfo === null && isFetching) {
      return <LiveGpuLoadingState />
    }

    if (liveInfo === null && liveError) {
      return <LiveGpuErrorState message={t('gpu.liveLoadError')} onRetry={retry} />
    }

    const live = liveByIndex[gpuIndex]
    if (!live) {
      return (
        <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
          <section className="rounded-xl border border-border/70 bg-card p-4">
            <p className="text-sm text-muted-foreground">{t('gpu.noLiveData')}</p>
          </section>
        </section>
      )
    }

    const dedicatedBudgetMb = live ? live.vramTotalMb - live.vramReservedMb : 0
    const dedicatedUsedMb = live?.vramUsedMb ?? 0
    const sharedUsedMb = live?.vramSharedMb ?? 0

    const usage = live ? gpuUsage(live) : 0
    const gpuHist = historyByIndex[gpuIndex]
    const hist3D = (gpuHist?.threeD ?? []).map((v: number) => ({ value: v }))
    const histCopy = (gpuHist?.copy ?? []).map((v: number) => ({ value: v }))
    const histEncode = (gpuHist?.encode ?? []).map((v: number) => ({ value: v }))
    const histDecode = (gpuHist?.decode ?? []).map((v: number) => ({ value: v }))
    const histHP3D = (gpuHist?.highPriority3d ?? []).map((v: number) => ({ value: v }))
    const histHPCompute = (gpuHist?.highPriorityCompute ?? []).map((v: number) => ({ value: v }))
    const histDedicated = (gpuHist?.dedicatedPct ?? []).map((v: number) => ({ value: v }))
    const histShared = (gpuHist?.sharedMb ?? []).map((v: number) => ({ value: v }))
    const engineCharts = getEngineCharts(gpu, live, {
      threeD: hist3D,
      copy: histCopy,
      encode: histEncode,
      decode: histDecode,
      highPriority3d: histHP3D,
      highPriorityCompute: histHPCompute,
    }, t)

    // PCI location string
    const pciParts: string[] = []
    if (gpu.pciBus != null) { pciParts.push(`${t('gpu.pciBus')} ${gpu.pciBus}`) }
    if (gpu.pciDevice != null) { pciParts.push(`${t('gpu.pciDevice')} ${gpu.pciDevice}`) }
    if (gpu.pciFunction != null) { pciParts.push(`${t('gpu.pciFunction')} ${gpu.pciFunction}`) }
    const pciString = pciParts.length > 0 ? pciParts.join(', ') : null

    return (
      <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
        {/* Engine charts 2×2 */}
        <div className="grid grid-cols-2 gap-4">
          {engineCharts.map(engine => (
            <EngineChart data={engine.data} key={engine.key} label={engine.label} value={engine.value} />
          ))}

          {/* Dedicated memory — full width */}
          {dedicatedBudgetMb > 0 && (
            <MemChart
              data={histDedicated}
              label={t('gpu.dedicated')}
              valueLabel={formatMbPair(dedicatedUsedMb, dedicatedBudgetMb, t)}
              yDomain={[0, 100]}
            />
          )}

          {/* Shared memory — full width */}
          {sharedUsedMb > 0 && (
            <MemChart
              data={histShared}
              label={t('gpu.shared')}
              unit={` ${t('format.megabyte')}`}
              valueLabel={formatMb(sharedUsedMb, t)}
            />
          )}
        </div>

        {/* Single info card */}
        <section className="flex flex-col gap-2.5 rounded-xl border border-border/70 bg-card p-4">
          {gpu.vendor && gpu.vendor !== 'Unknown' && (
            <Row label={t('home.vendor')} value={gpu.vendor} />
          )}
          <Row label={t('home.model')} value={gpu.name} />

          {live && (
            <Row
              label={t('gpu.load')}
              value={(
                <span className={loadColor(usage)}>
                  {usage}
                  {' '}
                  %
                </span>
              )}
            />
          )}

          {gpu.vramTotalMb > 0 && (
            <Row label={t('gpu.totalRam')} value={formatMbPair(live?.vramUsedMb ?? 0, gpu.vramTotalMb, t)} />
          )}

          {dedicatedBudgetMb > 0 && (
            <Row label={t('gpu.dedicated')} value={formatMbPair(dedicatedUsedMb, dedicatedBudgetMb, t)} />
          )}

          {(live?.vramSharedMb ?? 0) > 0 && (
            <Row label={t('gpu.shared')} value={formatMb(live?.vramSharedMb ?? 0, t)} />
          )}

          {live?.temperatureC != null && (
            <Row
              label={t('gpu.temperature')}
              value={(
                <span className={tempColorClass(live.temperatureC)}>
                  {live.temperatureC}
                  {' '}
                  {t('format.temperatureUnit')}
                </span>
              )}
            />
          )}

          {gpu.driverVersion && (
            <Row label={t('gpu.driverVersion')} value={gpu.driverVersion} />
          )}

          {gpu.driverDate && (
            <Row label={t('gpu.driverDate')} value={gpu.driverDate} />
          )}

          {gpu.directxVersion && (
            <Row label={t('gpu.directx')} value={gpu.directxVersion} />
          )}

          {pciString && (
            <Row label={t('gpu.pciLocation')} value={pciString} />
          )}
        </section>
      </section>
    )
  }

  // ── Overview (all GPUs, no index selected) ───────────────────────────────────
  const gpusToShow = gpuIndex !== null
    ? staticInfo.gpus[gpuIndex] ? [{ gpu: staticInfo.gpus[gpuIndex], idx: gpuIndex }] : []
    : staticInfo.gpus.map((g, idx) => ({ gpu: g, idx }))

  if (liveInfo === null && isFetching) {
    return <LiveGpuLoadingState />
  }

  if (liveInfo === null && liveError) {
    return <LiveGpuErrorState message={t('gpu.liveLoadError')} onRetry={retry} />
  }

  const hasAnyLiveData = liveInfo?.some(g => gpuUsage(g) > 0 || g.temperatureC != null) ?? false

  return (
    <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
      <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
        {gpusToShow.map(({ gpu: g, idx }) => {
          const gpuLive = liveByIndex[idx]
          const liveUsage = gpuLive ? gpuUsage(gpuLive) : 0
          return (
            <section className="flex flex-col gap-3 rounded-xl border border-border/70 bg-card p-4" key={idx}>
              {gpuIndex === null && (
                <h3 className="text-sm font-medium text-foreground">
                  {t('gpu.adapter')}
                  {staticInfo.gpus.length > 1 ? ` ${idx}` : ''}
                </h3>
              )}
              <Row label={t('home.model')} value={g.name} />
              {g.vendor && g.vendor !== 'Unknown' && (
                <Row label={t('home.vendor')} value={g.vendor} />
              )}
              {g.vramTotalMb > 0 && (
                <Row label={t('home.vram')} value={formatMb(g.vramTotalMb, t)} />
              )}
              {gpuLive && (
                <Row
                  label={t('gpu.load')}
                  value={(
                    <span className={loadColor(liveUsage)}>
                      {liveUsage}
                      {' '}
                      %
                    </span>
                  )}
                />
              )}
              {gpuLive?.temperatureC != null && (
                <Row label={t('gpu.temperature')} value={`${gpuLive.temperatureC} ${t('format.temperatureUnit')}`} />
              )}
            </section>
          )
        })}

        {!hasAnyLiveData && liveInfo && (
          <p className="col-span-2 text-xs text-muted-foreground">{t('gpu.noLiveData')}</p>
        )}
      </div>
    </section>
  )
}
