import type { GpuInfo, LiveSystemInfo, StaticSystemInfo } from '@/entities/system-info/model/types'
import type { ChartPoint } from '@/shared/ui/live-chart'
import { useParams } from '@tanstack/react-router'
import { useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { getLiveSystemInfo, getStaticSystemInfo } from '@/entities/system-info/api'
import { LiveChart } from '@/shared/ui/live-chart'
import { Skeleton } from '@/shared/ui/skeleton'

function gpuUsage(gpu: GpuInfo): number {
  return Math.max(
    gpu.util3d,
    gpu.utilCopy,
    gpu.utilEncode,
    gpu.utilDecode,
    gpu.utilHighPriority3d,
    gpu.utilHighPriorityCompute,
  )
}

/** Auto-format MB → GB when >= 1024 */
function formatMb(mb: number): string {
  if (mb === 0) { return '0 MB' }
  if (mb >= 1024) { return `${(mb / 1024).toFixed(1)} GB` }
  return `${mb} MB`
}

function formatMbPair(used: number, total: number): string {
  const useGb = total >= 1024
  if (useGb) {
    return `${(used / 1024).toFixed(1)} / ${(total / 1024).toFixed(1)} GB`
  }
  return `${used} / ${total} MB`
}

function Row({ label, value }: { label: string, value: React.ReactNode }) {
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

function EngineChart({ label, value, data }: { label: string, value: number, data: ChartPoint[] }) {
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
    </section>
  )
}

function MemChart({
  label,
  usedMb,
  totalMb,
  data,
}: {
  label: string
  usedMb: number
  totalMb: number
  data: ChartPoint[]
}) {
  return (
    <section className="col-span-2 flex flex-col gap-2 rounded-xl border border-border/70 bg-card p-4">
      <div className="flex items-center justify-between">
        <h3 className="text-xs font-medium text-muted-foreground">{label}</h3>
        <span className="text-xs font-semibold tabular-nums text-foreground">
          {formatMbPair(usedMb, totalMb)}
        </span>
      </div>
      <LiveChart data={data} height={64} unit="%" yDomain={[0, 100]} />
    </section>
  )
}

function pushHistory(
  ref: React.MutableRefObject<ChartPoint[]>,
  value: number,
  setter: (v: ChartPoint[]) => void,
): void {
  const next = [...ref.current, { value }]
  ref.current = next.length > 60 ? next.slice(-60) : next
  setter([...ref.current])
}

export function GpuPage() {
  const { t } = useTranslation()
  const params = useParams({ strict: false })
  const gpuIndex = params.gpuIndex !== undefined ? Number(params.gpuIndex) : null

  const [staticInfo, setStaticInfo] = useState<StaticSystemInfo | null>(null)
  const [liveInfo, setLiveInfo] = useState<LiveSystemInfo | null>(null)

  const hist3DRef = useRef<ChartPoint[]>([])
  const histCopyRef = useRef<ChartPoint[]>([])
  const histHP3DRef = useRef<ChartPoint[]>([])
  const histHPComputeRef = useRef<ChartPoint[]>([])
  const histDedicatedRef = useRef<ChartPoint[]>([])
  const histSharedRef = useRef<ChartPoint[]>([])

  const [hist3D, setHist3D] = useState<ChartPoint[]>([])
  const [histCopy, setHistCopy] = useState<ChartPoint[]>([])
  const [histHP3D, setHistHP3D] = useState<ChartPoint[]>([])
  const [histHPCompute, setHistHPCompute] = useState<ChartPoint[]>([])
  const [histDedicated, setHistDedicated] = useState<ChartPoint[]>([])
  const [histShared, setHistShared] = useState<ChartPoint[]>([])

  useEffect(() => {
    getStaticSystemInfo().then(setStaticInfo).catch(console.error)
  }, [])

  const gpu = staticInfo && gpuIndex !== null ? staticInfo.gpus[gpuIndex] : null
  const isDetailView = gpuIndex !== null && gpu != null

  useEffect(() => {
    if (!staticInfo) { return }

    // Reset history when the selected GPU changes
    hist3DRef.current = []
    histCopyRef.current = []
    histHP3DRef.current = []
    histHPComputeRef.current = []
    histDedicatedRef.current = []
    histSharedRef.current = []
    setHist3D([])
    setHistCopy([])
    setHistHP3D([])
    setHistHPCompute([])
    setHistDedicated([])
    setHistShared([])

    const tick = () => {
      getLiveSystemInfo()
        .then((live) => {
          setLiveInfo(live)
          if (!isDetailView) { return }

          const idx = gpuIndex ?? 0
          const entry = live.gpus[idx]
          if (!entry) { return }

          pushHistory(hist3DRef, entry.util3d, setHist3D)
          pushHistory(histCopyRef, entry.utilCopy, setHistCopy)
          pushHistory(histHP3DRef, entry.utilHighPriority3d, setHistHP3D)
          pushHistory(histHPComputeRef, entry.utilHighPriorityCompute, setHistHPCompute)

          const dedicatedBudget = entry.vramTotalMb - entry.vramReservedMb
          const dedicatedPct = dedicatedBudget > 0
            ? Math.min(100, (entry.vramUsedMb / dedicatedBudget) * 100)
            : 0
          pushHistory(histDedicatedRef, dedicatedPct, setHistDedicated)

          const sharedBudget = entry.vramSharedMb
          const sharedPct = sharedBudget > 0 ? 100 : 0
          pushHistory(histSharedRef, sharedPct, setHistShared)
        })
        .catch(console.error)
    }

    tick()
    const id = setInterval(tick, 1000)
    return () => clearInterval(id)
  }, [staticInfo, gpuIndex, isDetailView])

  if (!staticInfo) {
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
    const live = liveInfo?.gpus[gpuIndex]

    const dedicatedBudgetMb = live ? live.vramTotalMb - live.vramReservedMb : 0
    const dedicatedUsedMb = live?.vramUsedMb ?? 0
    const sharedUsedMb = live?.vramSharedMb ?? 0

    const usage = live ? gpuUsage(live) : 0

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
          <EngineChart data={hist3D} label={t('gpu.engine3D')} value={live?.util3d ?? 0} />
          <EngineChart data={histCopy} label={t('gpu.engineCopy')} value={live?.utilCopy ?? 0} />
          <EngineChart data={histHP3D} label={t('gpu.engineHP3D')} value={live?.utilHighPriority3d ?? 0} />
          <EngineChart data={histHPCompute} label={t('gpu.engineHPCompute')} value={live?.utilHighPriorityCompute ?? 0} />

          {/* Dedicated memory — full width */}
          {dedicatedBudgetMb > 0 && (
            <MemChart
              data={histDedicated}
              label={t('gpu.dedicated')}
              totalMb={dedicatedBudgetMb}
              usedMb={dedicatedUsedMb}
            />
          )}

          {/* Shared memory — full width */}
          {sharedUsedMb > 0 && (
            <MemChart
              data={histShared}
              label={t('gpu.shared')}
              totalMb={sharedUsedMb}
              usedMb={sharedUsedMb}
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
            <Row label={t('gpu.totalRam')} value={formatMbPair(live?.vramUsedMb ?? 0, gpu.vramTotalMb)} />
          )}

          {dedicatedBudgetMb > 0 && (
            <Row label={t('gpu.dedicated')} value={formatMbPair(dedicatedUsedMb, dedicatedBudgetMb)} />
          )}

          {(live?.vramSharedMb ?? 0) > 0 && (
            <Row label={t('gpu.shared')} value={formatMb(live?.vramSharedMb ?? 0)} />
          )}

          {live?.temperatureC != null && (
            <Row
              label={t('gpu.temperature')}
              value={(
                <span className={tempColorClass(live.temperatureC)}>
                  {live.temperatureC}
                  {' °C'}
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

  const hasAnyLiveData = liveInfo?.gpus.some(g => gpuUsage(g) > 0 || g.temperatureC != null) ?? false

  return (
    <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
      <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
        {gpusToShow.map(({ gpu: g, idx }) => {
          const gpuLive = liveInfo?.gpus[idx]
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
                <Row label={t('home.vram')} value={formatMb(g.vramTotalMb)} />
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
                <Row label={t('gpu.temperature')} value={`${gpuLive.temperatureC} °C`} />
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
