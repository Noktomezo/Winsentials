import type { TFunction } from 'i18next'
import type { LucideIcon } from 'lucide-react'
import type { TweakMeta } from '@/entities/tweak/model/types'
import { BellOff, CloudOff, Gauge, Keyboard, KeyboardOff, Settings } from 'lucide-react'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/shared/ui/select'

const DROPDOWN_OPTION_ICONS: Record<string, Record<string, LucideIcon>> = {
  fast_keyboard_repeat: {
    default: Settings,
    balanced: Gauge,
    fast: Keyboard,
    ultra_fast: Keyboard,
  },
  disable_cloud_sync: {
    default: Settings,
    partial: BellOff,
    full: CloudOff,
  },
  disable_ctf_ctfmon: {
    default: Settings,
    soft: Keyboard,
    aggressive: KeyboardOff,
  },
}

function dropdownOptionIcon(
  tweakId: string,
  optionValue: string,
): LucideIcon | null {
  return DROPDOWN_OPTION_ICONS[tweakId]?.[optionValue] ?? null
}

interface TweakCardDropdownProps {
  tweak: TweakMeta
  t: TFunction
  isPending: boolean
  isApplyBlocked: boolean
  isUnsupported: boolean
  onApplyValue: (value: string) => void
}

export function TweakCardDropdown({
  tweak,
  t,
  isPending,
  isApplyBlocked,
  isUnsupported,
  onApplyValue,
}: TweakCardDropdownProps) {
  if (tweak.control.kind !== 'dropdown') {
    return null
  }

  const dropdownOptions
    = tweak.currentValue === 'custom'
      ? [
          ...tweak.control.options,
          { label: 'tweaks.meta.customValue', value: 'custom' },
        ]
      : tweak.control.options

  const selectedDropdownOption
    = dropdownOptions.find(option => option.value === tweak.currentValue) ?? null

  const SelectedDropdownIcon
    = selectedDropdownOption
      ? dropdownOptionIcon(tweak.id, selectedDropdownOption.value)
      : null

  return (
    <Select
      disabled={isPending || isApplyBlocked}
      onValueChange={(value) => {
        if (isUnsupported && value !== tweak.defaultValue) {
          return
        }

        onApplyValue(value)
      }}
      value={tweak.currentValue}
    >
      <SelectTrigger className="ui-soft-surface bg-secondary! h-9 min-w-[10.5rem] justify-between rounded-md px-3 text-xs font-medium transition-colors hover:bg-accent/50! [&_svg]:size-3.5 [&_svg:not([class*='text-'])]:text-accent-foreground/70!">
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
        className="ui-soft-surface min-w-[var(--radix-select-trigger-width)] rounded-[10px] text-xs font-medium"
      >
        {dropdownOptions.map((option) => {
          const OptionIcon = dropdownOptionIcon(tweak.id, option.value)

          return (
            <SelectItem
              className="min-h-7 px-2 py-1 text-xs font-medium"
              disabled={option.value === 'custom' || (isUnsupported && option.value !== tweak.defaultValue)}
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
  )
}
