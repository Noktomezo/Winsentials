use tauri::State;

use crate::discord_presence::{DiscordPresenceMode, DiscordPresenceState};
use crate::error::AppError;

#[tauri::command]
pub fn set_discord_presence_mode(
    mode: DiscordPresenceMode,
    page_label: Option<String>,
    state: State<'_, DiscordPresenceState>,
) -> Result<(), AppError> {
    state.set_mode(mode, page_label.as_deref())
}
