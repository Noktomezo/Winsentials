import type { AppLanguage, AppLanguagePreference } from '@/shared/config/app'
import { DEFAULT_LANGUAGE, SUPPORTED_LANGUAGES } from '@/shared/config/app'

// Maps navigator.language codes to internal language codes where they differ
const NAVIGATOR_LANG_MAP: Partial<Record<string, AppLanguage>> = {
  uk: 'ua', // Ukrainian: navigator returns 'uk', internal code is 'ua'
}

export function resolveLanguage(pref: AppLanguagePreference): AppLanguage {
  if (pref !== 'system') { return pref }

  const navLang = navigator.language.split('-')[0]
  const mapped = NAVIGATOR_LANG_MAP[navLang] ?? navLang

  return SUPPORTED_LANGUAGES.includes(mapped as AppLanguage)
    ? (mapped as AppLanguage)
    : DEFAULT_LANGUAGE
}
