import type { LucideIcon } from 'lucide-react'
import { MoonStar, SunMedium } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { usePreferencesStore } from '@/entities/settings/model/preferences-store'
import { APP_THEMES } from '@/shared/config/app'
import { cn } from '@/shared/lib/utils'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/shared/ui/'

const THEME_ICONS: Record<(typeof APP_THEMES)[number], LucideIcon> = {
  light: SunMedium,
  dark: MoonStar,
}

export function ThemeSelect({ className }: { className?: string }) {
  const { t } = useTranslation()
  const theme = usePreferencesStore(state => state.theme)
  const setTheme = usePreferencesStore(state => state.setTheme)

  return (
    <div className={cn('w-full', className)}>
      <Select value={theme} onValueChange={value => setTheme(value as typeof theme)}>
        <SelectTrigger className="w-full justify-between">
          <SelectValue placeholder={t('settings.mode')} />
        </SelectTrigger>
        <SelectContent>
          {APP_THEMES.map((item) => {
            const Icon = THEME_ICONS[item]

            return (
              <SelectItem key={item} value={item}>
                <span className="flex items-center gap-2">
                  <Icon className="size-3.5 shrink-0 text-muted-foreground" />
                  <span>{t(`settings.themes.${item}`)}</span>
                </span>
              </SelectItem>
            )
          })}
        </SelectContent>
      </Select>
    </div>
  )
}
