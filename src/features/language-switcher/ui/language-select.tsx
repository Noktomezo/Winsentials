import { useTranslation } from 'react-i18next'
import { usePreferencesStore } from '@/entities/settings/model/preferences-store'
import { LANGUAGE_PREFERENCES } from '@/shared/config/app'
import { cn } from '@/shared/lib/utils'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/shared/ui/select'

const LANGUAGE_FLAGS = {
  system: '🌐',
  en: '🇺🇸',
  ru: '🇷🇺',
  ua: '🇺🇦',
  zh: '🇨🇳',
} as const

const LANGUAGE_NATIVE_NAMES = {
  en: 'English',
  ru: 'Русский',
  ua: 'Українська',
  zh: '中文',
} as const

export function LanguageSelect({ className }: { className?: string }) {
  const { t } = useTranslation()
  const language = usePreferencesStore(state => state.language)
  const setLanguage = usePreferencesStore(state => state.setLanguage)

  return (
    <div className={cn('w-full', className)}>
      <Select value={language} onValueChange={value => setLanguage(value as typeof language)}>
        <SelectTrigger className="w-full justify-between">
          <SelectValue placeholder={t('settings.language')} />
        </SelectTrigger>
        <SelectContent>
          {LANGUAGE_PREFERENCES.map(item => (
            <SelectItem key={item} value={item}>
              <span className="flex items-center gap-2">
                <span className="emoji-flag text-base leading-none">
                  {LANGUAGE_FLAGS[item]}
                </span>
                <span>{item === 'system' ? t('settings.languages.system') : LANGUAGE_NATIVE_NAMES[item]}</span>
              </span>
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  )
}
