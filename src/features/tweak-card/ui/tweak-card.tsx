import type { LucideIcon } from 'lucide-react'
import type { CSSProperties } from 'react'
import type { TweakMeta, WindowsVersion } from '@/entities/tweak/model/types'
import {
  ArrowLeftRight,
  BellOff,
  BotOff,
  Check,
  CircleAlert,
  Clock3,
  CloudOff,
  Cpu,
  ExternalLink,
  EyeOff,
  FileSearch,
  FileType,
  Gamepad2,
  Gauge,
  Globe,
  HardDrive,
  History,
  House,
  Images,
  Info,
  Keyboard,
  KeyboardOff,
  Link,
  ListX,
  MapPinned,
  MemoryStick,
  Menu,
  Mouse,
  MousePointer2,
  Network,
  PackageX,
  PanelsTopLeft,
  PlugZap,
  Power,
  RotateCcw,
  Shield,
  ShieldOff,
  Terminal,
  TextCursor,
  TriangleAlert,
  Type,
  Usb,
  Zap,
} from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { Button } from '@/shared/ui/button'
import { Skeleton } from '@/shared/ui/skeleton'
import { LabeledSwitch } from '@/shared/ui/switch'
import { TweakCardDropdown } from './tweak-card-dropdown'
import { TweakCardFooter } from './tweak-card-footer'

interface TweakCardProps {
  currentBuild: WindowsVersion
  currentInstalledMemoryBytes?: number | null
  isPending?: boolean
  onApplyValue: (value: string) => void
  tweak: TweakMeta
}

const BYTES_PER_GIB = 1024 ** 3
const HEADER_ICON_WIDTH = 36
const HEADER_ICON_GAP = 12
const HEADER_CONTROLS_GAP = 16
const CARD_HORIZONTAL_PADDING = 32
const RESET_BUTTON_WIDTH = 36
const CONTROL_GAP = 8
const TOGGLE_CONTROL_WIDTH = 94
const ACTION_CONTROL_WIDTH = 104
const DROPDOWN_CONTROL_WIDTH = 168
const TOGGLE_CUSTOM_CONTROL_WIDTH = 112
const MIN_CARD_WIDTH = 360
const MAX_CARD_WIDTH = 760

const TWEAK_ICONS: Record<string, LucideIcon> = {
  classic_context_menu: Menu,
  fast_taskbar_thumbnails: PanelsTopLeft,
  faster_cursor_blink_rate: TextCursor,
  hide_gallery_navigation_pane: Images,
  hide_home_navigation_pane: House,
  hide_network_navigation_pane: Network,
  disable_8dot3_name_creation: FileType,
  disable_wallpaper_jpeg_compression: Images,
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
  fast_udp_optimization: ArrowLeftRight,
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
  disable_ctf_ctfmon: KeyboardOff,
  disable_mouse_acceleration: Mouse,
  raw_mouse_throttle: MousePointer2,
  optimize_mmcss: Zap,
  fast_keyboard_repeat: Keyboard,
  enable_bbr2_congestion_control: Zap,
  disable_qos_bandwidth_limit: Gauge,
  enable_network_offloading_rss: Cpu,
  microsoft_edge_debloat: Globe,
  brave_browser_debloat: PackageX,
  disable_microsoft_copilot: BotOff,
  remove_microsoft_edge: Globe,
  remove_microsoft_onedrive: CloudOff,
  block_razer_auto_install: PlugZap,
  create_symbolic_link_context_menu: Link,
}

let tweakTitleMeasureCanvas: HTMLCanvasElement | null = null

function measureTweakTitleWidth(title: string) {
  if (typeof document === 'undefined') {
    return title.length * 8
  }

  tweakTitleMeasureCanvas ??= document.createElement('canvas')
  const context = tweakTitleMeasureCanvas.getContext('2d')

  if (!context) {
    return title.length * 8
  }

  context.font = '500 14px "IBM Plex Sans", "Segoe UI Variable Text", "Segoe UI", sans-serif'
  return Math.ceil(context.measureText(title).width)
}

function tweakControlWidth(tweak: TweakMeta) {
  if (tweak.control.kind === 'action') {
    return ACTION_CONTROL_WIDTH
  }

  if (tweak.control.kind === 'dropdown') {
    return DROPDOWN_CONTROL_WIDTH
  }

  if (tweak.control.kind === 'toggle' && tweak.currentValue === 'custom') {
    return TOGGLE_CUSTOM_CONTROL_WIDTH
  }

  return TOGGLE_CONTROL_WIDTH
}

function tweakCardWidth(title: string, tweak: TweakMeta) {
  const headerWidth
    = HEADER_ICON_WIDTH
      + HEADER_ICON_GAP
      + measureTweakTitleWidth(title)
      + HEADER_CONTROLS_GAP
      + RESET_BUTTON_WIDTH
      + CONTROL_GAP
      + tweakControlWidth(tweak)
      + CARD_HORIZONTAL_PADDING

  return Math.max(MIN_CARD_WIDTH, Math.min(MAX_CARD_WIDTH, headerWidth))
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
  currentInstalledMemoryBytes = null,
  isPending = false,
  onApplyValue,
  tweak,
}: TweakCardProps) {
  const { t } = useTranslation()
  const tweakName = t(tweak.name)
  const Icon = TWEAK_ICONS[tweak.id] ?? Info
  const isEnabled = tweak.currentValue === 'enabled'
  const isRecommended = tweak.currentValue === tweak.recommendedValue
  const isAtDefault = tweak.currentValue === tweak.defaultValue
  const isBelowBuildRequirement = isBelowMinBuild(currentBuild, tweak)
  const minInstalledMemoryGb = tweak.minRequiredMemoryGb ?? null
  const isBelowMemoryRequirement = minInstalledMemoryGb !== null
    && currentInstalledMemoryBytes !== null
    && currentInstalledMemoryBytes < minInstalledMemoryGb * BYTES_PER_GIB
  const isMemoryRequirementPending = minInstalledMemoryGb !== null
    && currentInstalledMemoryBytes === null
  const isUnsupported = isBelowBuildRequirement
    || isBelowMemoryRequirement
    || isMemoryRequirementPending
  const isCustomToggleBlocked
    = tweak.control.kind === 'toggle'
      && tweak.currentValue === 'custom'
      && !isEnabled
  const isApplyBlocked = isUnsupported && (isAtDefault || isCustomToggleBlocked)
  const minBuild = formatMinBuild(tweak)
  const cardWidth = tweakCardWidth(tweakName, tweak)
  const cardStyle = {
    '--tweak-card-width': `${cardWidth}px`,
    '--tweak-card-grow': `${cardWidth}`,
  } as CSSProperties

  return (
    <article
      className="rounded-lg border border-border/70 bg-card/95 p-4 shadow-[0_14px_36px_rgb(16_15_15_/_0.08)]"
      style={cardStyle}
    >
      <div className="flex h-full min-w-0 flex-col">
        <div className="flex min-w-0 items-center gap-3">
          <div className="flex min-h-9 min-w-0 flex-1 items-center gap-3">
            <span className="ui-soft-surface flex size-9 shrink-0 items-center justify-center rounded-md">
              <Icon className="size-4" />
            </span>
            <h2 className="min-w-0 flex-1 truncate text-sm font-medium leading-5 text-foreground">
              {tweakName}
            </h2>
          </div>

          <aside className="ml-auto flex shrink-0 items-center gap-2">
            <Button
              aria-label={t('tweaks.actions.resetToDefault')}
              className="ui-soft-surface transition-colors hover:border-destructive/30! hover:bg-destructive/10! hover:text-destructive!"
              disabled={isPending || isAtDefault}
              onClick={() => onApplyValue(tweak.defaultValue)}
              size="icon"
              type="button"
              variant="ghost"
            >
              <RotateCcw className="size-4" />
            </Button>

            {tweak.control.kind === 'action' && (
              <Button
                disabled={isPending || isApplyBlocked || isRecommended}
                onClick={() => onApplyValue(tweak.recommendedValue)}
                type="button"
              >
                <Check className="size-4" />
                {t('tweaks.actions.apply')}
              </Button>
            )}
            {tweak.control.kind === 'toggle' && (
              <LabeledSwitch
                aria-label={t(tweak.name)}
                checked={isEnabled}
                containerClassName="ui-soft-surface transition-colors hover:bg-accent/50!"
                disabled={isPending || isApplyBlocked}
                labelClassName="text-accent-foreground!"
                onCheckedChange={checked =>
                  onApplyValue(checked ? 'enabled' : 'disabled')}
              />
            )}
            {tweak.control.kind === 'dropdown' && (
              <TweakCardDropdown
                isApplyBlocked={isApplyBlocked}
                isPending={isPending}
                isUnsupported={isUnsupported}
                onApplyValue={onApplyValue}
                t={t}
                tweak={tweak}
              />
            )}
          </aside>
        </div>

        <p className="mt-4 text-xs leading-5 text-muted-foreground">
          {t(tweak.shortDescription)}
        </p>

        <TweakCardFooter
          isBelowBuildRequirement={isBelowBuildRequirement}
          isBelowMemoryRequirement={isBelowMemoryRequirement}
          isMemoryRequirementPending={isMemoryRequirementPending}
          minBuild={minBuild}
          minInstalledMemoryGb={minInstalledMemoryGb}
          t={t}
          tweak={tweak}
        />
      </div>
    </article>
  )
}

export function TweakCardSkeleton() {
  return (
    <article className="rounded-lg border border-border/70 bg-card/95 p-4 shadow-[0_14px_36px_rgb(16_15_15_/_0.08)]">
      <div className="flex h-full min-w-0 flex-col">
        <div className="flex min-w-0 items-center gap-3">
          <div className="flex min-h-9 min-w-0 flex-1 items-center gap-3">
            <Skeleton className="size-9 shrink-0 rounded-md" />
            <Skeleton className="h-4 flex-1" />
          </div>
          <div className="ml-auto flex shrink-0 items-center gap-2">
            <Skeleton className="size-9 rounded-md" />
            <Skeleton className="h-9 w-28 rounded-md" />
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
