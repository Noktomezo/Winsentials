import { useTranslation } from 'react-i18next'
import { usePreferencesStore } from '@/entities/settings/model/preferences-store'
import { LabeledSwitch } from '@/shared/ui/switch'

export function UpdateCheckSwitch() {
  const { t } = useTranslation()
  const updateChecksEnabled = usePreferencesStore(state => state.updateChecksEnabled)
  const setUpdateChecksEnabled = usePreferencesStore(state => state.setUpdateChecksEnabled)

  return (
    <LabeledSwitch
      aria-label={t('settings.updates')}
      checked={updateChecksEnabled}
      onCheckedChange={setUpdateChecksEnabled}
    />
  )
}
