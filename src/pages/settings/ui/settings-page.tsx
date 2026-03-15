import type { LucideIcon } from 'lucide-react'
import { Download, Languages, Palette } from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { UpdateCheckSwitch } from '@/features/app-updater/ui/update-check-switch'
import { LanguageSelect } from '@/features/language-switcher/ui/language-select'
import { ChromeAcrylicSwitch } from '@/features/theme-switcher/ui/sidebar-acrylic-switch'
import { ThemeSelect } from '@/features/theme-switcher/ui/theme-select'

function SettingsSection({
  children,
  control,
  description,
  icon: Icon,
  title,
}: {
  children?: React.ReactNode
  control?: React.ReactNode
  description: string
  icon: LucideIcon
  title: string
}) {
  return (
    <section className="space-y-3 rounded-xl border border-border/70 bg-card p-4">
      <div className="flex flex-col gap-3 sm:flex-row sm:items-start sm:justify-between">
        <div className="flex items-start gap-3">
          <span className="flex size-9 shrink-0 items-center justify-center rounded-lg bg-accent/60 text-accent-foreground">
            <Icon className="size-4" />
          </span>
          <div className="space-y-1">
            <h2 className="text-sm font-medium text-foreground">{title}</h2>
            <p className="text-xs leading-5 text-muted-foreground">{description}</p>
          </div>
        </div>
        {control && (
          <div className="w-full sm:w-[220px] sm:shrink-0">
            {control}
          </div>
        )}
      </div>
      {children}
    </section>
  )
}

export function SettingsPage() {
  const { t } = useTranslation()

  return (
    <section className="flex flex-1 flex-col gap-4 px-4 pb-4 md:px-6 md:pb-6">
      <div className="w-full space-y-4">
        <SettingsSection
          title={t('settings.language')}
          description={t('settings.languageDescription')}
          control={<LanguageSelect className="sm:ml-auto sm:w-[148px]" />}
          icon={Languages}
        />
        <SettingsSection
          title={t('settings.theme')}
          description={t('settings.themeDescription')}
          icon={Palette}
        >
          <div className="grid gap-3 lg:grid-cols-[minmax(0,1.4fr)_minmax(280px,0.8fr)] lg:items-start">
            <ThemeSelect />
            <ChromeAcrylicSwitch />
          </div>
        </SettingsSection>
        <SettingsSection
          title={t('settings.updates')}
          description={t('settings.updatesDescription')}
          icon={Download}
        >
          <UpdateCheckSwitch />
        </SettingsSection>
      </div>
    </section>
  )
}
