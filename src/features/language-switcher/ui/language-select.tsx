import type { ReactNode } from 'react'
import { RU, US } from 'country-flag-icons/react/3x2'
import { Globe2 } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { usePreferencesStore } from '@/entities/settings/model/preferences-store'
import { LANGUAGE_PREFERENCES } from '@/shared/config/app'
import { cn } from '@/shared/lib/utils'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
} from '@/shared/ui/select'

type LanguagePreference = (typeof LANGUAGE_PREFERENCES)[number]

const LANGUAGE_NATIVE_NAMES = {
  en: 'English',
  ru: 'Русский',
} as const

function FlagFrame({
  children,
  className,
}: {
  children: ReactNode
  className?: string
}) {
  return (
    <span
      className={cn(
        'inline-flex h-4 w-6 shrink-0 items-center justify-center overflow-hidden rounded-[4px] border border-border/60 bg-secondary/80 shadow-[0_1px_0_rgb(255_255_255_/_0.04)_inset]',
        className,
      )}
    >
      {children}
    </span>
  )
}

function LanguageBadge({ language }: { language: LanguagePreference }) {
  switch (language) {
    case 'system':
      return (
        <FlagFrame className="text-muted-foreground">
          <Globe2 className="size-3.25" />
        </FlagFrame>
      )

    case 'en':
      return (
        <FlagFrame>
          <US className="h-full w-full" title="English" />
        </FlagFrame>
      )

    case 'ru':
      return (
        <FlagFrame>
          <RU className="h-full w-full" title="Русский" />
        </FlagFrame>
      )
  }
}

function languageLabel(language: LanguagePreference, systemLabel: string) {
  return language === 'system' ? systemLabel : LANGUAGE_NATIVE_NAMES[language]
}

export function LanguageSelect({ className }: { className?: string }) {
  const { t } = useTranslation()
  const language = usePreferencesStore(state => state.language)
  const setLanguage = usePreferencesStore(state => state.setLanguage)
  const systemLabel = t('settings.languages.system')

  return (
    <div className={cn('w-full', className)}>
      <Select value={language} onValueChange={value => setLanguage(value as typeof language)}>
        <SelectTrigger className="w-full justify-between !border-border/60 !bg-accent/55 !text-accent-foreground shadow-xs [&_svg:not([class*='text-'])]:!text-accent-foreground/70">
          <span className="flex min-w-0 items-center gap-2">
            <LanguageBadge language={language} />
            <span className="truncate">{languageLabel(language, systemLabel)}</span>
          </span>
        </SelectTrigger>
        <SelectContent>
          {LANGUAGE_PREFERENCES.map(item => (
            <SelectItem key={item} value={item}>
              <span className="flex min-w-0 items-center gap-2">
                <LanguageBadge language={item} />
                <span className="truncate">{languageLabel(item, systemLabel)}</span>
              </span>
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  )
}
