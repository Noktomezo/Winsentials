import type { LucideIcon } from 'lucide-react'
import type { ReactNode } from 'react'
import type { GpuInfo, LiveGpuInfo, LiveHomeInfo, NetworkAdapterInfo, StaticSystemInfo } from '@/entities/system-info/model/types'
import { useNavigate, useRouter } from '@tanstack/react-router'
import { ChevronRight, Cpu, HardDrive, Layers, Monitor, Network, Server } from 'lucide-react'
import { useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useLiveHome } from '@/entities/system-info/model/live-system-store'
import { useStaticSystemInfo } from '@/entities/system-info/model/static-system-info'
import { formatBytesLocalized, formatRateLocalized } from '@/shared/lib/format-size'
import { useMountEffect } from '@/shared/lib/hooks/use-mount-effect'
import { useRouteIntentPreload } from '@/shared/lib/hooks/use-route-intent-preload'
import { mountLabel, mountToParam, networkAdapterToParam } from '@/shared/lib/mount-utils'
import { Button, Skeleton } from '@/shared/ui'

// ─── Utilities ────────────────────────────────────────────────────────────────

function loadColor(pct: number): string {
  if (pct >= 85) { return 'text-destructive' }
  if (pct >= 60) { return 'text-warning' }
  return 'text-success'
}

function usagePct(used: number, total: number): number {
  if (total === 0) { return 0 }
  return Math.round((used / total) * 100)
}

function formatBytes(bytes: number, t: ReturnType<typeof useTranslation>['t'], locale: string, decimals = 1): string {
  return formatBytesLocalized(bytes, { decimals, locale, t })
}

function formatRate(bytes: number, t: ReturnType<typeof useTranslation>['t'], locale: string): string {
  return formatRateLocalized(bytes, { locale, t })
}

function mergeVisibleNetworkAdapters(
  staticAdapters: NetworkAdapterInfo[],
  live: LiveHomeInfo | null,
): NetworkAdapterInfo[] {
  return (live?.network ?? [])
    .map(entry => staticAdapters.find(adapter => adapter.name === entry.name) ?? null)
    .filter((adapter): adapter is NetworkAdapterInfo => adapter !== null)
}

// ─── Marquee ──────────────────────────────────────────────────────────────────

function MeasuredMarqueeText({ text, className }: { text: string, className?: string }) {
  const outerRef = useRef<HTMLSpanElement>(null)
  const innerRef = useRef<HTMLSpanElement>(null)
  const [offset, setOffset] = useState(0)

  useMountEffect(() => {
    if (!outerRef.current || !innerRef.current) { return }

    const outer = outerRef.current
    const inner = innerRef.current

    const measure = () => {
      requestAnimationFrame(() => {
        const diff = inner.scrollWidth - outer.offsetWidth
        setOffset(diff > 0 ? diff + 2 : 0)
      })
    }

    measure()
    const ro = new ResizeObserver(measure)
    ro.observe(outer)
    return () => ro.disconnect()
  })

  return (
    <span
      className={`overflow-hidden ${className ?? ''}`}
      data-overflow={offset > 0 ? 'true' : 'false'}
      ref={outerRef}
    >
      <span
        data-marquee-inner="true"
        ref={innerRef}
        className={offset > 0
          ? 'inline-block whitespace-nowrap transition-transform duration-700 ease-out will-change-transform motion-reduce:transition-none'
          : 'inline-block whitespace-nowrap'}
        style={offset > 0 ? { ['--marquee-offset' as string]: `-${offset}px` } : undefined}
      >
        {text}
      </span>
    </span>
  )
}

// ─── OS card (static, full-width) ─────────────────────────────────────────────

interface RowProps {
  label: string
  value: ReactNode
}

interface SummaryCardProps {
  icon: LucideIcon
  title: ReactNode
  stat?: ReactNode
  onNavigate: () => void
  onPointerIntent?: () => void
  children?: ReactNode
}

function Row({ label, value }: RowProps) {
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
  stat,
  onNavigate,
  onPointerIntent,
  children,
}: SummaryCardProps) {
  return (
    <button
      className="group/summary flex cursor-pointer flex-col gap-3 rounded-xl border border-border/70 bg-card p-4 text-left transition-colors hover:bg-accent/20 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
      onClick={onNavigate}
      onFocus={onPointerIntent}
      onMouseEnter={onPointerIntent}
      type="button"
    >
      <div className="flex items-center justify-between gap-2">
        <div className="flex min-w-0 items-center gap-2">
          <span className="flex size-7 shrink-0 items-center justify-center rounded-md bg-accent/60 text-accent-foreground">
            <Icon className="size-3.5" />
          </span>
          <h2 className="min-w-0 text-sm font-medium text-foreground">{title}</h2>
        </div>
        <div className="flex shrink-0 items-center gap-2">
          {stat}
          <ChevronRight className="size-3.5 text-muted-foreground transition-transform group-hover/summary:translate-x-0.5" />
        </div>
      </div>
      {children}
    </button>
  )
}

function CpuSummary({ live, s }: { live: LiveHomeInfo | null, s: StaticSystemInfo }) {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const router = useRouter()
  const preloadRouteIntent = useRouteIntentPreload()
  const pct = live ? Math.round(live.cpuUsagePercent) : null
  return (
    <SummaryCard
      icon={Cpu}
      onNavigate={() => void navigate({ to: '/cpu' })}
      onPointerIntent={() => preloadRouteIntent(() => router.preloadRoute({ to: '/cpu' }))}
      stat={pct === null
        ? <span className="text-xs font-medium text-muted-foreground">{t('home.livePending')}</span>
        : (
            <span className={`text-xs font-medium tabular-nums ${loadColor(pct)}`}>
              {pct}
              %
            </span>
          )}
      title={(
        <span className="flex min-w-0 items-baseline gap-1">
          <span className="shrink-0">{t('home.cpu')}</span>
          <MarqueeText className="font-normal text-muted-foreground" text={`(${s.cpu.model})`} />
        </span>
      )}
    />
  )
}

function RamSummary({ live, s }: { live: LiveHomeInfo | null, s: StaticSystemInfo }) {
  const { t, i18n } = useTranslation()
  const navigate = useNavigate()
  const router = useRouter()
  const preloadRouteIntent = useRouteIntentPreload()
  const total = s.ram.totalBytes
  const used = live?.ramUsedBytes ?? null
  const pct = used === null ? null : usagePct(used, total)
  return (
    <SummaryCard
      icon={Server}
      onNavigate={() => void navigate({ to: '/ram' })}
      onPointerIntent={() => preloadRouteIntent(() => router.preloadRoute({ to: '/ram' }))}
      stat={used === null || pct === null
        ? <span className="text-xs font-medium text-muted-foreground">{t('home.livePending')}</span>
        : (
            <span className={`text-xs font-medium tabular-nums ${loadColor(pct)}`}>
              {t('home.usedOf', {
                used: formatBytes(used, t, i18n.language),
                total: formatBytes(total, t, i18n.language),
              })}
            </span>
          )}
      title={t('home.ram')}
    />
  )
}

function DiskSummary({ disk, index }: { disk: StaticSystemInfo['disks'][number], index: number }) {
  const { t, i18n } = useTranslation()
  const navigate = useNavigate()
  const router = useRouter()
  const preloadRouteIntent = useRouteIntentPreload()
  const used = disk.totalBytes - disk.availableBytes
  const param = mountToParam(disk.mountPoint)
  return (
    <SummaryCard
      icon={HardDrive}
      onNavigate={() => void navigate({ to: '/storage/$disk', params: { disk: param } })}
      onPointerIntent={() => preloadRouteIntent(() => router.preloadRoute({ to: '/storage/$disk', params: { disk: param } }))}
      stat={(
        <span className={`text-xs font-medium tabular-nums ${loadColor(usagePct(used, disk.totalBytes))}`}>
          {t('home.usedOf', {
            used: formatBytes(used, t, i18n.language),
            total: formatBytes(disk.totalBytes, t, i18n.language),
          })}
        </span>
      )}
      title={(
        <span className="flex min-w-0 items-baseline gap-1">
          <span className="shrink-0">{t('storage.diskLabel', { index })}</span>
          <MarqueeText
            className="font-normal text-muted-foreground"
            text={disk.volumeLabel
              ? `(${mountLabel(disk.mountPoint)} - ${disk.volumeLabel})`
              : `(${mountLabel(disk.mountPoint)})`}
          />
        </span>
      )}
    />
  )
}

function NetworkSummary({
  adapter,
  live,
}: {
  adapter: NetworkAdapterInfo
  live: LiveHomeInfo | null
}) {
  const { t, i18n } = useTranslation()
  const navigate = useNavigate()
  const router = useRouter()
  const preloadRouteIntent = useRouteIntentPreload()
  const traffic = live?.network.find(entry => entry.name === adapter.name)
  const adapterName = networkAdapterToParam(adapter.name)
  return (
    <SummaryCard
      icon={Network}
      onNavigate={() => void navigate({ to: '/network-stats/$adapterName', params: { adapterName } })}
      onPointerIntent={() => preloadRouteIntent(() => router.preloadRoute({ to: '/network-stats/$adapterName', params: { adapterName } }))}
      stat={(
        <div className="flex gap-2">
          <span className="text-xs tabular-nums text-primary">
            ↓
            {formatRate(traffic?.rxBytesPerSec ?? 0, t, i18n.language)}
          </span>
          <span className="text-xs tabular-nums text-primary">
            ↑
            {formatRate(traffic?.txBytesPerSec ?? 0, t, i18n.language)}
          </span>
        </div>
      )}
      title={(
        <span className="flex min-w-0 items-baseline gap-1">
          <span className="shrink-0">{t('home.network')}</span>
          <MarqueeText className="font-normal text-muted-foreground" text={`(${adapter.adapterDescription})`} />
        </span>
      )}
    />
  )
}

function MarqueeText({ text, className }: { text: string, className?: string }) {
  return <MeasuredMarqueeText key={text} className={className} text={text} />
}

function gpuUsage(gpu: Pick<LiveGpuInfo, 'util3d' | 'utilCopy' | 'utilEncode' | 'utilDecode' | 'utilHighPriority3d' | 'utilHighPriorityCompute'>): number {
  return Math.max(gpu.util3d, gpu.utilCopy, gpu.utilEncode, gpu.utilDecode, gpu.utilHighPriority3d, gpu.utilHighPriorityCompute)
}

function GpuSummary({ gpu, index, gpuLive }: {
  gpu: GpuInfo
  index: number
  gpuLive: LiveGpuInfo | null
}) {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const router = useRouter()
  const preloadRouteIntent = useRouteIntentPreload()
  const gpuIndex = String(index)
  const usage = gpuLive ? gpuUsage(gpuLive) : null
  return (
    <SummaryCard
      icon={Layers}
      onNavigate={() => void navigate({ to: '/gpu/$gpuIndex', params: { gpuIndex } })}
      onPointerIntent={() => preloadRouteIntent(() => router.preloadRoute({ to: '/gpu/$gpuIndex', params: { gpuIndex } }))}
      stat={usage != null
        ? (
            <span className={`text-xs font-medium tabular-nums ${loadColor(usage)}`}>
              {usage}
              %
            </span>
          )
        : undefined}
      title={(
        <span className="flex min-w-0 items-baseline gap-1">
          <span className="shrink-0">{t('gpu.gpuLabel', { index })}</span>
          <MarqueeText className="font-normal text-muted-foreground" text={`(${gpu.name})`} />
        </span>
      )}
    />
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
  const { t } = useTranslation()
  const {
    info: staticInfo,
    error: staticInfoError,
    isLoading: staticInfoLoading,
    retry: retryStaticInfo,
  } = useStaticSystemInfo()
  const { data: liveInfo } = useLiveHome()

  const networkCards = staticInfo
    ? mergeVisibleNetworkAdapters(staticInfo.networkAdapters, liveInfo)
    : []
  const disks = staticInfo?.disks ?? []

  return (
    <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
      {staticInfoError
        ? (
            <section className="flex flex-col gap-3 rounded-xl border border-border/70 bg-card p-4">
              <p className="text-sm text-muted-foreground">{t('home.loadError')}</p>
              <div>
                <Button onClick={retryStaticInfo} size="sm" type="button" variant="outline">
                  {t('tweaks.actions.retry')}
                </Button>
              </div>
            </section>
          )
        : staticInfoLoading || !staticInfo
          ? (
              <HomeSkeleton />
            )
          : (
              <div className="grid grid-cols-2 gap-4">
                <WindowsCard s={staticInfo} />
                <CpuSummary live={liveInfo} s={staticInfo} />
                <RamSummary live={liveInfo} s={staticInfo} />
                {disks.map((disk, i) => (
                  <DiskSummary disk={disk} index={i} key={disk.mountPoint} />
                ))}
                {networkCards.map(adapter => (
                  <NetworkSummary adapter={adapter} key={`network-${adapter.name}`} live={liveInfo} />
                ))}
                {staticInfo.gpus.map((gpu, i) => (
                  <GpuSummary gpu={gpu} gpuLive={liveInfo?.gpus[i] ?? null} index={i} key={`gpu-${i}`} />
                ))}
              </div>
            )}
    </section>
  )
}
