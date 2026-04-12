import type { LucideIcon } from 'lucide-react'
import type { CSSProperties } from 'react'
import type { TweakMeta, WindowsVersion } from '@/entities/tweak/model/types'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import {
  AlertTriangle,
  BellOff,
  CircleAlert,
  Clock3,
  CloudOff,
  Copy,
  Cpu,
  ExternalLink,
  EyeOff,
  FileSearch,
  FileType,
  Gamepad2,
  Gauge,
  HardDrive,
  History,
  House,
  Images,
  Info,
  Keyboard,
  ListX,
  LogOut,
  MapPinned,
  MemoryStick,
  Menu,
  Mouse,
  MousePointer2,
  Network,
  PanelsTopLeft,
  Power,
  RotateCcw,
  Settings,
  Shield,
  ShieldOff,
  Terminal,
  TextCursor,
  TriangleAlert,
  Type,
  Usb,
  Zap,
} from 'lucide-react'
import { forwardRef } from 'react'
import { Trans, useTranslation } from 'react-i18next'
import { toast } from '@/shared/lib/toast'
import { cn } from '@/shared/lib/utils'
import { Button } from '@/shared/ui/button'
import { MarqueeText } from '@/shared/ui/marquee-text'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/shared/ui/select'
import { Skeleton } from '@/shared/ui/skeleton'
import { LabeledSwitch } from '@/shared/ui/switch'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/shared/ui/tooltip'

interface TweakCardProps {
  currentBuild: WindowsVersion
  currentInstalledMemoryBytes?: number | null
  isPending?: boolean
  onApplyValue: (value: string) => void
  tweak: TweakMeta
}

const BYTES_PER_GIB = 1024 ** 3

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
  disable_ndu: Network,
  configure_kernel_timing_chain: Clock3,
  disable_fault_tolerant_heap: Gauge,
  disable_game_dvr: Gamepad2,
  disable_telemetry_scheduled_tasks: ListX,
  disable_cloud_sync: CloudOff,
  disable_input_data_collection: Keyboard,
  disable_inventory_collector: FileSearch,
  disable_location_data_collection: MapPinned,
  disable_targeted_advertising: BellOff,
  disable_dotnet_telemetry: FileType,
  disable_powershell_telemetry: Terminal,
  disable_windows_error_reporting: TriangleAlert,
  disable_windows_telemetry: EyeOff,
  svchost_split_threshold: MemoryStick,
  csrss_high_priority: Zap,
  disable_mouse_acceleration: Mouse,
  raw_mouse_throttle: MousePointer2,
  optimize_mmcss: Zap,
  fast_keyboard_repeat: Keyboard,
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

function requiredInstalledMemoryGb(tweak: TweakMeta) {
  if (tweak.id === 'svchost_split_threshold') {
    return 8
  }

  return null
}

function dropdownOptionIcon(
  tweakId: string,
  optionValue: string,
): LucideIcon | null {
  if (tweakId === 'fast_keyboard_repeat') {
    switch (optionValue) {
      case 'default':
        return Settings
      case 'balanced':
        return Gauge
      case 'fast':
        return Keyboard
      case 'ultra_fast':
        return Zap
      default:
        return null
    }
  }

  if (tweakId === 'disable_cloud_sync') {
    switch (optionValue) {
      case 'default':
        return Settings
      case 'partial':
        return BellOff
      case 'full':
        return CloudOff
      default:
        return null
    }
  }

  return null
}

function metadataChipClassName(
  tone: 'default' | 'details' | 'action' | 'warning' | 'danger' | 'system' = 'default',
) {
  if (tone === 'details') {
    return 'border-border/60 bg-accent/55 text-muted-foreground'
  }

  if (tone === 'action') {
    return 'border-[color:color-mix(in_oklch,var(--badge-blue)_28%,transparent)] bg-[color:color-mix(in_oklch,var(--badge-blue)_12%,transparent)] text-[var(--badge-blue)]'
  }

  if (tone === 'warning') {
    return 'border-[color:color-mix(in_oklch,var(--badge-yellow)_28%,transparent)] bg-[color:color-mix(in_oklch,var(--badge-yellow)_12%,transparent)] text-[var(--badge-yellow)]'
  }

  if (tone === 'danger') {
    return 'border-[color:color-mix(in_oklch,var(--badge-red)_28%,transparent)] bg-[color:color-mix(in_oklch,var(--badge-red)_12%,transparent)] text-[var(--badge-red)]'
  }

  if (tone === 'system') {
    return 'border-[color:color-mix(in_oklch,var(--badge-purple)_28%,transparent)] bg-[color:color-mix(in_oklch,var(--badge-purple)_12%,transparent)] text-[var(--badge-purple)]'
  }

  return 'border-border/70 bg-secondary text-muted-foreground'
}

function metadataTooltipBorderStyle(
  tone: 'default' | 'details' | 'action' | 'warning' | 'danger' | 'system' = 'default',
): CSSProperties {
  if (tone === 'details') {
    return { '--tooltip-border-color': 'color-mix(in oklch, var(--border) 60%, transparent)' } as CSSProperties
  }

  if (tone === 'action') {
    return { '--tooltip-border-color': 'color-mix(in oklch, var(--badge-blue) 28%, transparent)' } as CSSProperties
  }

  if (tone === 'warning') {
    return { '--tooltip-border-color': 'color-mix(in oklch, var(--badge-yellow) 28%, transparent)' } as CSSProperties
  }

  if (tone === 'danger') {
    return { '--tooltip-border-color': 'color-mix(in oklch, var(--badge-red) 28%, transparent)' } as CSSProperties
  }

  if (tone === 'system') {
    return { '--tooltip-border-color': 'color-mix(in oklch, var(--badge-purple) 28%, transparent)' } as CSSProperties
  }

  return { '--tooltip-border-color': 'color-mix(in oklch, var(--border) 70%, transparent)' } as CSSProperties
}

function requiresActionBadge(
  tweak: TweakMeta,
  t: ReturnType<typeof useTranslation>['t'],
): { icon: LucideIcon, label: string, tooltip: string } | null {
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
        tooltip: t('tweaks.prompts.restartService', {
          serviceName: tweak.requiresAction.serviceName,
        }),
      }
    case 'restart_app':
      return {
        icon: RotateCcw,
        label: tweak.requiresAction.appName,
        tooltip: t('tweaks.prompts.restartApp', {
          appName: tweak.requiresAction.appName,
        }),
      }
    case 'restart_device':
      return {
        icon: Usb,
        label: tweak.requiresAction.deviceName,
        tooltip: t('tweaks.prompts.restartDevice', {
          deviceName: tweak.requiresAction.deviceName,
        }),
      }
  }
}

function MetadataChip({
  children,
  tone = 'default',
  icon: Icon,
}: React.PropsWithChildren<{
  tone?: 'default' | 'details' | 'action' | 'warning' | 'danger' | 'system'
  icon?: LucideIcon
}>) {
  return (
    <span
      className={`inline-flex items-center rounded-[6px] border px-2 py-0.75 text-[10px] font-medium ${metadataChipClassName(tone)}`}
    >
      {Icon && <Icon className="mr-1 size-[11px]" />}
      {children}
    </span>
  )
}

const MetadataChipButton = forwardRef<
  HTMLButtonElement,
  React.PropsWithChildren<React.ComponentProps<'button'> & {
    ariaLabel: string
    tone?: 'default' | 'details' | 'action' | 'warning' | 'danger' | 'system'
    icon?: LucideIcon
  }>
>(({
  ariaLabel,
  children,
  tone = 'default',
  icon,
  className,
  type,
  ...props
}, ref) => {
  return (
    <button
      aria-label={ariaLabel}
      className={cn('cursor-help', className)}
      ref={ref}
      type={type ?? 'button'}
      {...props}
    >
      <MetadataChip icon={icon} tone={tone}>
        {children}
      </MetadataChip>
    </button>
  )
})
MetadataChipButton.displayName = 'MetadataChipButton'

function RiskCodeBlock({
  children,
  copyLabel,
  isCopyable = false,
  onCopy,
}: React.PropsWithChildren<{
  copyLabel?: string
  isCopyable?: boolean
  onCopy?: () => void
}>) {
  if (!isCopyable) {
    return (
      <code className="mt-2 block w-full rounded-md border border-border/70 bg-accent px-3 py-2 font-mono text-xs font-medium text-foreground shadow-xs">
        {children}
      </code>
    )
  }

  return (
    <button
      aria-label={copyLabel}
      className="mt-2 flex w-full items-start gap-3 rounded-md border border-border/70 bg-accent px-3 py-2 text-left font-mono text-xs font-medium text-foreground shadow-xs transition-colors hover:bg-accent/80 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring/60"
      onClick={onCopy}
      type="button"
    >
      <span className="min-w-0 flex-1 break-all">
        {children}
      </span>
      <Copy className="mt-0.5 size-3.5 shrink-0 text-muted-foreground" />
    </button>
  )
}

export function TweakCard({
  currentBuild,
  currentInstalledMemoryBytes = null,
  isPending = false,
  onApplyValue,
  tweak,
}: TweakCardProps) {
  const { t } = useTranslation()
  const Icon = TWEAK_ICONS[tweak.id] ?? Info
  const isEnabled = tweak.currentValue === 'enabled'
  const isAtDefault = tweak.currentValue === tweak.defaultValue
  const isBelowBuildRequirement = isBelowMinBuild(currentBuild, tweak)
  const minInstalledMemoryGb = requiredInstalledMemoryGb(tweak)
  const isBelowMemoryRequirement = minInstalledMemoryGb !== null
    && currentInstalledMemoryBytes !== null
    && currentInstalledMemoryBytes < minInstalledMemoryGb * BYTES_PER_GIB
  const isMemoryRequirementPending = minInstalledMemoryGb !== null
    && currentInstalledMemoryBytes === null
  const isUnsupported = isBelowBuildRequirement
    || isBelowMemoryRequirement
    || isMemoryRequirementPending
  const minBuild = formatMinBuild(tweak)
  const copyableRiskCommand = COPYABLE_RISK_COMMANDS[tweak.id]
  const conflicts = tweak.conflicts ?? []
  const requiresBadge = requiresActionBadge(tweak, t)

  const dropdownOptions
    = tweak.control.kind === 'dropdown'
      ? tweak.currentValue === 'custom'
        ? [
            ...tweak.control.options,
            { label: 'tweaks.meta.customValue', value: 'custom' },
          ]
        : tweak.control.options
      : []
  const selectedDropdownOption
    = tweak.control.kind === 'dropdown'
      ? dropdownOptions.find(option => option.value === tweak.currentValue) ?? null
      : null
  const SelectedDropdownIcon
    = selectedDropdownOption
      ? dropdownOptionIcon(tweak.id, selectedDropdownOption.value)
      : null

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
    <article
      className="h-full rounded-lg border border-border/70 bg-card/95 p-4 shadow-[0_14px_36px_rgb(16_15_15_/_0.08)]"
      data-marquee-group="true"
    >
      <div className="flex h-full min-w-0 flex-col">
        <div className="grid gap-3 sm:grid-cols-[minmax(0,1fr)_auto] sm:gap-4">
          <div className="flex min-h-9 min-w-0 items-center gap-3">
            <span className="ui-soft-surface flex size-9 shrink-0 items-center justify-center rounded-md">
              <Icon className="size-4" />
            </span>
            <h2 className="min-w-0 flex-1 text-sm font-medium leading-5 text-foreground">
              <MarqueeText
                className="block max-w-full"
                text={t(tweak.name)}
              />
            </h2>
          </div>

          <aside className="min-w-0">
            <div className="flex h-9 items-center justify-end gap-2">
              <Button
                aria-label={t('tweaks.actions.resetToDefault')}
                className="size-9 rounded-md !border-border/60 !bg-accent/55 !text-accent-foreground shadow-xs transition-colors hover:!bg-destructive/10 hover:!text-destructive"
                disabled={isPending || isAtDefault || isUnsupported}
                onClick={() => onApplyValue(tweak.defaultValue)}
                size="icon"
                type="button"
                variant="outline"
              >
                <RotateCcw className="size-3.5" />
              </Button>

              {tweak.control.kind === 'toggle' && (
                <LabeledSwitch
                  aria-label={t(tweak.name)}
                  checked={isEnabled}
                  containerClassName="!border-border/60 !bg-accent/55 !text-accent-foreground shadow-xs"
                  disabled={isPending || isUnsupported}
                  labelClassName="!text-accent-foreground"
                  onCheckedChange={checked =>
                    onApplyValue(checked ? 'enabled' : 'disabled')}
                />
              )}
              {tweak.control.kind === 'dropdown' && (
                <Select
                  disabled={isPending || isUnsupported}
                  onValueChange={value => onApplyValue(value)}
                  value={tweak.currentValue}
                >
                  <SelectTrigger className="h-9 min-w-[11rem] justify-between rounded-md !border-border/60 !bg-accent/55 px-3 text-xs font-medium !text-accent-foreground shadow-xs [&_svg]:size-3.5 [&_svg:not([class*='text-'])]:!text-accent-foreground/70">
                    {selectedDropdownOption
                      ? (
                          <span className="flex min-w-0 items-center gap-2">
                            {SelectedDropdownIcon && (
                              <SelectedDropdownIcon className="size-3.5 shrink-0 text-muted-foreground" />
                            )}
                            <span className="truncate">{t(selectedDropdownOption.label)}</span>
                          </span>
                        )
                      : (
                          <SelectValue
                            placeholder={t('tweaks.controls.selectPreset')}
                          />
                        )}
                  </SelectTrigger>
                  <SelectContent
                    align="end"
                    className="min-w-[var(--radix-select-trigger-width)] rounded-[10px] border text-xs font-medium"
                  >
                    {dropdownOptions.map((option) => {
                      const OptionIcon = dropdownOptionIcon(tweak.id, option.value)

                      return (
                        <SelectItem
                          className="min-h-7 py-1 pr-7 pl-2 text-xs font-medium"
                          disabled={option.value === 'custom'}
                          key={option.value}
                          value={option.value}
                        >
                          <span className="flex items-center gap-2">
                            {OptionIcon
                              ? <OptionIcon className="size-3.5 shrink-0 text-muted-foreground" />
                              : null}
                            <span>{t(option.label)}</span>
                          </span>
                        </SelectItem>
                      )
                    })}
                  </SelectContent>
                </Select>
              )}
            </div>
          </aside>
        </div>

        <p className="mt-4 text-xs leading-5 text-muted-foreground">
          {t(tweak.shortDescription)}
        </p>

        <div className="mt-auto pt-4">
          <footer>
            <div className="flex flex-wrap gap-2">
              <Tooltip>
                <TooltipTrigger asChild>
                  <MetadataChipButton
                    ariaLabel={t('tweaks.meta.details')}
                    icon={Info}
                    tone="details"
                  >
                    {t('tweaks.meta.details')}
                  </MetadataChipButton>
                </TooltipTrigger>
                <TooltipContent
                  className="max-w-80 text-pretty"
                  sideOffset={8}
                  style={metadataTooltipBorderStyle('details')}
                >
                  {t(tweak.detailDescription)}
                </TooltipContent>
              </Tooltip>

              {requiresBadge && (
                <Tooltip>
                  <TooltipTrigger asChild>
                    <MetadataChipButton
                      ariaLabel={requiresBadge.tooltip}
                      icon={requiresBadge.icon}
                      tone="action"
                    >
                      {requiresBadge.label}
                    </MetadataChipButton>
                  </TooltipTrigger>
                  <TooltipContent
                    className="max-w-80 text-pretty whitespace-pre-line"
                    sideOffset={8}
                    style={metadataTooltipBorderStyle('action')}
                  >
                    {requiresBadge.tooltip}
                  </TooltipContent>
                </Tooltip>
              )}

              {tweak.risk !== 'none' && tweak.riskDescription && (
                <>
                  <Tooltip>
                    <TooltipTrigger asChild>
                      <MetadataChipButton
                        ariaLabel={t('tweaks.meta.risk')}
                        icon={TriangleAlert}
                        tone="warning"
                      >
                        {t('tweaks.meta.risk')}
                      </MetadataChipButton>
                    </TooltipTrigger>
                    <TooltipContent
                      className="max-w-80 text-pretty whitespace-pre-line"
                      sideOffset={8}
                      style={metadataTooltipBorderStyle('warning')}
                    >
                      <Trans
                        components={{
                          code: (
                            <RiskCodeBlock
                              copyLabel={t('tweaks.actions.copyCommand')}
                              isCopyable={Boolean(copyableRiskCommand)}
                              onCopy={() => {
                                void handleCopyRiskCommand()
                              }}
                            />
                          ),
                        }}
                        i18nKey={tweak.riskDescription}
                      />
                    </TooltipContent>
                  </Tooltip>
                </>
              )}

              {conflicts.length > 0 && (
                <Tooltip>
                  <TooltipTrigger asChild>
                    <MetadataChipButton
                      ariaLabel={t('tweaks.meta.conflicts')}
                      icon={AlertTriangle}
                      tone="danger"
                    >
                      {t('tweaks.meta.conflicts')}
                    </MetadataChipButton>
                  </TooltipTrigger>
                  <TooltipContent
                    className="max-w-80 text-pretty whitespace-pre-line"
                    sideOffset={8}
                    style={metadataTooltipBorderStyle('danger')}
                  >
                    {conflicts.length === 1
                      ? (
                          <p>{t(conflicts[0].description)}</p>
                        )
                      : (
                          <ul className="list-disc space-y-1 pl-4">
                            {conflicts.map(conflict => (
                              <li key={conflict.description}>
                                {t(conflict.description)}
                              </li>
                            ))}
                          </ul>
                        )}
                  </TooltipContent>
                </Tooltip>
              )}

              {isBelowBuildRequirement && minBuild && (
                <Tooltip>
                  <TooltipTrigger asChild>
                    <MetadataChipButton
                      ariaLabel={t('tweaks.requires.windowsBuild', {
                        build: minBuild,
                      })}
                      icon={CircleAlert}
                      tone="system"
                    >
                      {t('tweaks.requires.windowsBuild', { build: minBuild })}
                    </MetadataChipButton>
                  </TooltipTrigger>
                  <TooltipContent
                    sideOffset={8}
                    style={metadataTooltipBorderStyle('system')}
                  >
                    {t('tweaks.requires.windowsBuild', { build: minBuild })}
                  </TooltipContent>
                </Tooltip>
              )}

              {(isBelowMemoryRequirement || isMemoryRequirementPending) && minInstalledMemoryGb && (
                <Tooltip>
                  <TooltipTrigger asChild>
                    <MetadataChipButton
                      ariaLabel={t('tweaks.requires.memoryGb', {
                        gb: minInstalledMemoryGb,
                      })}
                      icon={CircleAlert}
                      tone="system"
                    >
                      {t('tweaks.requires.memoryGb', { gb: minInstalledMemoryGb })}
                    </MetadataChipButton>
                  </TooltipTrigger>
                  <TooltipContent
                    sideOffset={8}
                    style={metadataTooltipBorderStyle('system')}
                  >
                    {t('tweaks.requires.memoryGb', { gb: minInstalledMemoryGb })}
                  </TooltipContent>
                </Tooltip>
              )}
            </div>
          </footer>
        </div>
      </div>
    </article>
  )
}

export function TweakCardSkeleton() {
  return (
    <article className="h-full rounded-lg border border-border/70 bg-card/95 p-4 shadow-[0_14px_36px_rgb(16_15_15_/_0.08)]">
      <div className="flex h-full min-w-0 flex-col">
        <div className="grid gap-3 sm:grid-cols-[minmax(0,1fr)_auto] sm:gap-4">
          <div className="flex min-h-9 min-w-0 items-center gap-3">
            <Skeleton className="size-9 shrink-0 rounded-md" />
            <Skeleton className="h-4 w-40" />
          </div>
          <div className="min-w-0">
            <div className="flex h-9 items-center justify-end gap-2">
              <Skeleton className="size-9 rounded-md" />
              <Skeleton className="h-9 w-28 rounded-md" />
            </div>
          </div>
        </div>
        <div className="mt-4 space-y-1.5">
          <Skeleton className="h-3 w-full max-w-xl" />
          <Skeleton className="h-3 w-full max-w-md" />
        </div>
        <div className="mt-auto pt-4">
          <div>
            <div className="flex flex-wrap gap-2">
              <Skeleton className="h-5 w-20 rounded-md" />
              <Skeleton className="h-5 w-30 rounded-md" />
              <Skeleton className="h-5 w-16 rounded-md" />
            </div>
          </div>
        </div>
      </div>
    </article>
  )
}
