import { Download } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { usePreferencesStore } from '@/entities/settings/model/preferences-store'
import { Switch } from '@/shared/ui/switch'

export function UpdateCheckSwitch() {
  const { t } = useTranslation()
  const updateChecksEnabled = usePreferencesStore(state => state.updateChecksEnabled)
  const setUpdateChecksEnabled = usePreferencesStore(state => state.setUpdateChecksEnabled)

  return (
    <label className="flex min-h-9 items-start justify-between gap-2 rounded-md border border-border/70 bg-background px-3 py-3">
      <div className="flex min-w-0 gap-2">
        <span className="mt-px flex size-4 shrink-0 items-center justify-center text-muted-foreground">
          <Download className="size-4" />
        </span>
        <div className="min-w-0">
          <div className="flex min-h-4 items-center">
            <span className="block text-sm font-medium leading-4 text-foreground">
              {t('settings.updateChecks')}
            </span>
          </div>
          <span className="mt-1 block text-xs leading-5 text-muted-foreground">
            {t('settings.updateChecksDescription')}
          </span>
        </div>
      </div>
      <Switch
        checked={updateChecksEnabled}
        className="mt-0.5"
        onCheckedChange={setUpdateChecksEnabled}
      />
    </label>
  )
}
