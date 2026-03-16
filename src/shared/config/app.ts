export const APP_NAME = 'Winsentials'

export const APP_THEMES = ['light', 'dark', 'system'] as const
export type AppTheme = (typeof APP_THEMES)[number]
export type ResolvedTheme = Exclude<AppTheme, 'system'>

export const APP_PALETTES = ['teal', 'flexoki'] as const
export type AppPalette = (typeof APP_PALETTES)[number]
export const DEFAULT_PALETTE: AppPalette = 'teal'

export const SUPPORTED_LANGUAGES = ['en', 'ru'] as const
export type AppLanguage = (typeof SUPPORTED_LANGUAGES)[number]

export const DEFAULT_LANGUAGE: AppLanguage = 'en'
export const DEFAULT_THEME: AppTheme = 'dark'
