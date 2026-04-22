import type { ReactNode } from 'react'
import type { StaticSystemInfo } from '@/entities/system-info/model/types'
import type { ChartPoint } from '@/shared/ui/live-chart'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { getStaticSystemInfo } from '@/entities/system-info/api'
import { useLiveRam } from '@/entities/system-info/model/live-system-store'
import { formatBytesLocalized } from '@/shared/lib/format-size'
import { useMountEffect } from '@/shared/lib/hooks/use-mount-effect'
import { LiveChart } from '@/shared/ui/live-chart'
import { Skeleton } from '@/shared/ui/skeleton'

// ─── Utilities ────────────────────────────────────────────────────────────────

function formatBytes(bytes: number, locale: string, t: ReturnType<typeof useTranslation>['t'], decimals = 1): string {
  return formatBytesLocalized(bytes, { decimals, locale, t })
}

/** Formats compressed memory with localized units, using 1 decimal for GiB+ and 0 otherwise. */
function formatCompressedBytes(
  bytes: number,
  locale: string,
  t: ReturnType<typeof useTranslation>['t'],
): string {
  return formatBytesLocalized(bytes, {
    decimals: bytes >= 1024 ** 3 ? 1 : 0,
    locale,
    t,
  })
}

function toGb(bytes: number, locale: string, t: ReturnType<typeof useTranslation>['t']): string {
  return formatBytesLocalized(bytes, { decimals: 1, locale, t })
}

function loadColor(pct: number): string {
  if (pct >= 85) {
    return 'metric-text-danger'
  }
  if (pct >= 60) {
    return 'metric-text-warning'
  }
  return 'metric-text-good'
}

function inverseLoadColor(pct: number): string {
  if (pct <= 15) {
    return 'metric-text-danger'
  }
  if (pct <= 40) {
    return 'metric-text-warning'
  }
  return 'metric-text-good'
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

// ─── Strip bar (used / reserved / free) ───────────────────────────────────────

function StripBar({
  usedBytes,
  availableBytes,
  totalBytes,
}: {
  usedBytes: number
  availableBytes: number
  totalBytes: number
}) {
  const { t, i18n } = useTranslation()
  if (totalBytes === 0) return null

  // "Other" = neither used nor available (kernel, cache, hardware reserved etc.)
  const otherBytes = Math.max(0, totalBytes - usedBytes - availableBytes)

  const usedPct = Math.min(100, (usedBytes / totalBytes) * 100)
  const availablePct = Math.min(100 - usedPct, (availableBytes / totalBytes) * 100)
  const otherPct = Math.max(0, 100 - usedPct - availablePct)

  const segments = [
    { pct: usedPct, color: 'metric-bg-accent', label: t('ram.used'), value: toGb(usedBytes, i18n.language, t) },
    { pct: otherPct, color: 'metric-bg-warning', label: t('ram.other'), value: toGb(otherBytes, i18n.language, t) },
    { pct: availablePct, color: 'metric-bg-good', label: t('ram.available'), value: toGb(availableBytes, i18n.language, t) },
  ]

  return (
    <section className="flex flex-col gap-2 rounded-lg border border-border/70 bg-card p-4">
      <span className="text-xs font-medium text-muted-foreground">
        {t('ram.memoryStructure')}
      </span>
      <div className="flex h-4 w-full overflow-hidden rounded-full">
        {segments.map(s => (
          s.pct > 0 && (
            <div
              className={`${s.color} transition-all`}
              key={s.label}
              style={{ width: `${s.pct}%` }}
            />
          )
        ))}
      </div>
      <div className="flex flex-wrap gap-x-4 gap-y-1">
        {segments.map(s => (
          <div className="flex items-center gap-1.5" key={s.label}>
            <span className={`size-2 shrink-0 rounded-sm ${s.color}`} />
            <span className="text-xs text-muted-foreground">{s.label}</span>
            <span className="text-xs font-medium tabular-nums text-foreground">{s.value}</span>
          </div>
        ))}
      </div>
    </section>
  )
}

// ─── Page ─────────────────────────────────────────────────────────────────────

export function RamPage() {
  const { t, i18n } = useTranslation()
  const [staticInfo, setStaticInfo] = useState<StaticSystemInfo | null>(null)
  const { data: liveInfo, history: rawHistory } = useLiveRam()
  const history: ChartPoint[] = rawHistory.map(v => ({ value: v }))
  const peak = rawHistory.length > 0 ? Math.max(...rawHistory) : 0

  useMountEffect(() => {
    getStaticSystemInfo().then(setStaticInfo).catch(console.error)
  })

  if (!staticInfo) {
    return (
      <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
        {[1, 2, 3].map(i => (
          <section className="rounded-lg border border-border/70 bg-card p-4" key={i}>
            <Skeleton className="mb-3 h-4 w-32" />
            <div className="space-y-2.5">
              <Skeleton className="h-3 w-full" />
              <Skeleton className="h-3 w-4/5" />
              <Skeleton className="h-3 w-3/5" />
            </div>
          </section>
        ))}
      </section>
    )
  }

  if (!liveInfo) {
    return (
      <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
        {[1, 2, 3].map(i => (
          <section className="rounded-lg border border-border/70 bg-card p-4" key={i}>
            <Skeleton className="mb-3 h-4 w-32" />
            <div className="space-y-2.5">
              <Skeleton className="h-3 w-full" />
              <Skeleton className="h-3 w-4/5" />
              <Skeleton className="h-3 w-3/5" />
            </div>
          </section>
        ))}
      </section>
    )
  }

  const { ram } = staticInfo
  const total = ram.totalBytes
  const maxY = total / 1024 ** 3

  const used = liveInfo.ramUsedBytes
  const available = liveInfo.ramAvailableBytes
  const committed = liveInfo.ramCommittedBytes
  const commitLimit = liveInfo.ramCommitLimitBytes
  const cached = liveInfo.ramCachedBytes
  const compressed = liveInfo.ramCompressedBytes
  const pagedPool = liveInfo.ramPagedPoolBytes
  const nonPagedPool = liveInfo.ramNonpagedPoolBytes

  // Hardware reserved: physical BIOS reservation = WMI total - sysinfo (used + available)
  const sysTotal = used + available
  const hwReserved = sysTotal > 0 ? Math.max(0, total - sysTotal) : 0
  const usedPercent = total > 0 ? (used / total) * 100 : 0
  const availablePercent = total > 0 ? (available / total) * 100 : 0
  const committedPercent = commitLimit > 0 ? (committed / commitLimit) * 100 : 0

  return (
    <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">

      {/* Usage chart with corner labels */}
      <section className="flex flex-col gap-1 rounded-lg border border-border/70 bg-card p-4">
        <div className="flex items-baseline justify-between">
          <span className="text-xs font-medium text-muted-foreground">
            {t('ram.memoryUsage')}
          </span>
          <span className="text-xs tabular-nums text-muted-foreground">
            {toGb(used, i18n.language, t)}
            {' ('}
            {t('ram.peak')}
            {': '}
            {toGb(Math.round(peak * 1024 ** 3), i18n.language, t)}
            )
          </span>
        </div>
        <LiveChart data={history} height={96} unit=" GB" yDomain={[0, maxY]} />
        <div className="flex items-baseline justify-between">
          <span className="text-xs text-muted-foreground">{t('ram.seconds', { n: 60 })}</span>
          <span className="text-xs tabular-nums text-muted-foreground">0</span>
        </div>
      </section>

      {/* Strip bar */}
      <StripBar availableBytes={available} totalBytes={total} usedBytes={used} />

      {/* Info card — 2 columns */}
      <section className="flex flex-col gap-3 rounded-lg border border-border/70 bg-card p-4">
        <h3 className="text-sm font-medium text-foreground">{t('ram.info')}</h3>
        <div className="system-info-grid grid grid-cols-1 gap-2 sm:grid-cols-2">
          <Row
            label={`${t('ram.used')} (${t('ram.compressed')})`}
            value={(
              <span className={`${loadColor(usedPercent)} tabular-nums`}>
                {formatBytes(used, i18n.language, t)}
                <span className="metric-text-accent ml-1 font-normal">
                  (
                  {formatCompressedBytes(compressed, i18n.language, t)}
                  )
                </span>
              </span>
            )}
          />
          <Row
            label={t('ram.available')}
            value={<span className={`${inverseLoadColor(availablePercent)} tabular-nums`}>{formatBytes(available, i18n.language, t)}</span>}
          />
          <Row
            label={t('ram.committed')}
            value={(
              <span className={`${loadColor(committedPercent)} tabular-nums`}>
                {formatBytes(committed, i18n.language, t)}
                {' / '}
                {formatBytes(commitLimit > 0 ? commitLimit : total, i18n.language, t)}
              </span>
            )}
          />
          <Row
            label={t('ram.cached')}
            value={<span className="metric-text-accent tabular-nums">{formatBytes(cached, i18n.language, t)}</span>}
          />
          <Row
            label={t('ram.pagedPool')}
            value={<span className="metric-text-accent tabular-nums">{formatBytes(pagedPool, i18n.language, t)}</span>}
          />
          <Row
            label={t('ram.nonPagedPool')}
            value={<span className="metric-text-accent tabular-nums">{formatBytes(nonPagedPool, i18n.language, t)}</span>}
          />
          {ram.speedMhz != null && (
            <Row label={t('home.speed')} value={`${ram.speedMhz} MHz`} />
          )}
          {ram.totalSlots > 0 && (
            <Row label={t('home.slots')} value={`${ram.usedSlots} / ${ram.totalSlots}`} />
          )}
          {ram.formFactor != null && (
            <Row label={t('ram.formFactor')} value={ram.formFactor} />
          )}
          <Row
            label={t('ram.hardwareReserved')}
            value={<span className="tabular-nums">{formatBytes(hwReserved, i18n.language, t)}</span>}
          />
        </div>
      </section>
    </section>
  )
}
