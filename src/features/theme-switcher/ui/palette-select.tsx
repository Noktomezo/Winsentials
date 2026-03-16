import type { LucideIcon } from 'lucide-react'
import { Leaf, Waves } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { usePreferencesStore } from '@/entities/settings/model/preferences-store'
import { APP_PALETTES } from '@/shared/config/app'
import { cn } from '@/shared/lib/utils'
import { RadioGroup, RadioGroupItem } from '@/shared/ui/radio-group'

const PALETTE_ICONS: Record<(typeof APP_PALETTES)[number], LucideIcon> = {
  flexoki: Leaf,
  teal: Waves,
}

export function PaletteSelect() {
  const { t } = useTranslation()
  const palette = usePreferencesStore(state => state.palette)
  const setPalette = usePreferencesStore(state => state.setPalette)

  return (
    <RadioGroup
      className="flex gap-1 rounded-lg border border-border/70 bg-background p-1"
      onValueChange={value => setPalette(value as typeof palette)}
      value={palette}
    >
      {APP_PALETTES.map((item) => {
        const checked = palette === item
        const Icon = PALETTE_ICONS[item]

        return (
          <label
            key={item}
            className={cn(
              'flex flex-1 cursor-pointer items-center justify-center gap-1.5 rounded-md px-3 py-1.5 text-xs font-medium text-muted-foreground transition-colors',
              checked && 'bg-accent text-accent-foreground shadow-xs',
            )}
          >
            <RadioGroupItem className="sr-only" value={item} />
            <Icon className="size-3.5 shrink-0" />
            {t(`settings.palettes.${item}`)}
          </label>
        )
      })}
    </RadioGroup>
  )
}
