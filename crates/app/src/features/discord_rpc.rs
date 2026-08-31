use std::time::{SystemTime, UNIX_EPOCH};

use discord_rich_presence::{DiscordIpc, DiscordIpcClient, activity};
use serde::{Deserialize, Serialize};

pub const DISCORD_CLIENT_ID: &str = "1501589879614869625";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum DiscordRpcActivity {
    #[default]
    Disabled,
    Playing,
    Listening,
    Watching,
    Competing,
}

impl DiscordRpcActivity {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Playing => "playing",
            Self::Listening => "listening",
            Self::Watching => "watching",
            Self::Competing => "competing",
        }
    }

    #[must_use]
    pub fn from_str(s: &str) -> Self {
        match s {
            "playing" => Self::Playing,
            "listening" => Self::Listening,
            "watching" => Self::Watching,
            "competing" => Self::Competing,
            _ => Self::Disabled,
        }
    }

    #[must_use]
    pub const fn label_key(self) -> &'static str {
        match self {
            Self::Disabled => "settings.discord_disabled",
            Self::Playing => "settings.discord_playing",
            Self::Listening => "settings.discord_listening",
            Self::Watching => "settings.discord_watching",
            Self::Competing => "settings.discord_competing",
        }
    }

    #[must_use]
    pub const fn icon(self) -> &'static str {
        match self {
            Self::Disabled => "icons/circle-slash.svg",
            Self::Playing => "icons/gamepad-2.svg",
            Self::Listening => "icons/headphones.svg",
            Self::Watching => "icons/tv.svg",
            Self::Competing => "icons/trophy.svg",
        }
    }
}

use crate::features::navigation::AppRoute;

pub struct DiscordRpcManager {
    current_activity: DiscordRpcActivity,
    current_route: AppRoute,
    client: Option<DiscordIpcClient>,
    start_time: i64,
    windows_build: u32,
}

impl Default for DiscordRpcManager {
    fn default() -> Self {
        Self::new(22000)
    }
}

impl DiscordRpcManager {
    #[must_use]
    pub fn new(windows_build: u32) -> Self {
        let start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(0));
        Self {
            current_activity: DiscordRpcActivity::Disabled,
            current_route: AppRoute::Dashboard,
            client: None,
            start_time,
            windows_build,
        }
    }

    pub fn set_route(&mut self, route: AppRoute) {
        if self.current_route != route {
            self.current_route = route;
            self.refresh_presence();
        }
    }

    pub fn refresh_presence(&mut self) {
        if self.current_activity == DiscordRpcActivity::Disabled {
            return;
        }

        let act_type = match self.current_activity {
            DiscordRpcActivity::Playing => activity::ActivityType::Playing,
            DiscordRpcActivity::Listening => activity::ActivityType::Listening,
            DiscordRpcActivity::Watching => activity::ActivityType::Watching,
            DiscordRpcActivity::Competing => activity::ActivityType::Competing,
            DiscordRpcActivity::Disabled => return,
        };

        if self.client.is_none() {
            if let Ok(mut client) = DiscordIpcClient::new(DISCORD_CLIENT_ID) {
                if client.connect().is_ok() {
                    self.client = Some(client);
                }
            }
        }

        if let Some(ref mut client) = self.client {
            let (applied, total) =
                crate::entities::tweaks::count_applied_tweaks(self.windows_build);
            let state_str = if applied == 1 {
                format!("Applied 1 tweak of {total}")
            } else {
                format!("Applied {applied} tweaks of {total}")
            };
            let page_str = self.current_route.breadcrumb_english();
            let app_name = if cfg!(debug_assertions) {
                "Winsentials (Dev)"
            } else {
                "Winsentials"
            };

            let timestamps = activity::Timestamps::new().start(self.start_time);
            let assets = activity::Assets::new()
                .large_image("app_icon")
                .large_text(app_name);

            let buttons = vec![activity::Button::new(
                "Perfected Windows",
                "https://github.com/Noktomezo/Winsentials",
            )];

            let payload = activity::Activity::new()
                .activity_type(act_type)
                .details(&page_str)
                .state(&state_str)
                .timestamps(timestamps)
                .assets(assets)
                .buttons(buttons);

            if client.set_activity(payload).is_err() {
                // Connection might have dropped, clear client
                let _ = client.close();
                self.client = None;
            }
        }
    }

    pub fn set_activity(&mut self, act: DiscordRpcActivity) {
        self.current_activity = act;

        if act == DiscordRpcActivity::Disabled {
            if let Some(mut client) = self.client.take() {
                let _ = client.close();
            }
            return;
        }

        self.refresh_presence();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discord_rpc_activity_strings() {
        assert_eq!(DiscordRpcActivity::Disabled.as_str(), "disabled");
        assert_eq!(DiscordRpcActivity::Playing.as_str(), "playing");
        assert_eq!(DiscordRpcActivity::Listening.as_str(), "listening");
        assert_eq!(DiscordRpcActivity::Watching.as_str(), "watching");
        assert_eq!(DiscordRpcActivity::Competing.as_str(), "competing");

        assert_eq!(
            DiscordRpcActivity::from_str("disabled"),
            DiscordRpcActivity::Disabled
        );
        assert_eq!(
            DiscordRpcActivity::from_str("playing"),
            DiscordRpcActivity::Playing
        );
        assert_eq!(
            DiscordRpcActivity::from_str("listening"),
            DiscordRpcActivity::Listening
        );
        assert_eq!(
            DiscordRpcActivity::from_str("watching"),
            DiscordRpcActivity::Watching
        );
        assert_eq!(
            DiscordRpcActivity::from_str("competing"),
            DiscordRpcActivity::Competing
        );
        assert_eq!(
            DiscordRpcActivity::from_str("unknown"),
            DiscordRpcActivity::Disabled
        );
    }

    #[test]
    fn test_discord_rpc_activity_serde() {
        #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
        struct TestConfig {
            activity: DiscordRpcActivity,
        }

        let original = TestConfig {
            activity: DiscordRpcActivity::Playing,
        };
        let serialized = toml::to_string(&original).unwrap();
        assert!(serialized.contains("activity = \"playing\""));
        let deserialized: TestConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized, original);
    }
}
