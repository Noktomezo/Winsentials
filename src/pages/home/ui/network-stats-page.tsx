import type { ReactNode } from 'react'
import type { NetworkAdapterInfo, NetworkIfaceStats } from '@/entities/system-info/model/types'
import { useNavigate, useParams, useRouter } from '@tanstack/react-router'
import { useTranslation } from 'react-i18next'
import { useDeviceInventory, useLiveNetwork } from '@/entities/system-info/model/live-system-store'
import { formatRateLocalized } from '@/shared/lib/format-size'
import { useRouteIntentPreload } from '@/shared/lib/hooks/use-route-intent-preload'
import { networkAdapterToParam } from '@/shared/lib/mount-utils'
import { Button, Skeleton } from '@/shared/ui'
import { LiveChart } from '@/shared/ui/live-chart'

function formatRate(bytesPerSec: number, locale: string, t: ReturnType<typeof useTranslation>['t']): string {
  return formatRateLocalized(bytesPerSec, { locale, t })
}

function getRateUnit(bytesPerSec: number, t: ReturnType<typeof useTranslation>['t']): { divisor: number, label: string } {
  if (bytesPerSec === 0) {
    return { divisor: 1, label: `${t('format.byte')}${t('format.perSecond')}` }
  }

  const k = 1024
  const units = [
    `${t('format.byte')}${t('format.perSecond')}`,
    `${t('format.kilobyte')}${t('format.perSecond')}`,
    `${t('format.megabyte')}${t('format.perSecond')}`,
    `${t('format.gigabyte')}${t('format.perSecond')}`,
    `${t('format.terabyte')}${t('format.perSecond')}`,
  ]
  const index = Math.min(Math.floor(Math.log(bytesPerSec) / Math.log(k)), units.length - 1)

  return {
    divisor: k ** index,
    label: units[index],
  }
}

function ceilToNiceNumber(value: number): number {
  if (value <= 0) { return 0 }
  const magnitude = 10 ** Math.floor(Math.log10(value))
  return Math.ceil(value / magnitude) * magnitude
}

interface LiveNetworkErrorStateProps {
  message: string
  onRetry: () => void
}

function getLiveAdapter(liveInfo: NetworkIfaceStats[] | null, adapter: NetworkAdapterInfo): NetworkIfaceStats | null {
  return liveInfo?.find(entry => entry.name === adapter.name) ?? null
}

interface RowProps {
  label: string
  value: ReactNode
}

interface NetworkAdapterCardProps {
  adapter: NetworkAdapterInfo
  traffic: NetworkIfaceStats | null
}

function Row({ label, value }: RowProps) {
  return (
    <div className="flex items-center justify-between gap-4">
      <span className="text-xs text-muted-foreground">{label}</span>
      <span className="text-right text-xs font-medium text-foreground">{value}</span>
    </div>
  )
}

function EmptyValue() {
  return <span className="text-muted-foreground">-</span>
}

function LiveNetworkLoadingState() {
  return (
    <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
      {Array.from({ length: 3 }).map((_, i) => (
        <section className="rounded-xl border border-border/70 bg-card p-4" key={i}>
          <Skeleton className="mb-3 h-4 w-40" />
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

function LiveNetworkErrorState({ message, onRetry }: LiveNetworkErrorStateProps) {
  const { t } = useTranslation()

  return (
    <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
      <section className="flex flex-col gap-3 rounded-xl border border-border/70 bg-card p-4">
        <p className="text-sm text-muted-foreground">{message}</p>
        <div>
          <Button onClick={onRetry} size="sm" type="button" variant="outline">
            {t('tweaks.actions.retry')}
          </Button>
        </div>
      </section>
    </section>
  )
}

function NetworkAdapterCard({ adapter, traffic }: NetworkAdapterCardProps) {
  const { t, i18n } = useTranslation()
  const navigate = useNavigate()
  const router = useRouter()
  const preloadRouteIntent = useRouteIntentPreload()
  const adapterName = networkAdapterToParam(adapter.name)

  function handlePointerIntent() {
    preloadRouteIntent(() => router.preloadRoute({ to: '/network-stats/$adapterName', params: { adapterName } }))
  }

  return (
    <button
      className="flex w-full flex-col gap-3 rounded-xl border border-border/70 bg-card p-4 text-left transition-colors hover:border-primary/40 hover:bg-accent/20"
      onFocus={handlePointerIntent}
      onMouseEnter={handlePointerIntent}
      onClick={() => {
        void navigate({ to: '/network-stats/$adapterName', params: { adapterName } })
      }}
      type="button"
    >
      <div className="flex items-start justify-between gap-4">
        <div className="space-y-1">
          <h3 className="text-sm font-medium text-foreground">{adapter.name}</h3>
          <p className="text-xs text-muted-foreground">{adapter.adapterDescription}</p>
          {adapter.isWifi && adapter.ssid && (
            <p className="text-xs text-primary">{adapter.ssid}</p>
          )}
        </div>
        <span className="text-xs tabular-nums text-muted-foreground">
          {t('networkStats.openAdapter')}
        </span>
      </div>

      <div className="grid grid-cols-2 gap-3 text-xs">
        <div className="space-y-1">
          <span className="text-muted-foreground">{t('networkStats.receive')}</span>
          <div className="font-medium text-foreground">{formatRate(traffic?.rxBytesPerSec ?? 0, i18n.language, t)}</div>
        </div>
        <div className="space-y-1">
          <span className="text-muted-foreground">{t('networkStats.send')}</span>
          <div className="font-medium text-foreground">{formatRate(traffic?.txBytesPerSec ?? 0, i18n.language, t)}</div>
        </div>
      </div>
    </button>
  )
}

export function NetworkStatsPage() {
  const { t, i18n } = useTranslation()
  const params = useParams({ strict: false })
  const adapterParam = params.adapterName !== undefined ? decodeURIComponent(params.adapterName) : null
  const { data: liveInfo, error: liveError, isFetching, retry, throughputHistory: storeThroughput } = useLiveNetwork()
  const {
    data: deviceInventory,
    error: inventoryError,
    isFetching: inventoryFetching,
    retry: retryInventory,
  } = useDeviceInventory()

  if (deviceInventory === null) {
    if (inventoryError) {
      return (
        <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
          <section className="flex flex-col gap-3 rounded-xl border border-border/70 bg-card p-4">
            <p className="text-sm text-muted-foreground">{t('networkStats.loadError')}</p>
            <div>
              <Button onClick={retryInventory} size="sm" type="button" variant="outline">
                {t('tweaks.actions.retry')}
              </Button>
            </div>
          </section>
        </section>
      )
    }

    if (inventoryFetching) {
      return (
        <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
          {Array.from({ length: 3 }).map((_, i) => (
            <section className="rounded-xl border border-border/70 bg-card p-4" key={i}>
              <Skeleton className="mb-3 h-4 w-40" />
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
  }

  const adapters = deviceInventory?.networkAdapters ?? []
  const selectedAdapter = adapterParam !== null
    ? adapters.find(adapter => adapter.name === adapterParam) ?? null
    : null

  if (liveInfo === null && isFetching) {
    return <LiveNetworkLoadingState />
  }

  if (liveInfo === null && liveError) {
    return <LiveNetworkErrorState message={t('networkStats.liveLoadError')} onRetry={retry} />
  }

  if (adapterParam !== null && deviceInventory !== null && !selectedAdapter) {
    return (
      <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
        <section className="rounded-xl border border-border/70 bg-card p-4">
          <span className="text-xs text-muted-foreground">{t('networkStats.adapterUnavailable')}</span>
        </section>
      </section>
    )
  }

  if (selectedAdapter) {
    const traffic = getLiveAdapter(liveInfo, selectedAdapter)
    const currentThroughput = (traffic?.rxBytesPerSec ?? 0) + (traffic?.txBytesPerSec ?? 0)
    const throughputHistory = storeThroughput[selectedAdapter.name] ?? []
    const peakBytes = Math.max(currentThroughput, ...throughputHistory)
    const unit = getRateUnit(peakBytes, t)
    const roundedPeak = ceilToNiceNumber(peakBytes / unit.divisor)
    const chartMax = roundedPeak > 0 ? roundedPeak : 1
    const chartData = throughputHistory.map(v => ({ value: v / unit.divisor }))

    return (
      <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
        <section className="flex flex-col gap-1 rounded-xl border border-border/70 bg-card p-4">
          <div className="flex items-baseline justify-between gap-4">
            <span className="text-xs font-medium text-foreground">{t('networkStats.throughput')}</span>
            <span className="text-xs tabular-nums text-muted-foreground">
              {t('networkStats.max60')}
              {': '}
              {roundedPeak > 0 ? `${roundedPeak} ${unit.label}` : formatRate(0, i18n.language, t)}
            </span>
          </div>
          <LiveChart data={chartData} height={96} unit={` ${unit.label}`} yDomain={[0, chartMax]} />
          <div className="flex items-baseline justify-between">
            <span className="text-xs text-muted-foreground">{t('ram.seconds', { n: 60 })}</span>
            <span className="text-xs tabular-nums text-muted-foreground">0</span>
          </div>
        </section>

        <section className="flex flex-col gap-3 rounded-xl border border-border/70 bg-card p-4">
          <h3 className="text-sm font-medium text-foreground">{t('networkStats.info')}</h3>
          <div className="grid grid-cols-1 gap-2 sm:grid-cols-2">
            <Row label={t('networkStats.send')} value={formatRate(traffic?.txBytesPerSec ?? 0, i18n.language, t)} />
            <Row label={t('networkStats.receive')} value={formatRate(traffic?.rxBytesPerSec ?? 0, i18n.language, t)} />
            <Row label={t('networkStats.adapter')} value={selectedAdapter.name} />
            <Row label={t('networkStats.adapterDescription')} value={selectedAdapter.adapterDescription} />
            <Row label={t('networkStats.dnsName')} value={selectedAdapter.dnsName ?? <EmptyValue />} />
            <Row label={t('networkStats.connectionType')} value={selectedAdapter.connectionType} />
            <Row
              label={t('networkStats.ipv4')}
              value={selectedAdapter.ipv4Addresses.length > 0 ? selectedAdapter.ipv4Addresses.join(', ') : <EmptyValue />}
            />
            <Row
              label={t('networkStats.ipv6')}
              value={selectedAdapter.ipv6Addresses.length > 0 ? selectedAdapter.ipv6Addresses.join(', ') : <EmptyValue />}
            />
            {selectedAdapter.isWifi && (
              <Row label={t('networkStats.ssid')} value={selectedAdapter.ssid ?? <EmptyValue />} />
            )}
            {selectedAdapter.isWifi && (
              <Row
                label={t('networkStats.signal')}
                value={selectedAdapter.signalPercent !== null ? `${selectedAdapter.signalPercent}%` : <EmptyValue />}
              />
            )}
          </div>
        </section>
      </section>
    )
  }

  return (
    <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
      {adapters.length === 0
        ? (
            <section className="rounded-xl border border-border/70 bg-card p-4">
              <span className="text-xs text-muted-foreground">{t('home.noActivity')}</span>
            </section>
          )
        : (
            <div className="grid grid-cols-1 gap-4 xl:grid-cols-2">
              {adapters.map(adapter => (
                <NetworkAdapterCard
                  adapter={adapter}
                  key={adapter.index}
                  traffic={getLiveAdapter(liveInfo, adapter)}
                />
              ))}
            </div>
          )}
    </section>
  )
}
