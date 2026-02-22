import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'
import en from './locales/en.json'
import ru from './locales/ru.json'

function getSystemLanguage(): string {
  if (typeof navigator === 'undefined')
    return 'en'
  const lang = navigator.language.split('-')[0]
  return lang === 'ru' ? 'ru' : 'en'
}

i18n.use(initReactI18next).init({
  resources: {
    en: { translation: en },
    ru: { translation: ru },
  },
  lng: getSystemLanguage(),
  fallbackLng: 'en',
  interpolation: {
    escapeValue: false,
  },
})

export default i18n
