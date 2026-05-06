import type { ComponentType } from 'react'
import type { DiscordPresenceMode } from '@/shared/config/app'
import {
  Eye,
  Gamepad2,
  Headphones,
  Trophy,
  Unlink,
} from 'lucide-react'
import { useTranslation } from 'react-i18next'
import { usePreferencesStore } from '@/entities/settings/model/preferences-store'
import { DISCORD_PRESENCE_MODES } from '@/shared/config/app'
import { cn } from '@/shared/lib/utils'
import {
  filledSelectTriggerClassName,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
} from '@/shared/ui/select'

const PRESENCE_ICONS = {
  none: Unlink,
  playing: Gamepad2,
  listening: Headphones,
  watching: Eye,
  competing: Trophy,
} satisfies Record<DiscordPresenceMode, ComponentType<{ className?: string }>>

function PresenceOption({ mode }: { mode: DiscordPresenceMode }) {
  const { t } = useTranslation()
  const Icon = PRESENCE_ICONS[mode]

  return (
    <span className="flex min-w-0 items-center gap-2">
      <Icon className="size-4 shrink-0 text-muted-foreground" />
      <span className="truncate">{t(`settings.discordPresenceModes.${mode}`)}</span>
    </span>
  )
}

export function DiscordPresenceSelect({ className }: { className?: string }) {
  const mode = usePreferencesStore(state => state.discordPresenceMode)
  const setMode = usePreferencesStore(state => state.setDiscordPresenceMode)

  return (
    <div className={cn('w-full', className)}>
      <Select
        value={mode}
        onValueChange={value => setMode(value as DiscordPresenceMode)}
      >
        <SelectTrigger className={filledSelectTriggerClassName}>
          <PresenceOption mode={mode} />
        </SelectTrigger>
        <SelectContent>
          {DISCORD_PRESENCE_MODES.map(item => (
            <SelectItem key={item} value={item}>
              <PresenceOption mode={item} />
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  )
}
