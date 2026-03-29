import type { LucideIcon } from 'lucide-react'
import type { TweakMeta, WindowsVersion } from '@/entities/tweak/model/types'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { AlertTriangle, AppWindow, BellOff, CircleAlert, Clock3, Cpu, ExternalLink, FileType, Gauge, HardDrive, History, House, Images, Info, LogOut, Menu, Network, PanelsTopLeft, Power, RotateCcw, Settings, Shield, ShieldOff, TextCursor, TriangleAlert, Type, Usb, Zap } from 'lucide-react'
import { Trans, useTranslation } from 'react-i18next'
import { toast } from '@/shared/lib/toast'
import { MarqueeText } from '@/shared/ui/marquee-text'
import { Skeleton } from '@/shared/ui/skeleton'
import { Switch } from '@/shared/ui/switch'
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/shared/ui/tooltip'

interface TweakCardProps {
  currentBuild: WindowsVersion
  isPending?: boolean
  onToggle: (checked: boolean) => void
  tweak: TweakMeta
}

const TWEAK_ICONS: Record<string, LucideIcon> = {
  classic_context_menu: Menu,
  fast_taskbar_thumbnails: PanelsTopLeft,
  faster_cursor_blink_rate: TextCursor,
  hide_gallery_navigation_pane: Images,
  hide_home_navigation_pane: House,
  hide_network_navigation_pane: Network,
  disable_8dot3_name_creation: FileType,
  disable_startup_delay: Power,
  disable_recent_items_and_frequent_places: History,
  open_explorer_to_this_pc: HardDrive,
  unlock_lock_screen_timeout_setting: Clock3,
  remove_shortcut_arrows: ExternalLink,
  remove_shortcut_suffix: Type,
  disable_autoplay: Usb,
  disable_security_center_notifications: BellOff,
  disable_open_file_warning: ShieldOff,
  disable_user_account_control: Shield,
  disable_ncsi_active_probing: CircleAlert,
  disable_fault_tolerant_heap: Gauge,
  optimize_mmcss: Zap,
  enable_bbr2_congestion_control: Zap,
  disable_qos_bandwidth_limit: Gauge,
  enable_network_offloading_rss: Cpu,
}

const COPYABLE_RISK_COMMANDS: Record<string, string> = {
  disable_user_account_control: 'runas /trustlevel:0x20000 "program.exe"',
}

function formatMinBuild(tweak: TweakMeta) {
  if (typeof tweak.minOsBuild !== 'number') {
    return null
  }

  return typeof tweak.minOsUbr === 'number'
    ? `${tweak.minOsBuild}.${tweak.minOsUbr}`
    : `${tweak.minOsBuild}`
}

function isBelowMinBuild(currentBuild: WindowsVersion, tweak: TweakMeta) {
  if (typeof tweak.minOsBuild !== 'number') {
    return false
  }

  if (currentBuild.build !== tweak.minOsBuild) {
    return currentBuild.build < tweak.minOsBuild
  }

  if (typeof tweak.minOsUbr !== 'number') {
    return false
  }

  return currentBuild.ubr < tweak.minOsUbr
}

function metadataChipClassName(tone: 'default' | 'warning' | 'info' = 'default') {
  if (tone === 'warning') {
    return 'border-amber-500/25 bg-amber-500/10 text-amber-700 dark:text-amber-300'
  }

  if (tone === 'info') {
    return 'border-primary/20 bg-primary/8 text-primary'
  }

  return 'border-border/70 bg-accent/45 text-muted-foreground'
}

function requiresActionBadge(tweak: TweakMeta, t: ReturnType<typeof useTranslation>['t']): { icon: LucideIcon, label: string, tooltip: string } | null {
  switch (tweak.requiresAction.type) {
    case 'none':
      return null
    case 'logout':
      return {
        icon: LogOut,
        label: t('tweaks.meta.logout'),
        tooltip: t('tweaks.prompts.logout'),
      }
    case 'restart_pc':
      return {
        icon: Power,
        label: t('tweaks.meta.restart'),
        tooltip: t('tweaks.prompts.restartPc'),
      }
    case 'restart_service':
      return {
        icon: Settings,
        label: tweak.requiresAction.serviceName,
        tooltip: t('tweaks.prompts.restartService', { serviceName: tweak.requiresAction.serviceName }),
      }
    case 'restart_app':
      return {
        icon: AppWindow,
        label: tweak.requiresAction.appName,
        tooltip: t('tweaks.prompts.restartApp', { appName: tweak.requiresAction.appName }),
      }
    case 'restart_device':
      return {
        icon: Usb,
        label: tweak.requiresAction.deviceName,
        tooltip: t('tweaks.prompts.restartDevice', { deviceName: tweak.requiresAction.deviceName }),
      }
  }
}

function MetadataChip({ children, tone = 'default', icon: Icon }: React.PropsWithChildren<{ tone?: 'default' | 'warning' | 'info', icon?: LucideIcon }>) {
  return (
    <span className={`inline-flex items-center rounded-md border px-2 py-0.75 text-[10px] font-medium ${metadataChipClassName(tone)}`}>
      {Icon && <Icon className="mr-1 size-[11px]" />}
      {children}
    </span>
  )
}

export function TweakCard({
  currentBuild,
  isPending = false,
  onToggle,
  tweak,
}: TweakCardProps) {
  const { t } = useTranslation()
  const Icon = TWEAK_ICONS[tweak.id] ?? Info
  const isEnabled = tweak.currentValue === 'enabled'
  const isDefaultEnabled = tweak.defaultValue === 'enabled'
  const isAtDefault = tweak.currentValue === tweak.defaultValue
  const isUnsupported = isBelowMinBuild(currentBuild, tweak)
  const minBuild = formatMinBuild(tweak)
  const copyableRiskCommand = COPYABLE_RISK_COMMANDS[tweak.id]
  const conflicts = tweak.conflicts ?? []
  const requiresBadge = requiresActionBadge(tweak, t)

  const handleCopyRiskCommand = async () => {
    if (!copyableRiskCommand) {
      return
    }

    try {
      await writeText(copyableRiskCommand)
      toast.success(t('tweaks.success.copyCommand'))
    }
    catch {
      toast.error(t('tweaks.errors.copyCommand'))
    }
  }

  return (
    <article className="h-full rounded-xl border border-border/70 bg-card p-4" data-marquee-group="true">
      <div className="flex h-full min-w-0 flex-col">
        <div className="min-w-0 flex items-start gap-3">
          <span className="flex size-9 shrink-0 items-center justify-center rounded-lg bg-accent/60 text-accent-foreground">
            <Icon className="size-4" />
          </span>
          <div className="min-w-0 flex-1 space-y-1">
            <div className="flex items-start justify-between gap-4">
              <div className="min-w-0 flex flex-1 items-start">
                <h2 className="min-w-0 flex-1 text-sm font-medium text-foreground">
                  <MarqueeText className="block max-w-full" text={t(tweak.name)} />
                </h2>
              </div>
              {tweak.control.kind === 'toggle' && (
                <div className="flex shrink-0 items-center gap-2 self-start">
                  <Switch
                    aria-label={t(tweak.name)}
                    checked={isEnabled}
                    disabled={isPending || isUnsupported}
                    onCheckedChange={onToggle}
                  />
                  <button
                    aria-label={t('tweaks.actions.resetToDefault')}
                    className="inline-flex cursor-pointer items-center justify-center rounded-md p-1 text-muted-foreground transition-colors hover:bg-destructive/15 hover:text-destructive disabled:pointer-events-none disabled:opacity-50"
                    disabled={isPending || isAtDefault || isUnsupported}
                    onClick={() => onToggle(isDefaultEnabled)}
                    type="button"
                  >
                    <RotateCcw className="size-3.5" />
                  </button>
                </div>
              )}
            </div>
            <p className="text-xs leading-5 text-muted-foreground">
              {t(tweak.shortDescription)}
            </p>
          </div>
        </div>

        <footer className="mt-auto border-t border-border/40 pt-2">
          <div className="flex flex-wrap gap-2">
            <Tooltip>
              <TooltipTrigger asChild>
                <button
                  aria-label={t('tweaks.meta.details')}
                  className={`inline-flex cursor-help items-center rounded-md border px-2 py-0.75 text-[10px] font-medium ${metadataChipClassName()}`}
                  type="button"
                >
                  <Info className="mr-1 size-[11px]" />
                  {t('tweaks.meta.details')}
                </button>
              </TooltipTrigger>
              <TooltipContent className="max-w-80 text-pretty" sideOffset={8}>
                {t(tweak.detailDescription)}
              </TooltipContent>
            </Tooltip>

            {requiresBadge && (
              <Tooltip>
                <TooltipTrigger asChild>
                  <button
                    aria-label={requiresBadge.tooltip}
                    className="cursor-help"
                    type="button"
                  >
                    <MetadataChip icon={requiresBadge.icon} tone="info">
                      {requiresBadge.label}
                    </MetadataChip>
                  </button>
                </TooltipTrigger>
                <TooltipContent className="max-w-80 text-pretty whitespace-pre-line" sideOffset={8}>
                  {requiresBadge.tooltip}
                </TooltipContent>
              </Tooltip>
            )}

            {tweak.risk !== 'none' && tweak.riskDescription && (
              <Tooltip>
                <TooltipTrigger asChild>
                  <button
                    aria-label={t('tweaks.meta.risk')}
                    className="cursor-help"
                    type="button"
                  >
                    <MetadataChip icon={TriangleAlert} tone="warning">{t('tweaks.meta.risk')}</MetadataChip>
                  </button>
                </TooltipTrigger>
                <TooltipContent className="max-w-80 text-pretty whitespace-pre-line" sideOffset={8}>
                  <Trans
                    components={{
                      code: (
                        <code
                          aria-label={t('tweaks.actions.copyCommand')}
                          className="mt-2 block w-fit cursor-copy rounded-[4px] border border-border/70 bg-accent px-2 py-1 font-mono text-xs font-medium text-foreground shadow-xs transition-colors hover:bg-accent/80"
                          onClick={() => {
                            void handleCopyRiskCommand()
                          }}
                          onKeyDown={(event) => {
                            if (event.key === 'Enter' || event.key === ' ') {
                              event.preventDefault()
                              void handleCopyRiskCommand()
                            }
                          }}
                          role="button"
                          tabIndex={0}
                        />
                      ),
                    }}
                    i18nKey={tweak.riskDescription}
                  />
                </TooltipContent>
              </Tooltip>
            )}

            {conflicts.length > 0 && (
              <Tooltip>
                <TooltipTrigger asChild>
                  <button
                    aria-label={t('tweaks.meta.conflicts')}
                    className="cursor-help"
                    type="button"
                  >
                    <MetadataChip icon={AlertTriangle} tone="warning">{t('tweaks.meta.conflicts')}</MetadataChip>
                  </button>
                </TooltipTrigger>
                <TooltipContent className="max-w-80 text-pretty whitespace-pre-line" sideOffset={8}>
                  {conflicts.length === 1
                    ? <p>{t(conflicts[0].description)}</p>
                    : (
                        <ul className="list-disc space-y-1 pl-4">
                          {conflicts.map(conflict => (
                            <li key={conflict.description}>{t(conflict.description)}</li>
                          ))}
                        </ul>
                      )}
                </TooltipContent>
              </Tooltip>
            )}

            {isUnsupported && minBuild && (
              <Tooltip>
                <TooltipTrigger asChild>
                  <button
                    aria-label={t('tweaks.requires.windowsBuild', { build: minBuild })}
                    className="cursor-help"
                    type="button"
                  >
                    <MetadataChip icon={CircleAlert} tone="warning">{t('tweaks.requires.windowsBuild', { build: minBuild })}</MetadataChip>
                  </button>
                </TooltipTrigger>
                <TooltipContent sideOffset={8}>
                  {t('tweaks.requires.windowsBuild', { build: minBuild })}
                </TooltipContent>
              </Tooltip>
            )}
          </div>
        </footer>
      </div>
    </article>
  )
}

export function TweakCardSkeleton() {
  return (
    <article className="h-full rounded-xl border border-border/70 bg-card p-4">
      <div className="flex h-full min-w-0 flex-col">
        <div className="min-w-0 flex items-start gap-3">
          <Skeleton className="size-9 shrink-0 rounded-lg" />
          <div className="min-w-0 flex-1 space-y-1">
            <div className="flex items-start justify-between gap-4">
              <div className="min-w-0 flex-1">
                <Skeleton className="h-4 w-40" />
              </div>
              <div className="flex shrink-0 items-center gap-2 self-start">
                <Skeleton className="h-6 w-11 rounded-full" />
                <Skeleton className="size-6 rounded-md" />
              </div>
            </div>
            <div className="space-y-1.5">
              <Skeleton className="h-3 w-full max-w-xl" />
              <Skeleton className="h-3 w-full max-w-md" />
            </div>
          </div>
        </div>
        <div className="mt-auto border-t border-border/40 pt-2">
          <div className="flex flex-wrap gap-2">
            <Skeleton className="h-5 w-20 rounded-md" />
            <Skeleton className="h-5 w-30 rounded-md" />
            <Skeleton className="h-5 w-16 rounded-md" />
          </div>
        </div>
      </div>
    </article>
  )
}
