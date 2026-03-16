import { Info, MirrorRectangular } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { usePreferencesStore } from '@/entities/settings/model/preferences-store'
import { Switch } from '@/shared/ui/switch'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/shared/ui/tooltip'

export function ChromeAcrylicSwitch() {
  const { t } = useTranslation()
  const chromeAcrylic = usePreferencesStore(state => state.chromeAcrylic)
  const setChromeAcrylic = usePreferencesStore(state => state.setChromeAcrylic)

  return (
    <label className="flex min-h-9 items-center justify-between gap-4">
      <div className="flex shrink-0 items-center gap-2 text-sm font-medium text-foreground">
        <MirrorRectangular className="size-4 text-muted-foreground" />
        {t('settings.acrylic')}
        <Tooltip>
          <TooltipTrigger asChild>
            <button aria-label={t('settings.acrylicInfo')} className="inline-flex items-center justify-center text-muted-foreground transition-colors hover:text-accent-foreground" type="button">
              <Info className="size-3.5" />
            </button>
          </TooltipTrigger>
          <TooltipContent className="max-w-64 text-pretty" sideOffset={8}>
            {t('settings.acrylicDescription')}
          </TooltipContent>
        </Tooltip>
      </div>
      <Switch
        checked={chromeAcrylic}
        onCheckedChange={setChromeAcrylic}
      />
    </label>
  )
}
