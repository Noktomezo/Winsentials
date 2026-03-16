import type { LucideIcon } from 'lucide-react'
import type { LiveSystemInfo, StaticSystemInfo } from '@/entities/system-info/model/types'
import { useNavigate } from '@tanstack/react-router'
import { ChevronRight, Cpu, HardDrive, Layers, Monitor, Network, Server } from 'lucide-react'
import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { getLiveSystemInfo, getStaticSystemInfo } from '@/entities/system-info/api'
import { Skeleton } from '@/shared/ui/skeleton'

// ─── Utilities ────────────────────────────────────────────────────────────────

function loadColor(pct: number): string {
  if (pct >= 85) { return 'text-destructive' }
  if (pct >= 60) { return 'text-warning' }
  return 'text-success'
}

function tempColor(celsius: number): string {
  if (celsius >= 85) { return 'text-destructive' }
  if (celsius >= 70) { return 'text-warning' }
  return 'text-success'
}

function usagePct(used: number, total: number): number {
  if (total === 0) { return 0 }
  return Math.round((used / total) * 100)
}

function formatBytes(bytes: number, decimals = 1): string {
  if (bytes === 0) { return '0 B' }
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return `${Number.parseFloat((bytes / k ** i).toFixed(decimals))} ${sizes[i]}`
}

// ─── OS card (static, full-width) ─────────────────────────────────────────────

function Row({ label, value }: { label: string, value: React.ReactNode }) {
  return (
    <div className="flex items-center justify-between gap-4">
      <span className="text-xs text-muted-foreground">{label}</span>
      <span className="text-right text-xs font-medium text-foreground">{value}</span>
    </div>
  )
}

function WindowsCard({ s }: { s: StaticSystemInfo }) {
  const { t } = useTranslation()
  const w = s.windows
  return (
    <section className="col-span-2 flex flex-col gap-3 rounded-xl border border-border/70 bg-card p-4">
      <div className="flex items-center gap-2">
        <span className="flex size-7 shrink-0 items-center justify-center rounded-md bg-accent/60 text-accent-foreground">
          <Monitor className="size-3.5" />
        </span>
        <h2 className="text-sm font-medium text-foreground">{t('home.os')}</h2>
      </div>
      <div className="grid grid-cols-2 gap-x-8 gap-y-2">
        <Row label={t('home.version')} value={`${w.productName} ${w.displayVersion}`} />
        <Row label={t('home.build')} value={`${w.build}.${w.ubr}`} />
        <Row label={t('home.hostname')} value={w.hostname} />
        <Row label={t('home.username')} value={w.username} />
        <Row label={t('home.architecture')} value={w.architecture} />
        <Row
          label={t('home.activation')}
          value={(
            <span className={({
              activated: 'text-success',
              not_activated: 'text-destructive',
              grace_period: 'text-warning',
            } as Record<string, string>)[w.activationStatus] ?? 'text-muted-foreground'}
            >
              {t(`home.activationStatus.${w.activationStatus}`)}
            </span>
          )}
        />
      </div>
    </section>
  )
}

// ─── Clickable summary cards ───────────────────────────────────────────────────

function SummaryCard({
  icon: Icon,
  title,
  onNavigate,
  children,
}: {
  icon: LucideIcon
  title: React.ReactNode
  onNavigate: () => void
  children: React.ReactNode
}) {
  return (
    <button
      className="group flex flex-col gap-3 rounded-xl border border-border/70 bg-card p-4 text-left transition-colors hover:bg-accent/20 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
      onClick={onNavigate}
      type="button"
    >
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-2">
          <span className="flex size-7 shrink-0 items-center justify-center rounded-md bg-accent/60 text-accent-foreground">
            <Icon className="size-3.5" />
          </span>
          <h2 className="text-sm font-medium text-foreground">{title}</h2>
        </div>
        <ChevronRight className="size-3.5 text-muted-foreground transition-transform group-hover:translate-x-0.5" />
      </div>
      {children}
    </button>
  )
}

function CpuSummary({ live }: { live: LiveSystemInfo | null }) {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const pct = live ? Math.round(live.cpuUsagePercent) : 0
  return (
    <SummaryCard icon={Cpu} onNavigate={() => void navigate({ to: '/cpu' })} title={t('home.cpu')}>
      <div className="space-y-1">
        <div className="flex justify-between">
          <span className="text-xs text-muted-foreground">{t('home.usage')}</span>
          <span className={`text-xs font-medium tabular-nums ${loadColor(pct)}`}>
            {pct}
            %
          </span>
        </div>
      </div>
    </SummaryCard>
  )
}

function RamSummary({ live, s }: { live: LiveSystemInfo | null, s: StaticSystemInfo }) {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const total = s.ram.totalBytes
  const used = live?.ramUsedBytes ?? 0
  const pct = usagePct(used, total)
  return (
    <SummaryCard icon={Server} onNavigate={() => void navigate({ to: '/ram' })} title={t('home.ram')}>
      <div className="space-y-1">
        <div className="flex justify-between">
          <span className="text-xs text-muted-foreground">{t('home.usage')}</span>
          <span className={`text-xs font-medium tabular-nums ${live ? loadColor(pct) : 'text-primary'}`}>
            {t('home.usedOf', { used: formatBytes(used), total: formatBytes(total) })}
          </span>
        </div>
      </div>
    </SummaryCard>
  )
}

function DiskSummary({ disk, index }: { disk: StaticSystemInfo['disks'][number], index: number }) {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const used = disk.totalBytes - disk.availableBytes
  const param = disk.mountPoint.replace(/[:\\/]/g, '')
  return (
    <SummaryCard
      icon={HardDrive}
      onNavigate={() => void navigate({ to: '/storage/$disk', params: { disk: param } })}
      title={(
        <>
          {t('storage.diskLabel', { index })}
          {' '}
          <span className="font-normal text-muted-foreground">
            (
            {disk.mountPoint}
            )
          </span>
        </>
      )}
    >
      <div className="space-y-1">
        <div className="flex justify-between text-xs text-muted-foreground">
          <span>{disk.kind}</span>
          <span>{disk.fileSystem}</span>
        </div>
        <div className="flex justify-between">
          <span className="text-xs text-muted-foreground">{t('home.usage')}</span>
          <span className={`text-xs font-medium tabular-nums ${loadColor(usagePct(used, disk.totalBytes))}`}>
            {t('home.usedOf', { used: formatBytes(used), total: formatBytes(disk.totalBytes) })}
          </span>
        </div>
      </div>
    </SummaryCard>
  )
}

function NetworkSummary({ live }: { live: LiveSystemInfo | null }) {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const ifaces = live?.network ?? []
  const active = ifaces.find(i => i.rxBytesPerSec > 0 || i.txBytesPerSec > 0) ?? ifaces[0]
  return (
    <SummaryCard icon={Network} onNavigate={() => void navigate({ to: '/network-stats' })} title={t('home.network')}>
      <div className="flex items-center justify-between gap-2">
        <span className="truncate text-xs font-medium text-muted-foreground">{active?.name ?? '—'}</span>
        <div className="flex shrink-0 gap-3">
          <span className="text-xs tabular-nums text-primary">
            ↓
            {formatBytes(active?.rxBytesPerSec ?? 0)}
            /s
          </span>
          <span className="text-xs tabular-nums text-primary">
            ↑
            {formatBytes(active?.txBytesPerSec ?? 0)}
            /s
          </span>
        </div>
      </div>
    </SummaryCard>
  )
}

function GpuSummary({ live, s }: { live: LiveSystemInfo | null, s: StaticSystemInfo }) {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const gpu = s.gpus[0]
  const gpuLive = live?.gpuLive[0]
  return (
    <SummaryCard icon={Layers} onNavigate={() => void navigate({ to: '/gpu' })} title={t('home.gpu')}>
      <div className="space-y-1">
        <div className="flex justify-between">
          <span className="text-xs text-muted-foreground">{t('home.model')}</span>
          <span className="max-w-[55%] truncate text-right text-xs font-medium text-foreground">{gpu.model}</span>
        </div>
        {gpuLive?.usagePercent != null && (
          <div className="flex justify-between">
            <span className="text-xs text-muted-foreground">{t('home.usage')}</span>
            <span className={`text-xs font-medium tabular-nums ${loadColor(Math.round(gpuLive.usagePercent))}`}>
              {Math.round(gpuLive.usagePercent)}
              %
            </span>
          </div>
        )}
        {gpuLive?.temperatureCelsius != null && (
          <div className="flex justify-between">
            <span className="text-xs text-muted-foreground">{t('gpu.temperature')}</span>
            <span className={`text-xs font-medium tabular-nums ${tempColor(Math.round(gpuLive.temperatureCelsius))}`}>
              {Math.round(gpuLive.temperatureCelsius)}
              {' '}
              °C
            </span>
          </div>
        )}
      </div>
    </SummaryCard>
  )
}

// ─── Loading skeleton ─────────────────────────────────────────────────────────

function HomeSkeleton() {
  return (
    <div className="grid grid-cols-2 gap-4">
      <section className="col-span-2 rounded-xl border border-border/70 bg-card p-4">
        <div className="mb-3 flex items-center gap-2">
          <Skeleton className="size-7 rounded-md" />
          <Skeleton className="h-4 w-24" />
        </div>
        <div className="grid grid-cols-2 gap-x-8 gap-y-2">
          {Array.from({ length: 5 }).map((_, i) => (
            <Skeleton className="h-3 w-full" key={i} />
          ))}
        </div>
      </section>
      {Array.from({ length: 4 }).map((_, i) => (
        <section className="rounded-xl border border-border/70 bg-card p-4" key={i}>
          <div className="mb-3 flex items-center gap-2">
            <Skeleton className="size-7 rounded-md" />
            <Skeleton className="h-4 w-24" />
          </div>
          <div className="space-y-2.5">
            <Skeleton className="h-3 w-full" />
            <Skeleton className="h-3 w-4/5" />
          </div>
        </section>
      ))}
    </div>
  )
}

// ─── Page ─────────────────────────────────────────────────────────────────────

export function HomePage() {
  const [staticInfo, setStaticInfo] = useState<StaticSystemInfo | null>(null)
  const [liveInfo, setLiveInfo] = useState<LiveSystemInfo | null>(null)

  useEffect(() => {
    getStaticSystemInfo()
      .then(setStaticInfo)
      .catch(console.error)
  }, [])

  useEffect(() => {
    if (!staticInfo) { return }

    const tick = () => {
      getLiveSystemInfo()
        .then(setLiveInfo)
        .catch(console.error)
    }

    tick()
    const id = setInterval(tick, 1000)
    return () => clearInterval(id)
  }, [staticInfo])

  return (
    <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
      {!staticInfo
        ? (
            <HomeSkeleton />
          )
        : (
            <div className="grid grid-cols-2 gap-4">
              <WindowsCard s={staticInfo} />
              <CpuSummary live={liveInfo} />
              <RamSummary live={liveInfo} s={staticInfo} />
              {staticInfo.disks.map((disk, i) => (
                <DiskSummary disk={disk} index={i} key={disk.mountPoint} />
              ))}
              <NetworkSummary live={liveInfo} />
              {staticInfo.gpus.length > 0 && <GpuSummary live={liveInfo} s={staticInfo} />}
            </div>
          )}
    </section>
  )
}
