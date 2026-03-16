import { usePreferencesStore } from '@/entities/settings/model/preferences-store'
import { Switch } from '@/shared/ui/switch'

export function UpdateCheckSwitch() {
  const updateChecksEnabled = usePreferencesStore(state => state.updateChecksEnabled)
  const setUpdateChecksEnabled = usePreferencesStore(state => state.setUpdateChecksEnabled)

  return (
    <Switch
      checked={updateChecksEnabled}
      onCheckedChange={setUpdateChecksEnabled}
    />
  )
}
