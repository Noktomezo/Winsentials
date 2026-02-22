import { Cpu, Gpu, HardDrive, MemoryStick, Monitor } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { useDynamicSystemInfo, useStaticSystemInfo } from '@/shared/api/systemInfo'
import { SystemCard } from '@/widgets/system-card'

function formatBytes(bytes: number): string {
  if (bytes === 0)
    return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return `${Number.parseFloat((bytes / k ** i).toFixed(1))} ${sizes[i]}`
}

function formatFrequency(mhz: number): string {
  if (mhz >= 1000) {
    return `${(mhz / 1000).toFixed(1)} GHz`
  }
  return `${mhz} MHz`
}

export function HomePage() {
  const { t } = useTranslation()
  const { data: staticInfo, isLoading: staticLoading } = useStaticSystemInfo()
  const { data: dynamicInfo } = useDynamicSystemInfo()

  if (staticLoading || !staticInfo) {
    return (
      <div className="space-y-6">
        <div>
          <h1 className="text-2xl font-bold">{t('home.title')}</h1>
          <p className="text-muted-foreground">{t('home.loading')}</p>
        </div>
        <div className="grid gap-4 md:grid-cols-2">
          {[1, 2, 3, 4].map(i => (
            <div key={i} className="h-32 animate-pulse rounded-lg bg-muted" />
          ))}
        </div>
      </div>
    )
  }

  const { os, cpu: staticCpu, gpu: staticGpu, ram: staticRam, disks: staticDisks } = staticInfo

  const cpuUsage = dynamicInfo?.cpu.usage
  const cpuFrequency = dynamicInfo?.cpu.frequency
  const gpuUsage = dynamicInfo?.gpu?.usage
  const gpuMemoryUsed = dynamicInfo?.gpu?.memory_used
  const ramUsed = dynamicInfo?.ram.used
  const ramUsage = dynamicInfo?.ram.usage
  const dynamicDisks = dynamicInfo?.disks

  const diskUsageMap = new Map(
    dynamicDisks?.map(d => [d.mount_point, d]) ?? [],
  )

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold">{t('home.title')}</h1>
        <p className="text-muted-foreground">{t('home.description')}</p>
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        <SystemCard
          icon={Monitor}
          title={t('home.os')}
          value={`${os.name} ${os.version}`}
          metrics={[
            { label: t('home.build'), value: os.build },
            { label: t('home.arch'), value: os.arch },
            { label: t('home.displayVersion'), value: os.display_version },
            { label: t('home.hostname'), value: os.hostname },
            { label: t('home.username'), value: os.username },
          ]}
        />

        <SystemCard
          icon={Cpu}
          title={t('home.cpu')}
          value={staticCpu.name}
          usage={cpuUsage}
          metrics={[
            {
              label: t('home.cores'),
              value: `${staticCpu.cores} / ${staticCpu.logical_cores}`,
            },
            {
              label: t('home.frequency'),
              value: formatFrequency(cpuFrequency ?? 0),
            },
          ]}
        />

        {staticGpu && (
          <SystemCard
            icon={Gpu}
            title={t('home.gpu')}
            value={staticGpu.name}
            usage={gpuUsage}
            metrics={[
              {
                label: t('home.vramTotal'),
                value: formatBytes(staticGpu.memory_total),
              },
              {
                label: t('home.vramUsed'),
                value: gpuMemoryUsed !== undefined ? formatBytes(gpuMemoryUsed) : '—',
              },
            ]}
          />
        )}

        <SystemCard
          icon={MemoryStick}
          title={t('home.ram')}
          value={`${formatBytes(ramUsed ?? 0)} / ${formatBytes(staticRam.total)}`}
          usage={ramUsage}
          metrics={[
            {
              label: t('home.slots'),
              value: `${staticRam.slots_used} / ${staticRam.slots_total}`,
            },
            { label: t('home.speed'), value: `${staticRam.speed} MHz` },
          ]}
        />
      </div>

      <SystemCard
        icon={HardDrive}
        title={t('home.disk')}
        value={
          staticDisks.length > 1
            ? t('home.disks', { count: staticDisks.length })
            : staticDisks[0]?.mount_point || 'Unknown'
        }
        progressBars={staticDisks.map((disk) => {
          const dynamicDisk = diskUsageMap.get(disk.mount_point)
          return {
            label: disk.label
              ? `${disk.mount_point} (${disk.label})`
              : disk.mount_point,
            value: dynamicDisk?.usage ?? 0,
          }
        })}
      />
    </div>
  )
}
