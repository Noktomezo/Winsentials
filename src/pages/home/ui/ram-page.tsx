import type { LiveSystemInfo, StaticSystemInfo } from '@/entities/system-info/model/types'
import type { ChartPoint } from '@/shared/ui/live-chart'
import { useEffect, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { getLiveSystemInfo, getStaticSystemInfo } from '@/entities/system-info/api'
import { LiveChart } from '@/shared/ui/live-chart'
import { Skeleton } from '@/shared/ui/skeleton'

// ─── Utilities ────────────────────────────────────────────────────────────────

function formatBytes(bytes: number, decimals = 1): string {
  if (bytes === 0) { return '0 B' }
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return `${Number.parseFloat((bytes / k ** i).toFixed(decimals))} ${sizes[i]}`
}

/** Format `bytes` as a plain number in the same unit as `reference`, no unit suffix. */
function formatBytesNumberOnly(bytes: number, reference: number): string {
  if (reference === 0) { return '0' }
  const k = 1024
  const i = Math.floor(Math.log(Math.max(reference, 1)) / Math.log(k))
  return Number.parseFloat((bytes / k ** i).toFixed(2)).toString()
}

function toGb(bytes: number): string {
  return `${(bytes / 1024 ** 3).toFixed(1)} GB`
}

function Row({ label, value }: { label: string, value: React.ReactNode }) {
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
  const { t } = useTranslation()
  if (totalBytes === 0) { return null }

  // "Other" = neither used nor available (kernel, cache, hardware reserved etc.)
  const otherBytes = Math.max(0, totalBytes - usedBytes - availableBytes)

  const usedPct = Math.min(100, (usedBytes / totalBytes) * 100)
  const availablePct = Math.min(100 - usedPct, (availableBytes / totalBytes) * 100)
  const otherPct = Math.max(0, 100 - usedPct - availablePct)

  const segments = [
    { pct: usedPct, color: 'bg-primary', label: t('ram.used'), value: toGb(usedBytes) },
    { pct: otherPct, color: 'bg-warning/70', label: t('ram.cached'), value: toGb(otherBytes) },
    { pct: availablePct, color: 'bg-muted', label: t('ram.available'), value: toGb(availableBytes) },
  ]

  return (
    <section className="flex flex-col gap-2 rounded-xl border border-border/70 bg-card p-4">
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
  const { t } = useTranslation()
  const [staticInfo, setStaticInfo] = useState<StaticSystemInfo | null>(null)
  const [liveInfo, setLiveInfo] = useState<LiveSystemInfo | null>(null)
  const [history, setHistory] = useState<ChartPoint[]>([])
  const historyRef = useRef<ChartPoint[]>([])
  const peakRef = useRef<number>(0)
  const [peak, setPeak] = useState<number>(0)

  useEffect(() => {
    getStaticSystemInfo().then(setStaticInfo).catch(console.error)
  }, [])

  useEffect(() => {
    if (!staticInfo) { return }

    const tick = () => {
      getLiveSystemInfo()
        .then((live) => {
          setLiveInfo(live)
          const gb = live.ramUsedBytes / 1024 ** 3
          const next = [...historyRef.current, { value: gb }]
          historyRef.current = next.length > 60 ? next.slice(-60) : next
          setHistory([...historyRef.current])
          if (gb > peakRef.current) {
            peakRef.current = gb
            setPeak(gb)
          }
        })
        .catch(console.error)
    }

    tick()
    const id = setInterval(tick, 1000)
    return () => clearInterval(id)
  }, [staticInfo])

  if (!staticInfo) {
    return (
      <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
        {[1, 2, 3].map(i => (
          <section className="rounded-xl border border-border/70 bg-card p-4" key={i}>
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

  const used = liveInfo?.ramUsedBytes ?? 0
  const available = liveInfo?.ramAvailableBytes ?? 0
  const committed = liveInfo?.ramCommittedBytes ?? 0
  const commitLimit = liveInfo?.ramCommitLimitBytes ?? 0
  const cached = liveInfo?.ramCachedBytes ?? 0
  const compressed = liveInfo?.ramCompressedBytes ?? 0
  const pagedPool = liveInfo?.ramPagedPoolBytes ?? 0
  const nonPagedPool = liveInfo?.ramNonpagedPoolBytes ?? 0

  // Hardware reserved: physical BIOS reservation = WMI total - sysinfo (used + available)
  const sysTotal = used + available
  const hwReserved = sysTotal > 0 ? Math.max(0, total - sysTotal) : 0

  const currentGb = used / 1024 ** 3

  return (
    <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">

      {/* Usage chart with corner labels */}
      <section className="flex flex-col gap-1 rounded-xl border border-border/70 bg-card p-4">
        <div className="flex items-baseline justify-between">
          <span className="text-xs font-medium text-foreground">
            {t('ram.memoryUsage')}
          </span>
          <span className="text-xs tabular-nums text-muted-foreground">
            {currentGb.toFixed(1)}
            {' GB ('}
            {t('ram.peak')}
            {': '}
            {peak.toFixed(1)}
            {' GB)'}
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
      <section className="flex flex-col gap-3 rounded-xl border border-border/70 bg-card p-4">
        <h3 className="text-sm font-medium text-foreground">{t('ram.info')}</h3>
        <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
          <Row
            label={t('ram.used')}
            value={(
              <span className="tabular-nums">
                {formatBytes(used)}
                <span className="ml-1 font-normal text-muted-foreground">
                  (
                  {formatBytesNumberOnly(compressed, used)}
                  {' '}
                  {t('ram.compressed')}
                  )
                </span>
              </span>
            )}
          />
          <Row
            label={t('ram.available')}
            value={<span className="tabular-nums">{formatBytes(available)}</span>}
          />
          <Row
            label={t('ram.committed')}
            value={(
              <span className="tabular-nums">
                {formatBytes(committed)}
                {' / '}
                {formatBytes(commitLimit > 0 ? commitLimit : total)}
              </span>
            )}
          />
          <Row
            label={t('ram.cached')}
            value={<span className="tabular-nums">{formatBytes(cached)}</span>}
          />
          <Row
            label={t('ram.pagedPool')}
            value={<span className="tabular-nums">{formatBytes(pagedPool)}</span>}
          />
          <Row
            label={t('ram.nonPagedPool')}
            value={<span className="tabular-nums">{formatBytes(nonPagedPool)}</span>}
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
            value={<span className="tabular-nums">{formatBytes(hwReserved)}</span>}
          />
        </div>
      </section>
    </section>
  )
}
