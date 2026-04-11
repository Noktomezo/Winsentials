import type { LucideIcon } from 'lucide-react'
import { Info, Layers, Monitor, Palette, PanelsTopLeft } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { usePreferencesStore } from '@/entities/settings/model/preferences-store'
import { APP_WEBVIEW_MATERIALS } from '@/shared/config/app'
import { cn } from '@/shared/lib/utils'
import {
  filledSelectTriggerClassName,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from '@/shared/ui'

const MATERIAL_ICONS: Record<(typeof APP_WEBVIEW_MATERIALS)[number], LucideIcon> = {
  none: Monitor,
  acrylic: Palette,
  mica: Layers,
  tabbed: PanelsTopLeft,
}

export function WebviewMaterialSelect({ className }: { className?: string }) {
  const { t } = useTranslation()
  const webviewMaterial = usePreferencesStore(state => state.webviewMaterial)
  const setWebviewMaterial = usePreferencesStore(state => state.setWebviewMaterial)

  return (
    <div className="space-y-2">
      <div className="flex flex-col gap-2 sm:flex-row sm:items-center sm:justify-between">
        <div className="flex shrink-0 items-center gap-2 self-center text-sm font-medium text-foreground">
          <Layers className="size-4 text-muted-foreground" />
          {t('settings.windowMaterial')}
          <Tooltip>
            <TooltipTrigger asChild>
              <button aria-label={t('settings.materialInfo')} className="inline-flex items-center justify-center text-muted-foreground transition-colors hover:text-accent-foreground" type="button">
                <Info className="size-3.5" />
              </button>
            </TooltipTrigger>
            <TooltipContent className="max-w-72 text-pretty" sideOffset={8}>
              {t('settings.materialDescription')}
            </TooltipContent>
          </Tooltip>
        </div>
        <div className={cn('w-full sm:w-[163px]', className)}>
          <Select value={webviewMaterial} onValueChange={value => setWebviewMaterial(value as typeof webviewMaterial)}>
            <SelectTrigger className={filledSelectTriggerClassName}>
              <SelectValue placeholder={t('settings.windowMaterial')} />
            </SelectTrigger>
            <SelectContent>
              {APP_WEBVIEW_MATERIALS.map((item) => {
                const Icon = MATERIAL_ICONS[item]

                return (
                  <SelectItem key={item} value={item}>
                    <span className="flex items-center gap-2">
                      <Icon className="size-3.5 shrink-0 text-muted-foreground" />
                      <span>{t(`settings.materials.${item}`)}</span>
                    </span>
                  </SelectItem>
                )
              })}
            </SelectContent>
          </Select>
        </div>
      </div>
    </div>
  )
}
