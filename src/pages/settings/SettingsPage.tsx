import { useTranslation } from 'react-i18next'
import {
  Select,
  SelectContent,
  SelectItem,
  SelectItemText,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select'
import { useSettingsStore } from '@/shared/store/settings'

export function SettingsPage() {
  const { t } = useTranslation()
  const { theme, language, setTheme, setLanguage } = useSettingsStore()

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-2xl font-bold">{t('sidebar.settings')}</h1>
        <p className="text-muted-foreground">{t('settings.description')}</p>
      </div>

      <div className="space-y-4">
        <div className="flex items-center justify-between rounded-lg border border-border bg-card p-4">
          <div>
            <h3 className="font-medium">{t('settings.language')}</h3>
            <p className="text-sm text-muted-foreground">{t('settings.languageDescription')}</p>
          </div>
          <Select value={language} onValueChange={v => setLanguage(v as 'en' | 'ru')}>
            <SelectTrigger className="w-32">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="en">
                <SelectItemText>English</SelectItemText>
              </SelectItem>
              <SelectItem value="ru">
                <SelectItemText>Русский</SelectItemText>
              </SelectItem>
            </SelectContent>
          </Select>
        </div>

        <div className="flex items-center justify-between rounded-lg border border-border bg-card p-4">
          <div>
            <h3 className="font-medium">{t('settings.theme')}</h3>
            <p className="text-sm text-muted-foreground">{t('settings.themeDescription')}</p>
          </div>
          <Select value={theme} onValueChange={v => setTheme(v as 'light' | 'dark' | 'system')}>
            <SelectTrigger className="w-32">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="system">
                <SelectItemText>{t('settings.themeSystem')}</SelectItemText>
              </SelectItem>
              <SelectItem value="light">
                <SelectItemText>{t('settings.themeLight')}</SelectItemText>
              </SelectItem>
              <SelectItem value="dark">
                <SelectItemText>{t('settings.themeDark')}</SelectItemText>
              </SelectItem>
            </SelectContent>
          </Select>
        </div>
      </div>
    </div>
  )
}
