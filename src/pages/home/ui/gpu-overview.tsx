import type { TFunction } from 'i18next'
import type { ReactNode } from 'react'
import type {
  GpuInfo,
  LiveGpuInfo,
} from '@/entities/system-info/model/types'
import { formatMb, gpuUsage, loadColor } from './gpu-helpers'

interface GpuOverviewProps {
  gpus: GpuInfo[]
  gpuIndex: number | null
  liveByIndex: Record<string, LiveGpuInfo>
  liveInfo: LiveGpuInfo[] | null
  t: TFunction
}

function Row({ label, value }: { label: string, value: ReactNode }) {
  return (
    <div className="flex items-center justify-between gap-4">
      <span className="text-xs text-muted-foreground">{label}</span>
      <span className="text-right text-xs font-medium text-foreground">
        {value}
      </span>
    </div>
  )
}

export function GpuOverview({
  gpus,
  gpuIndex,
  liveByIndex,
  liveInfo,
  t,
}: GpuOverviewProps) {
  const gpusToShow
    = gpuIndex !== null
      ? [{ gpu: gpus[gpuIndex], idx: gpuIndex }]
      : gpus.map((g, idx) => ({ gpu: g, idx }))

  const hasAnyLiveData = (liveInfo?.length ?? 0) > 0

  return (
    <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
      <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
        {gpusToShow.map(({ gpu: g, idx }) => {
          const gpuLive = liveByIndex[idx]
          const liveUsage = gpuLive ? gpuUsage(gpuLive) : 0
          return (
            <section
              className="flex flex-col gap-3 rounded-lg border border-border/70 bg-card p-4"
              key={g.index}
            >
              {gpuIndex === null && (
                <h3 className="text-sm font-medium text-foreground">
                  {t('gpu.adapter')}
                  {gpus.length > 1 ? ` ${idx}` : ''}
                </h3>
              )}
              <Row label={t('home.model')} value={g.name} />
              {g.vendor && g.vendor !== 'Unknown' && (
                <Row label={t('home.vendor')} value={g.vendor} />
              )}
              {g.vramTotalMb > 0 && (
                <Row
                  label={t('home.vram')}
                  value={formatMb(g.vramTotalMb, t)}
                />
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
                <Row
                  label={t('gpu.temperature')}
                  value={`${gpuLive.temperatureC} ${t('format.temperatureUnit')}`}
                />
              )}
            </section>
          )
        })}

        {!hasAnyLiveData && liveInfo && (
          <p className="col-span-2 text-xs text-muted-foreground">
            {t('gpu.noLiveData')}
          </p>
        )}
      </div>
    </section>
  )
}
