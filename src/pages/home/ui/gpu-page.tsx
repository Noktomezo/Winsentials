import type { LiveGpuInfo } from '@/entities/system-info/model/types'
import { Navigate, useParams } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'
import { useLiveGpu } from '@/entities/system-info/model/live-system-store'
import { useStaticSystemInfo } from '@/entities/system-info/model/static-system-info'
import { LiveErrorState, Skeleton } from '@/shared/ui'
import { GpuDetailView } from './gpu-detail-view'
import { GpuOverview } from './gpu-overview'

function LiveGpuLoadingState() {
  return (
    <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
      <div className="grid grid-cols-2 gap-4">
        {Array.from({ length: 4 }).map((_, i) => (
          <section
            className="rounded-lg border border-border/70 bg-card p-4"
            key={i}
          >
            <Skeleton className="mb-3 h-3 w-24" />
            <Skeleton className="h-16 w-full" />
          </section>
        ))}
        <section className="col-span-2 rounded-lg border border-border/70 bg-card p-4">
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

function GpuPage() {
  const { t } = useTranslation()
  const params = useParams({ strict: false })
  const parsedGpuIndex
    = params.gpuIndex !== undefined ? Number(params.gpuIndex) : null

  const { info: staticInfo, error: staticInfoError, retry: retryStaticInfo } = useStaticSystemInfo()
  const {
    data: liveInfo,
    error: liveError,
    history: gpuHistory,
    isFetching,
    retry,
  } = useLiveGpu()

  const gpuIndex
    = staticInfo
      && parsedGpuIndex !== null
      && Number.isInteger(parsedGpuIndex)
      && parsedGpuIndex >= 0
      && parsedGpuIndex < staticInfo.gpus.length
      ? parsedGpuIndex
      : null
  const gpu
    = staticInfo && gpuIndex !== null ? staticInfo.gpus[gpuIndex] : null
  const isDetailView = gpuIndex !== null && gpu != null
  const liveByIndex = Object.fromEntries(
    (liveInfo ?? []).map(sample => [sample.index, sample]),
  )
  const historyByIndex = gpuHistory

  if (staticInfo && params.gpuIndex !== undefined && gpuIndex === null) {
    return <Navigate replace to="/gpu" />
  }

  if (!staticInfo) {
    if (staticInfoError) {
      return (
        <LiveErrorState message={t('gpu.loadError')} onRetry={retryStaticInfo} />
      )
    }

    return <LiveGpuLoadingState />
  }

  // ── Detail view ──────────────────────────────────────────────────────────────
  if (isDetailView && gpu && gpuIndex !== null) {
    if (liveInfo === null && isFetching) {
      return <LiveGpuLoadingState />
    }

    if (liveInfo === null && liveError) {
      return (
        <LiveErrorState message={t('gpu.liveLoadError')} onRetry={retry} />
      )
    }

    const live = liveByIndex[gpuIndex] as LiveGpuInfo | undefined
    if (!live) {
      return (
        <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
          <section className="rounded-lg border border-border/70 bg-card p-4">
            <p className="text-sm text-muted-foreground">
              {t('gpu.noLiveData')}
            </p>
          </section>
        </section>
      )
    }

    return (
      <GpuDetailView
        gpu={gpu}
        gpuIndex={gpuIndex}
        historyByIndex={historyByIndex}
        live={live}
        t={t}
      />
    )
  }

  // ── Overview (all GPUs, no index selected) ───────────────────────────────────
  if (liveInfo === null && isFetching) {
    return <LiveGpuLoadingState />
  }

  if (liveInfo === null && liveError) {
    return (
      <LiveErrorState message={t('gpu.liveLoadError')} onRetry={retry} />
    )
  }

  return (
    <GpuOverview
      gpus={staticInfo.gpus}
      gpuIndex={gpuIndex}
      liveByIndex={liveByIndex}
      liveInfo={liveInfo}
      t={t}
    />
  )
}

export default GpuPage
