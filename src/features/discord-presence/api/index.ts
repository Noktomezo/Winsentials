import type { DiscordPresenceMode } from '@/shared/config/app'
import { invoke } from '@tauri-apps/api/core'

export function setDiscordPresenceActivity({
  mode,
  pageLabel,
}: {
  mode: DiscordPresenceMode
  pageLabel?: string
}): Promise<void> {
  return invoke('set_discord_presence_mode', { mode, pageLabel })
}
