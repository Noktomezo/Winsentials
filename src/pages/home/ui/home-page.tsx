import type { LucideIcon } from 'lucide-react'
import type { CSSProperties, ReactNode } from 'react'
import type {
  GpuInfo,
  LiveGpuInfo,
  LiveHomeInfo,
  NetworkAdapterInfo,
  StaticSystemInfo,
} from '@/entities/system-info/model/types'
import { useNavigate, useRouter } from '@tanstack/react-router'
import {
  ChevronRight,
  Cpu,
  HardDrive,
  Layers,
  Monitor,
  Network,
  Server,
} from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { useDeviceInventory, useLiveHome } from '@/entities/system-info/model/live-system-store'
import { useStaticSystemInfo } from '@/entities/system-info/model/static-system-info'
import { useTweakCacheStore } from '@/entities/tweak/model/tweak-cache-store'
import {
  formatBytesLocalized,
  formatRateLocalized,
} from '@/shared/lib/format-size'
import { useMountEffect } from '@/shared/lib/hooks/use-mount-effect'
import { useRouteIntentPreload } from '@/shared/lib/hooks/use-route-intent-preload'
import {
  mountLabel,
  mountToParam,
  networkAdapterToParam,
} from '@/shared/lib/mount-utils'
import { Button, Skeleton } from '@/shared/ui'

// ─── Utilities ────────────────────────────────────────────────────────────────

const SUMMARY_ICON_WIDTH = 36
const SUMMARY_ICON_GAP = 12
const SUMMARY_HEADER_GAP = 8
const SUMMARY_CHEVRON_WIDTH = 14
const SUMMARY_HORIZONTAL_PADDING = 32
const SUMMARY_MIN_CARD_WIDTH = 360
const SUMMARY_MAX_CARD_WIDTH = 760
const HOME_TWEAK_CATEGORIES = ['appearance', 'behaviour', 'security', 'privacy', 'network', 'performance', 'memory', 'input'] as const

let summaryMeasureCanvas: HTMLCanvasElement | null = null
const summaryWidthCache = new Map<string, number>()

// Font metrics here approximate the rendered UI font. If fonts or text styles
// change at runtime, this cache may need invalidation to re-measure accurately.
function measureSummaryTextWidth(text: string, font: string) {
  if (typeof document === 'undefined') {
    return text.length * 8
  }

  summaryMeasureCanvas ??= document.createElement('canvas')
  const context = summaryMeasureCanvas.getContext('2d')

  if (!context) {
    return text.length * 8
  }

  context.font = font
  return Math.ceil(context.measureText(text).width)
}

function summaryCardWidth(titleText: string, secondaryText?: string) {
  const cacheKey = `${titleText}|${secondaryText ?? ''}`
  const cachedWidth = summaryWidthCache.get(cacheKey)

  if (cachedWidth) {
    return cachedWidth
  }

  const titleWidth = measureSummaryTextWidth(
    titleText,
    '500 14px "IBM Plex Sans", "Segoe UI Variable Text", "Segoe UI", sans-serif',
  )
  const secondaryWidth = secondaryText
    ? measureSummaryTextWidth(
        secondaryText,
        '400 14px "IBM Plex Sans", "Segoe UI Variable Text", "Segoe UI", sans-serif',
      )
    : 0

  const headerWidth
    = SUMMARY_ICON_WIDTH
      + SUMMARY_ICON_GAP
      + titleWidth
      + (secondaryWidth ? SUMMARY_HEADER_GAP + secondaryWidth : 0)
      + SUMMARY_CHEVRON_WIDTH
      + SUMMARY_HORIZONTAL_PADDING

  const width = Math.max(
    SUMMARY_MIN_CARD_WIDTH,
    Math.min(SUMMARY_MAX_CARD_WIDTH, headerWidth),
  )

  summaryWidthCache.set(cacheKey, width)
  return width
}

function loadColor(pct: number): string {
  if (pct >= 85) return 'metric-text-danger'
  if (pct >= 60) return 'metric-text-warning'
  return 'metric-text-good'
}

function usagePct(used: number, total: number): number {
  if (total === 0) return 0
  return Math.round((used / total) * 100)
}

function formatBytes(
  bytes: number,
  t: ReturnType<typeof useTranslation>['t'],
  locale: string,
  decimals = 1,
): string {
  return formatBytesLocalized(bytes, { decimals, locale, t })
}

function formatRate(
  bytes: number,
  t: ReturnType<typeof useTranslation>['t'],
  locale: string,
): string {
  return formatRateLocalized(bytes, { locale, t })
}

function createSummaryNavHandlers(
  navigate: ReturnType<typeof useNavigate>,
  router: ReturnType<typeof useRouter>,
  preloadRouteIntent: ReturnType<typeof useRouteIntentPreload>,
  to: string,
  params?: Record<string, string>,
) {
  const route = params ? { to, params } : { to }

  return {
    onNavigate: () => {
      void navigate(route as never)
    },
    onPointerIntent: () => {
      preloadRouteIntent(() => router.preloadRoute(route as never))
    },
  }
}

// ─── OS card (static, full-width) ─────────────────────────────────────────────

interface RowProps {
  label: string
  value: ReactNode
}

interface SummaryCardProps {
  icon: LucideIcon
  title: ReactNode
  secondaryText?: string
  titleText: string
  stat?: ReactNode
  statText?: string
  onNavigate: () => void
  onPointerIntent?: () => void
  children?: ReactNode
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

function SystemOverviewCard({
  s,
  appliedTweaks,
  totalTweaks,
  tweaksReady,
}: {
  s: StaticSystemInfo
  appliedTweaks: number
  totalTweaks: number
  tweaksReady: boolean
}) {
  const { t } = useTranslation()
  const w = s.windows
  return (
    <section className="rounded-lg border border-border/70 bg-card p-4">
      <div className="mb-3 border-b border-border/70 pb-3">
        <div className="flex min-w-0 items-center gap-3">
          <span className="ui-soft-surface flex size-9 shrink-0 items-center justify-center rounded-md">
            <Monitor className="size-4" />
          </span>
          <div className="min-w-0 flex-1">
            <h2 className="text-sm font-medium text-foreground">{t('home.system')}</h2>
            <p className="mt-0.5 text-xs text-muted-foreground">{t('home.description')}</p>
          </div>
        </div>
      </div>
      <div className="system-info-grid grid grid-cols-2 gap-x-8 gap-y-2">
        <Row
          label={t('home.version')}
          value={`${w.productName} ${w.displayVersion}`}
        />
        <Row label={t('home.build')} value={`${w.build}.${w.ubr}`} />
        <Row label={t('home.hostname')} value={w.hostname} />
        <Row label={t('home.username')} value={w.username} />
        <Row
          label={t('home.appliedTweaks')}
          value={tweaksReady
            ? <span className="metric-text-accent tabular-nums">{`${appliedTweaks} / ${totalTweaks}`}</span>
            : t('home.livePending')}
        />
        <Row label={t('home.architecture')} value={w.architecture} />
        <Row
          label={t('home.activation')}
          value={(
            <span
              className={
                (
                  {
                    activated: 'metric-text-good',
                    not_activated: 'metric-text-danger',
                    grace_period: 'metric-text-warning',
                  } as Record<string, string>
                )[w.activationStatus] ?? 'text-muted-foreground'
              }
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
  secondaryText,
  titleText,
  stat,
  onNavigate,
  onPointerIntent,
  children,
}: SummaryCardProps) {
  const cardWidth = summaryCardWidth(titleText, secondaryText)
  const cardStyle = {
    '--tweak-card-width': `${cardWidth}px`,
    '--tweak-card-grow': `${cardWidth}`,
  } as CSSProperties

  return (
    <button
      className="group/summary flex h-full cursor-pointer flex-col gap-3 rounded-lg border border-border/70 bg-card p-4 text-left transition-colors hover:bg-accent/20 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
      onClick={onNavigate}
      onFocus={onPointerIntent}
      onMouseEnter={onPointerIntent}
      style={cardStyle}
      type="button"
    >
      <div className="flex items-center gap-3">
        <span className="ui-soft-surface flex size-9 shrink-0 items-center justify-center rounded-md self-center">
          <Icon className="size-4" />
        </span>
        <div className="min-w-0 flex-1">
          <h2 className="truncate text-sm font-medium text-foreground">
            {title}
          </h2>
          {stat && (
            <div className="mt-0.5 min-w-0 text-xs font-medium text-foreground">
              {stat}
            </div>
          )}
        </div>
        <ChevronRight className="size-4 shrink-0 self-center text-muted-foreground transition-transform group-hover/summary:translate-x-0.5" />
      </div>
      {children}
    </button>
  )
}

function CpuSummary({
  live,
  s,
}: {
  live: LiveHomeInfo | null
  s: StaticSystemInfo
}) {
  const { t } = useTranslation()
  const navigate = useNavigate()
  const router = useRouter()
  const preloadRouteIntent = useRouteIntentPreload()
  const pct = live ? Math.round(live.cpuUsagePercent) : null
  const navHandlers = createSummaryNavHandlers(
    navigate,
    router,
    preloadRouteIntent,
    '/cpu',
  )
  return (
    <SummaryCard
      icon={Cpu}
      {...navHandlers}
      stat={
        pct === null
          ? (
              <span className="text-xs font-medium text-muted-foreground">
                {t('home.livePending')}
              </span>
            )
          : (
              <span
                className={`text-xs font-medium tabular-nums ${loadColor(pct)}`}
              >
                {pct}
                %
              </span>
            )
      }
      secondaryText={s.cpu.model}
      title={(
        <span className="flex min-w-0 items-baseline gap-1">
          <span className="shrink-0">{t('home.cpu')}</span>
          <span className="truncate font-normal text-muted-foreground">
            {`(${s.cpu.model})`}
          </span>
        </span>
      )}
      titleText={t('home.cpu')}
    />
  )
}

function RamSummary({
  live,
  s,
}: {
  live: LiveHomeInfo | null
  s: StaticSystemInfo
}) {
  const { t, i18n } = useTranslation()
  const navigate = useNavigate()
  const router = useRouter()
  const preloadRouteIntent = useRouteIntentPreload()
  const total = s.ram.totalBytes
  const used = live?.ramUsedBytes ?? null
  const pct = used === null ? null : usagePct(used, total)
  const navHandlers = createSummaryNavHandlers(
    navigate,
    router,
    preloadRouteIntent,
    '/ram',
  )
  return (
    <SummaryCard
      icon={Server}
      {...navHandlers}
      stat={
        used === null || pct === null
          ? (
              <span className="text-xs font-medium text-muted-foreground">
                {t('home.livePending')}
              </span>
            )
          : (
              <span
                className={`text-xs font-medium tabular-nums ${loadColor(pct)}`}
              >
                {t('home.usedOf', {
                  used: formatBytes(used, t, i18n.language),
                  total: formatBytes(total, t, i18n.language),
                })}
              </span>
            )
      }
      title={t('home.ram')}
      titleText={t('home.ram')}
    />
  )
}

function DiskSummary({
  disk,
  index,
}: {
  disk: StaticSystemInfo['disks'][number]
  index: number
}) {
  const { t, i18n } = useTranslation()
  const navigate = useNavigate()
  const router = useRouter()
  const preloadRouteIntent = useRouteIntentPreload()
  const used = disk.totalBytes - disk.availableBytes
  const param = mountToParam(disk.mountPoint)
  const navHandlers = createSummaryNavHandlers(
    navigate,
    router,
    preloadRouteIntent,
    '/storage/$disk',
    { disk: param },
  )
  return (
    <SummaryCard
      icon={HardDrive}
      {...navHandlers}
      stat={(
        <span
          className={`text-xs font-medium tabular-nums ${loadColor(usagePct(used, disk.totalBytes))}`}
        >
          {t('home.usedOf', {
            used: formatBytes(used, t, i18n.language),
            total: formatBytes(disk.totalBytes, t, i18n.language),
          })}
        </span>
      )}
      secondaryText={
        disk.volumeLabel
          ? `${mountLabel(disk.mountPoint)} - ${disk.volumeLabel}`
          : mountLabel(disk.mountPoint)
      }
      title={(
        <span className="flex min-w-0 items-baseline gap-1">
          <span className="shrink-0">{t('storage.diskLabel', { index })}</span>
          <span className="truncate font-normal text-muted-foreground">
            {disk.volumeLabel
              ? `(${mountLabel(disk.mountPoint)} - ${disk.volumeLabel})`
              : `(${mountLabel(disk.mountPoint)})`}
          </span>
        </span>
      )}
      titleText={t('storage.diskLabel', { index })}
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
  const navHandlers = createSummaryNavHandlers(
    navigate,
    router,
    preloadRouteIntent,
    '/network-stats/$adapterName',
    { adapterName },
  )
  return (
    <SummaryCard
      icon={Network}
      {...navHandlers}
      stat={(
        <div className="flex gap-2">
          <span className="metric-text-accent text-xs tabular-nums">
            ↓
            {formatRate(traffic?.rxBytesPerSec ?? 0, t, i18n.language)}
          </span>
          <span className="metric-text-accent text-xs tabular-nums">
            ↑
            {formatRate(traffic?.txBytesPerSec ?? 0, t, i18n.language)}
          </span>
        </div>
      )}
      secondaryText={adapter.adapterDescription}
      title={(
        <span className="flex min-w-0 items-baseline gap-1">
          <span className="shrink-0">{t('home.network')}</span>
          <span className="truncate font-normal text-muted-foreground">
            {`(${adapter.adapterDescription})`}
          </span>
        </span>
      )}
      titleText={t('home.network')}
    />
  )
}

function gpuUsage(
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

function GpuSummary({
  gpu,
  index,
  gpuLive,
}: {
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
  const navHandlers = createSummaryNavHandlers(
    navigate,
    router,
    preloadRouteIntent,
    '/gpu/$gpuIndex',
    { gpuIndex },
  )
  return (
    <SummaryCard
      icon={Layers}
      {...navHandlers}
      stat={
        usage != null
          ? (
              <span
                className={`text-xs font-medium tabular-nums ${loadColor(usage)}`}
              >
                {usage}
                %
              </span>
            )
          : undefined
      }
      secondaryText={gpu.name}
      title={(
        <span className="flex min-w-0 items-baseline gap-1">
          <span className="shrink-0">{t('gpu.gpuLabel', { index })}</span>
          <span className="truncate font-normal text-muted-foreground">
            {`(${gpu.name})`}
          </span>
        </span>
      )}
      titleText={t('gpu.gpuLabel', { index })}
    />
  )
}

// ─── Loading skeleton ─────────────────────────────────────────────────────────

function HomeSkeleton() {
  return (
    <div className="flex flex-col gap-4">
      <section className="rounded-lg border border-border/70 bg-card p-4">
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
      <div className="tweak-card-grid">
        {Array.from({ length: 4 }).map((_, i) => (
          <section
            className="rounded-lg border border-border/70 bg-card p-4"
            key={i}
            style={
              {
                '--tweak-card-width': '22.5rem',
                '--tweak-card-grow': '360',
              } as CSSProperties
            }
          >
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
  const { data: deviceInventory } = useDeviceInventory()
  const tweakCategories = useTweakCacheStore(state => state.categories)

  useMountEffect(() => {
    const store = useTweakCacheStore.getState()
    for (const category of HOME_TWEAK_CATEGORIES) {
      void store.ensureCategory(category)
    }
  })

  let appliedTweaks = 0
  let totalTweaks = 0
  let tweaksReady = true

  for (const category of HOME_TWEAK_CATEGORIES) {
    const cached = tweakCategories[category]
    if (!cached?.hasLoaded) {
      tweaksReady = false
      continue
    }

    totalTweaks += cached.tweaks.length
    appliedTweaks += cached.tweaks.filter(tweak => tweak.currentValue !== tweak.defaultValue).length
  }

  const networkCards = deviceInventory?.networkAdapters ?? staticInfo?.networkAdapters ?? []
  const disks = deviceInventory?.disks ?? staticInfo?.disks ?? []

  return (
    <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
      {staticInfoError
        ? (
            <section className="flex flex-col gap-3 rounded-lg border border-border/70 bg-card p-4">
              <p className="text-sm text-muted-foreground">{t('home.loadError')}</p>
              <div>
                <Button
                  onClick={retryStaticInfo}
                  size="sm"
                  type="button"
                  variant="outline"
                >
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
              <div className="flex flex-col gap-4">
                <SystemOverviewCard
                  appliedTweaks={appliedTweaks}
                  s={staticInfo}
                  totalTweaks={totalTweaks}
                  tweaksReady={tweaksReady}
                />
                <div className="tweak-card-grid">
                  <CpuSummary live={liveInfo} s={staticInfo} />
                  <RamSummary live={liveInfo} s={staticInfo} />
                  {disks.map((disk, i) => (
                    <DiskSummary disk={disk} index={i} key={disk.mountPoint} />
                  ))}
                  {networkCards.map(adapter => (
                    <NetworkSummary
                      adapter={adapter}
                      key={`network-${adapter.name}`}
                      live={liveInfo}
                    />
                  ))}
                  {staticInfo.gpus.map((gpu, i) => (
                    <GpuSummary
                      gpu={gpu}
                      gpuLive={liveInfo?.gpus[i] ?? null}
                      index={i}
                      key={`gpu-${i}`}
                    />
                  ))}
                </div>
              </div>
            )}
    </section>
  )
}
