import type { LucideIcon } from 'lucide-react'
import { Anchor, Leaf, Waves } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { usePreferencesStore } from '@/entities/settings/model/preferences-store'
import { APP_PALETTES } from '@/shared/config/app'
import { cn } from '@/shared/lib/utils'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/shared/ui/select'

const PALETTE_ICONS: Record<(typeof APP_PALETTES)[number], LucideIcon> = {
  abyss: Anchor,
  flexoki: Leaf,
  teal: Waves,
}

export function PaletteSelect({ className }: { className?: string }) {
  const { t } = useTranslation()
  const palette = usePreferencesStore(state => state.palette)
  const setPalette = usePreferencesStore(state => state.setPalette)

  return (
    <div className={cn('w-full', className)}>
      <Select value={palette} onValueChange={value => setPalette(value as typeof palette)}>
        <SelectTrigger className="w-full justify-between aria-expanded:border-primary/40 aria-expanded:ring-1 aria-expanded:ring-primary/50 focus-visible:border-primary/40 focus-visible:ring-primary/50">
          <SelectValue placeholder={t('settings.palette')} />
        </SelectTrigger>
        <SelectContent>
          {APP_PALETTES.map((item) => {
            const Icon = PALETTE_ICONS[item]

            return (
              <SelectItem key={item} value={item}>
                <span className="flex items-center gap-2">
                  <Icon className="size-3.5 shrink-0 text-muted-foreground" />
                  <span>{t(`settings.palettes.${item}`)}</span>
                </span>
              </SelectItem>
            )
          })}
        </SelectContent>
      </Select>
    </div>
  )
}
