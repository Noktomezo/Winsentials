import { Switch as SwitchPrimitive } from 'radix-ui'
import * as React from 'react'
import { useTranslation } from 'react-i18next'

import { cn } from '@/shared/lib/utils'

type SwitchProps = React.ComponentProps<typeof SwitchPrimitive.Root> & {
  size?: 'sm' | 'default'
}

function Switch({
  className,
  size = 'default',
  ...props
}: SwitchProps) {
  return (
    <SwitchPrimitive.Root
      data-slot="switch"
      data-size={size}
      className={cn(
        'peer group/switch inline-flex shrink-0 cursor-pointer items-center overflow-hidden rounded-[6px] border border-border/70 p-[1px] shadow-xs transition-[background-color,border-color,box-shadow] outline-none focus-visible:border-ring focus-visible:ring-[3px] focus-visible:ring-ring/50 disabled:cursor-not-allowed disabled:opacity-50 data-[size=default]:h-[1.15rem] data-[size=default]:w-8 data-[size=sm]:h-3.5 data-[size=sm]:w-6 data-[state=checked]:border-[color:color-mix(in_oklch,var(--state-good-strong)_72%,var(--border)_28%)] data-[state=checked]:bg-[var(--state-good-strong)] data-[state=unchecked]:border-[color:color-mix(in_oklch,var(--state-danger-strong)_72%,var(--border)_28%)] data-[state=unchecked]:bg-[var(--state-danger-strong)]',
        className,
      )}
      {...props}
    >
      <SwitchPrimitive.Thumb
        data-slot="switch-thumb"
        className={cn(
          'pointer-events-none block rounded-[4px] ring-0 transition-[transform,background-color] group-data-[size=default]/switch:size-[14px] group-data-[size=sm]/switch:size-[10px] data-[state=checked]:translate-x-full data-[state=unchecked]:translate-x-0 data-[state=checked]:bg-[color:color-mix(in_oklch,var(--primary-foreground)_90%,white_10%)] data-[state=unchecked]:bg-[color:color-mix(in_oklch,var(--primary-foreground)_90%,white_10%)]',
        )}
      />
    </SwitchPrimitive.Root>
  )
}

type LabeledSwitchProps = SwitchProps & {
  falseLabel?: string
  trueLabel?: string
  containerClassName?: string
  labelClassName?: string
}

function LabeledSwitch({
  checked = false,
  className,
  containerClassName,
  falseLabel,
  labelClassName,
  size = 'default',
  trueLabel,
  ...props
}: LabeledSwitchProps) {
  const { t } = useTranslation()
  const stateLabel = checked
    ? (trueLabel ?? t('common.on'))
    : (falseLabel ?? t('common.off'))

  return (
    <div
      data-slot="switch-control"
      className={cn(
        'ui-soft-surface inline-flex h-9 w-fit items-center gap-1.5 rounded-md px-2 transition-[background-color,border-color,box-shadow]',
        props.disabled && 'opacity-50',
        containerClassName,
      )}
    >
      <span
        className={cn(
          'text-[10px] font-semibold tracking-[0.01em] text-accent-foreground',
          labelClassName,
        )}
      >
        {stateLabel}
      </span>
      <Switch
        checked={checked}
        className={className}
        size={size}
        {...props}
      />
    </div>
  )
}

export { LabeledSwitch, Switch }
