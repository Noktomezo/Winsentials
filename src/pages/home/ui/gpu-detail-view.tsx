import type { TFunction } from 'i18next'
import type { ReactNode } from 'react'
import type { useLiveGpu } from '@/entities/system-info/model/live-system-store'
import type {
  GpuInfo,
  LiveGpuInfo,
} from '@/entities/system-info/model/types'
import type { ChartPoint } from '@/shared/ui/live-chart'
import { useTranslation } from 'react-i18next'
import { LiveChart } from '@/shared/ui/live-chart'
import {
  formatMbPair,
  getEngineCharts,
  gpuUsage,
  loadColor,
  memoryColorClass,
  tempColorClass,
} from './gpu-helpers'

interface GpuDetailViewProps {
  gpu: GpuInfo
  gpuIndex: number
  live: LiveGpuInfo | undefined
  historyByIndex: ReturnType<typeof useLiveGpu>['history']
  t: TFunction
}

interface RowProps {
  label: string
  value: ReactNode
}

function Row({ label, value }: RowProps) {
  return (
    <div className="flex items-center justify-between gap-4">
      <span className="text-xs text-muted-foreground">{label}</span>
      <span className="text-right text-xs font-medium text-foreground">
        {value}
      </span>
    </div>
  )
}

interface EngineChartProps {
  label: string
  value: number
  data: ChartPoint[]
}

function EngineChart({ label, value, data }: EngineChartProps) {
  const { t } = useTranslation()

  return (
    <section className="flex flex-col gap-2 rounded-lg border border-border/70 bg-card p-4">
      <div className="flex items-center justify-between">
        <h3 className="text-xs font-medium text-muted-foreground">{label}</h3>
        <span className="text-xs font-semibold tabular-nums text-muted-foreground">
          {value}
          %
        </span>
      </div>
      <LiveChart data={data} height={64} unit="%" yDomain={[0, 100]} />
      <div className="flex items-baseline justify-between">
        <span className="text-xs text-muted-foreground">
          {t('ram.seconds', { n: 60 })}
        </span>
        <span className="text-xs tabular-nums text-muted-foreground">0</span>
      </div>
    </section>
  )
}

interface MemChartProps {
  label: string
  valueLabel: string
  data: ChartPoint[]
  unit?: string
  yDomain?: [number, number]
}

function MemChart({
  label,
  valueLabel,
  data,
  unit = '%',
  yDomain,
}: MemChartProps) {
  const { t } = useTranslation()

  return (
    <section className="col-span-2 flex flex-col gap-2 rounded-lg border border-border/70 bg-card p-4">
      <div className="flex items-center justify-between">
        <h3 className="text-xs font-medium text-muted-foreground">{label}</h3>
        <span className="text-xs font-semibold tabular-nums text-muted-foreground">
          {valueLabel}
        </span>
      </div>
      <LiveChart data={data} height={64} unit={unit} yDomain={yDomain} />
      <div className="flex items-baseline justify-between">
        <span className="text-xs text-muted-foreground">
          {t('ram.seconds', { n: 60 })}
        </span>
        <span className="text-xs tabular-nums text-muted-foreground">0</span>
      </div>
    </section>
  )
}

export function GpuDetailView({
  gpu,
  gpuIndex,
  live,
  historyByIndex,
  t,
}: GpuDetailViewProps) {
  const totalMemoryMb = gpu.dedicatedVramMb + gpu.sharedSystemMb
  const dedicatedBudgetMb = gpu.dedicatedVramMb
  const dedicatedUsedMb = live?.vramUsedMb ?? 0
  const sharedUsedMb = live?.vramSharedMb ?? 0
  const totalUsedMb = dedicatedUsedMb + sharedUsedMb

  const usage = live ? gpuUsage(live) : 0
  const gpuHist = historyByIndex[gpuIndex]
  const hist3D = (gpuHist?.threeD ?? []).map((v: number) => ({ value: v }))
  const histCopy = (gpuHist?.copy ?? []).map((v: number) => ({ value: v }))
  const histEncode = (gpuHist?.encode ?? []).map((v: number) => ({
    value: v,
  }))
  const histDecode = (gpuHist?.decode ?? []).map((v: number) => ({
    value: v,
  }))
  const histHP3D = (gpuHist?.highPriority3d ?? []).map((v: number) => ({
    value: v,
  }))
  const histHPCompute = (gpuHist?.highPriorityCompute ?? []).map(
    (v: number) => ({ value: v }),
  )
  const histDedicated = (gpuHist?.dedicatedPct ?? []).map((v: number) => ({
    value: v,
  }))
  const histShared = (gpuHist?.sharedMb ?? []).map((v: number) => ({
    value: v,
  }))
  const engineCharts = getEngineCharts(
    gpu,
    live,
    {
      threeD: hist3D,
      copy: histCopy,
      encode: histEncode,
      decode: histDecode,
      highPriority3d: histHP3D,
      highPriorityCompute: histHPCompute,
    },
    t,
  )

  const pciParts: string[] = []
  if (gpu.pciBus != null) {
    pciParts.push(`${t('gpu.pciBus')} ${gpu.pciBus}`)
  }
  if (gpu.pciDevice != null) {
    pciParts.push(`${t('gpu.pciDevice')} ${gpu.pciDevice}`)
  }
  if (gpu.pciFunction != null) {
    pciParts.push(`${t('gpu.pciFunction')} ${gpu.pciFunction}`)
  }
  const pciString = pciParts.length > 0 ? pciParts.join(', ') : null

  return (
    <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
      {/* Engine charts 2×2 */}
      <div className="grid grid-cols-2 gap-4">
        {engineCharts.map(engine => (
          <EngineChart
            data={engine.data}
            key={engine.key}
            label={engine.label}
            value={engine.value}
          />
        ))}
      </div>

      <div className="flex flex-col gap-4">
        {dedicatedBudgetMb > 0 && (
          <MemChart
            data={histDedicated}
            label={t('gpu.dedicated')}
            valueLabel={formatMbPair(dedicatedUsedMb, dedicatedBudgetMb, t)}
            yDomain={[0, 100]}
          />
        )}

        {gpu.sharedSystemMb > 0 && (
          <MemChart
            data={histShared}
            label={t('gpu.shared')}
            unit={` ${t('format.megabyte')}`}
            valueLabel={formatMbPair(sharedUsedMb, gpu.sharedSystemMb, t)}
          />
        )}
      </div>

      {/* Single info card */}
      <section className="flex flex-col gap-3 rounded-lg border border-border/70 bg-card p-4">
        <h3 className="text-sm font-medium text-foreground">{t('gpu.info')}</h3>
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

        {totalMemoryMb > 0 && (
          <Row
            label={t('gpu.totalRam')}
            value={<span className={memoryColorClass(totalUsedMb, totalMemoryMb)}>{formatMbPair(totalUsedMb, totalMemoryMb, t)}</span>}
          />
        )}

        {dedicatedBudgetMb > 0 && (
          <Row
            label={t('gpu.dedicated')}
            value={<span className={memoryColorClass(dedicatedUsedMb, dedicatedBudgetMb)}>{formatMbPair(dedicatedUsedMb, dedicatedBudgetMb, t)}</span>}
          />
        )}

        {gpu.sharedSystemMb > 0 && (
          <Row
            label={t('gpu.shared')}
            value={<span className={memoryColorClass(sharedUsedMb, gpu.sharedSystemMb)}>{formatMbPair(sharedUsedMb, gpu.sharedSystemMb, t)}</span>}
          />
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

        {pciString && <Row label={t('gpu.pciLocation')} value={pciString} />}
      </section>
    </section>
  )
}
