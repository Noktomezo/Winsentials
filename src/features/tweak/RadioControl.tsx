import type { TweakInfo } from '@/shared/types/tweak'
import { useTranslation } from 'react-i18next'
import { Badge } from '@/components/ui/badge'
import { Select, SelectContent, SelectItem, SelectItemText, SelectTrigger, SelectValue } from '@/components/ui/select'

interface RadioControlProps {
  tweak: TweakInfo
  onApply: (value: string) => void
  isLoading: boolean
}

export function RadioControl({ tweak, onApply, isLoading }: RadioControlProps) {
  const { t } = useTranslation()

  return (
    <Select
      value={tweak.state.current_value ?? undefined}
      onValueChange={onApply}
      disabled={isLoading}
    >
      <SelectTrigger className="w-[200px]">
        <SelectValue placeholder="Select option" />
      </SelectTrigger>
      <SelectContent>
        {tweak.meta.options.map(option => (
          <SelectItem key={option.value} value={option.value}>
            <SelectItemText>{t(option.label_key)}</SelectItemText>
            <div className="ml-auto flex shrink-0 gap-1">
              {option.is_recommended && (
                <Badge variant="default" className="px-1.5 py-0 text-[10px]">
                  {t('common.recommended')}
                </Badge>
              )}
              {option.is_default && (
                <Badge variant="secondary" className="px-1.5 py-0 text-[10px]">
                  {t('common.default')}
                </Badge>
              )}
            </div>
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  )
}
