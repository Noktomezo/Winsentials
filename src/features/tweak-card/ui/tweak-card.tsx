import type { LucideIcon } from 'lucide-react'
import type { TweakMeta, WindowsVersion } from '@/entities/tweak/model/types'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { ArrowLeftRight, BellOff, CircleAlert, Clock3, Cpu, ExternalLink, FileType, Gauge, HardDrive, History, House, Images, Info, Menu, Network, PanelsTopLeft, Power, RotateCcw, Shield, ShieldOff, TextCursor, TriangleAlert, Type, Zap } from 'lucide-react'
import { Trans, useTranslation } from 'react-i18next'
import { toast } from '@/shared/lib/toast'
import { cn } from '@/shared/lib/utils'
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

function TweakMetaPill({
  className,
  children,
}: React.PropsWithChildren<{ className?: string }>) {
  return (
    <span
      className={cn(
        'inline-flex items-center justify-center text-muted-foreground',
        className,
      )}
    >
      {children}
    </span>
  )
}

const TWEAK_ICONS: Record<string, LucideIcon> = {
  classic_context_menu: Menu,
  fast_taskbar_thumbnails: PanelsTopLeft,
  faster_cursor_blink_rate: TextCursor,
  hide_gallery_navigation_pane: Images,
  hide_home_navigation_pane: House,
  hide_network_navigation_pane: Network,
  disable_8dot3_name_creation: FileType,
  disable_recent_items_and_frequent_places: History,
  open_explorer_to_this_pc: HardDrive,
  unlock_lock_screen_timeout_setting: Clock3,
  remove_shortcut_arrows: ExternalLink,
  remove_shortcut_suffix: Type,
  disable_security_center_notifications: BellOff,
  disable_open_file_warning: ShieldOff,
  disable_user_account_control: Shield,
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
    <article className="rounded-xl border border-border/70 bg-card p-4">
      <div className="min-w-0 flex items-start gap-3">
        <span className="flex size-9 shrink-0 items-center justify-center rounded-lg bg-accent/60 text-accent-foreground">
          <Icon className="size-4" />
        </span>
        <div className="min-w-0 flex-1 space-y-1">
          <div className="flex items-start justify-between gap-4">
            <div className="min-w-0 flex flex-wrap items-center gap-2">
              <h2 className="text-sm font-medium text-foreground">
                {t(tweak.name)}
              </h2>
              <Tooltip>
                <TooltipTrigger asChild>
                  <button
                    aria-label={t('tweaks.actions.showDetails')}
                    className="inline-flex items-center justify-center text-muted-foreground transition-colors hover:text-accent-foreground"
                    type="button"
                  >
                    <TweakMetaPill className="text-inherit">
                      <Info aria-hidden className="size-3.5" />
                    </TweakMetaPill>
                  </button>
                </TooltipTrigger>
                <TooltipContent className="max-w-80 text-pretty" sideOffset={8}>
                  {t(tweak.detailDescription)}
                </TooltipContent>
              </Tooltip>
              {tweak.requiresAction.type === 'restart_app' && tweak.requiresAction.appName === 'Explorer' && (
                <Tooltip>
                  <TooltipTrigger asChild>
                    <button
                      aria-label={t('tweaks.requires.restartExplorer')}
                      className="inline-flex items-center justify-center text-primary/80 transition-colors hover:text-primary"
                      type="button"
                    >
                      <TweakMetaPill className="text-inherit">
                        <RotateCcw aria-hidden className="size-3.5" />
                      </TweakMetaPill>
                    </button>
                  </TooltipTrigger>
                  <TooltipContent sideOffset={8}>
                    {t('tweaks.requires.restartExplorer')}
                  </TooltipContent>
                </Tooltip>
              )}
              {tweak.requiresAction.type === 'restart_pc' && (
                <Tooltip>
                  <TooltipTrigger asChild>
                    <button
                      aria-label={t('tweaks.requires.restartPc')}
                      className="inline-flex items-center justify-center text-primary/80 transition-colors hover:text-primary"
                      type="button"
                    >
                      <TweakMetaPill className="text-inherit">
                        <Power aria-hidden className="size-3.5" />
                      </TweakMetaPill>
                    </button>
                  </TooltipTrigger>
                  <TooltipContent sideOffset={8}>
                    {t('tweaks.requires.restartPc')}
                  </TooltipContent>
                </Tooltip>
              )}
              {tweak.risk !== 'none' && tweak.riskDescription && (
                <Tooltip>
                  <TooltipTrigger asChild>
                    <button
                      aria-label={t('tweaks.actions.showRiskDetails')}
                      className="inline-flex items-center justify-center text-amber-700 transition-colors hover:text-amber-800 dark:text-amber-300 dark:hover:text-amber-200"
                      type="button"
                    >
                      <TweakMetaPill className="text-inherit">
                        <TriangleAlert aria-hidden className="size-3.5" />
                      </TweakMetaPill>
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
                      aria-label={t('tweaks.actions.showConflictDetails')}
                      className="inline-flex items-center justify-center text-amber-700 transition-colors hover:text-amber-800 dark:text-amber-300 dark:hover:text-amber-200"
                      type="button"
                    >
                      <TweakMetaPill className="text-inherit">
                        <ArrowLeftRight aria-hidden className="size-3.5" />
                      </TweakMetaPill>
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
              {isUnsupported && tweak.minOsBuild && (
                <Tooltip>
                  <TooltipTrigger asChild>
                    <button
                      aria-label={t('tweaks.requires.windowsBuild', { build: minBuild })}
                      className="inline-flex items-center justify-center text-amber-700 dark:text-amber-300"
                      type="button"
                    >
                      <TweakMetaPill className="text-inherit">
                        <CircleAlert aria-hidden className="size-3.5" />
                      </TweakMetaPill>
                    </button>
                  </TooltipTrigger>
                  <TooltipContent sideOffset={8}>
                    {t('tweaks.requires.windowsBuild', { build: minBuild })}
                  </TooltipContent>
                </Tooltip>
              )}
            </div>
            {tweak.control.kind === 'toggle' && (
              <div className="flex shrink-0 items-center gap-2 self-start">
                <Switch
                  aria-label={t(tweak.name)}
                  checked={isEnabled}
                  disabled={isPending || isUnsupported}
                  onCheckedChange={onToggle}
                />
                <Tooltip>
                  <TooltipTrigger asChild>
                    <button
                      aria-label={t('tweaks.actions.resetToDefault')}
                      className="inline-flex cursor-pointer items-center justify-center rounded-md p-1 text-muted-foreground transition-colors hover:bg-destructive/15 hover:text-destructive disabled:pointer-events-none disabled:opacity-50"
                      disabled={isPending || isAtDefault || isUnsupported}
                      onClick={() => onToggle(isDefaultEnabled)}
                      type="button"
                    >
                      <RotateCcw className="size-3.5" />
                    </button>
                  </TooltipTrigger>
                  <TooltipContent sideOffset={8}>
                    {t('tweaks.actions.resetToDefault')}
                  </TooltipContent>
                </Tooltip>
              </div>
            )}
          </div>
          <p className="text-xs leading-5 text-muted-foreground">
            {t(tweak.shortDescription)}
          </p>
        </div>
      </div>
    </article>
  )
}

export function TweakCardSkeleton() {
  return (
    <article className="rounded-xl border border-border/70 bg-card p-4">
      <div className="min-w-0 flex items-start gap-3">
        <Skeleton className="size-9 shrink-0 rounded-lg" />
        <div className="min-w-0 flex-1 space-y-1">
          <div className="flex items-start justify-between gap-4">
            <div className="min-w-0 flex flex-wrap items-center gap-2">
              <Skeleton className="h-4 w-40" />
              <Skeleton className="size-3.5 rounded-full" />
              <Skeleton className="size-3.5 rounded-full" />
              <Skeleton className="size-3.5 rounded-full" />
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
    </article>
  )
}
