import type { TFunction } from 'i18next'
import type { LucideIcon } from 'lucide-react'
import type { TweakMeta } from '@/entities/tweak/model/types'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import {
  AlertTriangle,
  CircleAlert,
  Copy,
  Info,
  LogOut,
  Power,
  RotateCcw,
  Settings,
  TriangleAlert,
  Usb,
} from 'lucide-react'
import { Trans } from 'react-i18next'
import { toast } from '@/shared/lib/toast'
import { cn } from '@/shared/lib/utils'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/shared/ui/tooltip'

const COPYABLE_RISK_COMMANDS: Record<string, string> = {
  disable_user_account_control: 'runas /trustlevel:0x20000 "program.exe"',
}

function metadataChipClassName(
  tone: 'default' | 'details' | 'action' | 'warning' | 'danger' | 'system' = 'default',
) {
  if (tone === 'details') {
    return '!border-border/60 !bg-accent/55 text-muted-foreground'
  }

  if (tone === 'action') {
    return '!border-[color:color-mix(in_oklch,var(--badge-blue)_28%,transparent)] !bg-[color:color-mix(in_oklch,var(--badge-blue)_12%,transparent)] text-[var(--badge-blue)]'
  }

  if (tone === 'warning') {
    return '!border-[color:color-mix(in_oklch,var(--badge-yellow)_28%,transparent)] !bg-[color:color-mix(in_oklch,var(--badge-yellow)_12%,transparent)] text-[var(--badge-yellow)]'
  }

  if (tone === 'danger') {
    return '!border-[color:color-mix(in_oklch,var(--badge-red)_28%,transparent)] !bg-[color:color-mix(in_oklch,var(--badge-red)_12%,transparent)] text-[var(--badge-red)]'
  }

  if (tone === 'system') {
    return '!border-[color:color-mix(in_oklch,var(--badge-purple)_28%,transparent)] !bg-[color:color-mix(in_oklch,var(--badge-purple)_12%,transparent)] text-[var(--badge-purple)]'
  }

  return '!border-border/70 !bg-secondary text-muted-foreground'
}

function requiresActionBadge(
  tweak: TweakMeta,
  t: TFunction,
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

function MetadataChipButton({
  ariaLabel,
  children,
  tone = 'default',
  icon,
  className,
  type,
  ref,
  ...props
}: React.PropsWithChildren<React.ComponentProps<'button'> & {
  ariaLabel: string
  tone?: 'default' | 'details' | 'action' | 'warning' | 'danger' | 'system'
  icon?: LucideIcon
  ref?: React.Ref<HTMLButtonElement>
}>) {
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
}

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

interface TweakCardFooterProps {
  tweak: TweakMeta
  t: TFunction
  isBelowBuildRequirement: boolean
  minBuild: string | null
  isBelowMemoryRequirement: boolean
  isMemoryRequirementPending: boolean
  minInstalledMemoryGb: number | null
}

export function TweakCardFooter({
  tweak,
  t,
  isBelowBuildRequirement,
  minBuild,
  isBelowMemoryRequirement,
  isMemoryRequirementPending,
  minInstalledMemoryGb,
}: TweakCardFooterProps) {
  const requiresBadge = requiresActionBadge(tweak, t)
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
              className={cn('max-w-80 text-pretty', metadataChipClassName('details'))}
              sideOffset={8}
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
                className={cn('max-w-80 text-pretty whitespace-pre-line', metadataChipClassName('action'))}
                sideOffset={8}
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
                  className={cn('max-w-80 text-pretty whitespace-pre-line', metadataChipClassName('warning'))}
                  sideOffset={8}
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
                className={cn('max-w-80 text-pretty whitespace-pre-line', metadataChipClassName('danger'))}
                sideOffset={8}
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
                className={cn(metadataChipClassName('system'))}
                sideOffset={8}
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
                className={cn(metadataChipClassName('system'))}
                sideOffset={8}
              >
                {t('tweaks.requires.memoryGb', { gb: minInstalledMemoryGb })}
              </TooltipContent>
            </Tooltip>
          )}
        </div>
      </footer>
    </div>
  )
}
