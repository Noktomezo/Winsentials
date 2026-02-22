import type { TweakInfo } from '@/shared/types/tweak'
import { Switch } from '@/components/ui/switch'

interface ToggleControlProps {
  tweak: TweakInfo
  onApply: () => void
  onRevert: () => void
  isLoading: boolean
}

export function ToggleControl({ tweak, onApply, onRevert, isLoading }: ToggleControlProps) {
  function handleChange(checked: boolean) {
    if (checked) {
      onApply()
    }
    else {
      onRevert()
    }
  }

  return (
    <Switch
      checked={tweak.state.is_applied}
      onCheckedChange={handleChange}
      disabled={isLoading}
    />
  )
}
