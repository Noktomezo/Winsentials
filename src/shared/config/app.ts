export const APP_THEMES = ['system', 'light', 'dark'] as const
export type AppTheme = (typeof APP_THEMES)[number]
export type ResolvedTheme = 'light' | 'dark'

export const APP_WEBVIEW_MATERIALS = ['none', 'acrylic', 'mica', 'tabbed'] as const
export type AppWebviewMaterial = (typeof APP_WEBVIEW_MATERIALS)[number]
export const DEFAULT_WEBVIEW_MATERIAL: AppWebviewMaterial = 'none'

export const SUPPORTED_LANGUAGES = ['en', 'ru'] as const
export type AppLanguage = (typeof SUPPORTED_LANGUAGES)[number]

export const LANGUAGE_PREFERENCES = ['system', ...SUPPORTED_LANGUAGES] as const
export type AppLanguagePreference = (typeof LANGUAGE_PREFERENCES)[number]

export const DEFAULT_LANGUAGE: AppLanguage = 'en'
export const DEFAULT_THEME: AppTheme = 'system'

export const DISCORD_PRESENCE_MODES = ['none', 'playing', 'listening', 'watching', 'competing'] as const
export type DiscordPresenceMode = (typeof DISCORD_PRESENCE_MODES)[number]
export const DEFAULT_DISCORD_PRESENCE_MODE: DiscordPresenceMode = 'none'

export const TWEAK_CATEGORIES = ['appearance', 'behaviour', 'security', 'privacy', 'network', 'performance', 'memory', 'input', 'debloat', 'context_menu'] as const
