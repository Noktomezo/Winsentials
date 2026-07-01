import type { TFunction } from 'i18next'
import type {
  GpuInfo,
  LiveGpuInfo,
} from '@/entities/system-info/model/types'
import type { ChartPoint } from '@/shared/ui/live-chart'

export function gpuUsage(
  gpu: Pick<
    LiveGpuInfo,
    | 'util3d'
    | 'utilCopy'
    | 'utilEncode'
    | 'utilDecode'
    | 'utilHighPriority3d'
    | 'utilHighPriorityCompute'
  >,
): number {
  return Math.max(
    gpu.util3d,
    gpu.utilCopy,
    gpu.utilEncode,
    gpu.utilDecode,
    gpu.utilHighPriority3d,
    gpu.utilHighPriorityCompute,
  )
}

export function formatMb(mb: number, t: TFunction): string {
  if (mb === 0) return `0 ${t('format.megabyte')}`
  if (mb >= 1024) return `${(mb / 1024).toFixed(1)} ${t('format.gigabyte')}`
  return `${mb} ${t('format.megabyte')}`
}

export function formatMbPair(
  used: number,
  total: number,
  t: TFunction,
): string {
  const useGb = total >= 1024
  if (useGb) {
    return `${(used / 1024).toFixed(1)} / ${(total / 1024).toFixed(1)} ${t('format.gigabyte')}`
  }
  return `${used} / ${total} ${t('format.megabyte')}`
}

export function loadColor(pct: number): string {
  if (pct >= 85) {
    return 'metric-text-danger'
  }
  if (pct >= 60) {
    return 'metric-text-warning'
  }
  return 'metric-text-good'
}

export function tempColorClass(temp: number): string {
  if (temp >= 80) {
    return 'metric-text-danger'
  }
  if (temp >= 60) {
    return 'metric-text-warning'
  }
  return 'metric-text-good'
}

export function memoryColorClass(used: number, total: number): string {
  if (total <= 0) {
    return 'metric-text-accent'
  }

  return loadColor(Math.round((used / total) * 100))
}

export function getEngineCharts(
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
  t: TFunction,
) {
  if (gpu.isIntegrated) {
    return [
      {
        key: '3d',
        label: t('gpu.engine3D'),
        value: live?.util3d ?? 0,
        data: history.threeD,
      },
      {
        key: 'copy',
        label: t('gpu.engineCopy'),
        value: live?.utilCopy ?? 0,
        data: history.copy,
      },
      {
        key: 'hp3d',
        label: t('gpu.engineHP3D'),
        value: live?.utilHighPriority3d ?? 0,
        data: history.highPriority3d,
      },
      {
        key: 'hpcompute',
        label: t('gpu.engineHPCompute'),
        value: live?.utilHighPriorityCompute ?? 0,
        data: history.highPriorityCompute,
      },
    ]
  }

  return [
    {
      key: '3d',
      label: t('gpu.engine3D'),
      value: live?.util3d ?? 0,
      data: history.threeD,
    },
    {
      key: 'copy',
      label: t('gpu.engineCopy'),
      value: live?.utilCopy ?? 0,
      data: history.copy,
    },
    {
      key: 'encode',
      label: t('gpu.engineVideoEncode'),
      value: live?.utilEncode ?? 0,
      data: history.encode,
    },
    {
      key: 'decode',
      label: t('gpu.engineVideoDecode'),
      value: live?.utilDecode ?? 0,
      data: history.decode,
    },
  ]
}
