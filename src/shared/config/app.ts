export const APP_NAME = 'Winsentials'

export const APP_THEMES = ['light', 'dark'] as const
export type AppTheme = (typeof APP_THEMES)[number]
export type ResolvedTheme = AppTheme

export const APP_WEBVIEW_MATERIALS = ['none', 'acrylic', 'mica', 'tabbed'] as const
export type AppWebviewMaterial = (typeof APP_WEBVIEW_MATERIALS)[number]
export const DEFAULT_WEBVIEW_MATERIAL: AppWebviewMaterial = 'none'

export const SUPPORTED_LANGUAGES = ['en', 'ru'] as const
export type AppLanguage = (typeof SUPPORTED_LANGUAGES)[number]

export const LANGUAGE_PREFERENCES = ['system', ...SUPPORTED_LANGUAGES] as const
export type AppLanguagePreference = (typeof LANGUAGE_PREFERENCES)[number]

export const DEFAULT_LANGUAGE: AppLanguage = 'en'
export const DEFAULT_THEME: AppTheme = 'dark'
