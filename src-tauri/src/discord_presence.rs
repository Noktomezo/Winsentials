use std::sync::Mutex;

use discord_rich_presence::{
    DiscordIpc, DiscordIpcClient,
    activity::{self, Activity},
};
use serde::{Deserialize, Serialize};

use crate::error::AppError;

const DISCORD_CLIENT_ID_ENV: &str = "WINSENTIALS_DISCORD_CLIENT_ID";

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DiscordPresenceMode {
    None,
    Playing,
    Listening,
    Watching,
    Competing,
}

#[derive(Default)]
pub struct DiscordPresenceState {
    client: Mutex<Option<DiscordIpcClient>>,
}

impl DiscordPresenceState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_mode(
        &self,
        mode: DiscordPresenceMode,
        page_label: Option<&str>,
    ) -> Result<(), AppError> {
        let mut client = self
            .client
            .lock()
            .map_err(|_| AppError::message("discord presence state lock poisoned"))?;

        if mode == DiscordPresenceMode::None {
            if let Some(active_client) = client.as_mut() {
                let _ = active_client.clear_activity();
                let _ = active_client.close();
            }
            *client = None;
            return Ok(());
        }

        let Some(client_id) = std::env::var(DISCORD_CLIENT_ID_ENV)
            .ok()
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty())
        else {
            log::warn!("Discord Rich Presence requested but {DISCORD_CLIENT_ID_ENV} is not set");
            return Ok(());
        };

        if client.is_none() {
            let mut next_client = DiscordIpcClient::new(client_id.as_str());
            next_client.connect().map_err(discord_error)?;
            *client = Some(next_client);
        }

        let activity = activity_for_mode(mode, page_label);
        let active_client = client
            .as_mut()
            .ok_or_else(|| AppError::message("discord presence client is not connected"))?;

        if active_client.set_activity(activity.clone()).is_err() {
            active_client.reconnect().map_err(discord_error)?;
            active_client
                .set_activity(activity)
                .map_err(discord_error)?;
        }

        Ok(())
    }
}

fn activity_for_mode(mode: DiscordPresenceMode, page_label: Option<&str>) -> Activity<'static> {
    let activity_type = match mode {
        DiscordPresenceMode::None => {
            unreachable!("DiscordPresenceMode::None is handled before activity_for_mode")
        }
        DiscordPresenceMode::Playing => activity::ActivityType::Playing,
        DiscordPresenceMode::Listening => activity::ActivityType::Listening,
        DiscordPresenceMode::Watching => activity::ActivityType::Watching,
        DiscordPresenceMode::Competing => activity::ActivityType::Competing,
    };

    let state = page_label
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("Winsentials")
        .to_string();

    Activity::new()
        .details("Optimizing Windows")
        .state(state)
        .status_display_type(activity::StatusDisplayType::Name)
        .activity_type(activity_type)
}

fn discord_error(error: discord_rich_presence::error::Error) -> AppError {
    AppError::message(format!("Discord Rich Presence error: {error}"))
}
