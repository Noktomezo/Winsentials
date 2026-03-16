import { useTranslation } from 'react-i18next'
import { usePreferencesStore } from '@/entities/settings/model/preferences-store'
import { SUPPORTED_LANGUAGES } from '@/shared/config/app'
import { cn } from '@/shared/lib/utils'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/shared/ui/select'

const LANGUAGE_FLAGS = {
  en: '🇺🇸',
  ru: '🇷🇺',
} as const

export function LanguageSelect({ className }: { className?: string }) {
  const { t } = useTranslation()
  const language = usePreferencesStore(state => state.language)
  const setLanguage = usePreferencesStore(state => state.setLanguage)

  return (
    <div className={cn('w-full', className)}>
      <Select value={language} onValueChange={value => setLanguage(value as typeof language)}>
        <SelectTrigger className="w-full justify-between aria-expanded:border-primary/40 aria-expanded:ring-1 aria-expanded:ring-primary/50 focus-visible:border-primary/40 focus-visible:ring-primary/50">
          <SelectValue placeholder={t('settings.language')} />
        </SelectTrigger>
        <SelectContent>
          {SUPPORTED_LANGUAGES.map(item => (
            <SelectItem key={item} value={item}>
              <span className="flex items-center gap-2">
                <span className="emoji-flag text-base leading-none">
                  {LANGUAGE_FLAGS[item]}
                </span>
                <span>{t(`settings.languages.${item}`)}</span>
              </span>
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  )
}
