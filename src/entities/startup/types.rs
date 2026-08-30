use gpui::Rgba;
use serde::{Deserialize, Serialize};

use crate::shared::theme::Theme;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StartupSource {
    Registry,
    StartupFolder,
    Service,
    ScheduledTask,
}

impl StartupSource {
    #[must_use]
    pub const fn icon(self) -> &'static str {
        match self {
            Self::Registry => "icons/binary.svg",
            Self::StartupFolder => "icons/folder.svg",
            Self::Service => "icons/cog.svg",
            Self::ScheduledTask => "icons/clock.svg",
        }
    }

    #[must_use]
    pub fn label(self) -> String {
        match self {
            Self::Registry => rust_i18n::t!("startup.source_registry").to_string(),
            Self::StartupFolder => rust_i18n::t!("startup.source_folder").to_string(),
            Self::Service => rust_i18n::t!("startup.source_service").to_string(),
            Self::ScheduledTask => rust_i18n::t!("startup.source_task").to_string(),
        }
    }

    #[must_use]
    pub fn color(self, theme: &Theme) -> Rgba {
        match self {
            Self::Registry => theme.accent_blue,
            Self::StartupFolder => theme.accent_green,
            Self::Service => theme.accent_orange,
            Self::ScheduledTask => theme.accent_purple,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StartupScope {
    CurrentUser,
    AllUsers,
}

impl StartupScope {
    #[must_use]
    pub const fn icon(self) -> &'static str {
        match self {
            Self::CurrentUser => "icons/user.svg",
            Self::AllUsers => "icons/users.svg",
        }
    }

    #[must_use]
    pub fn label(self) -> String {
        match self {
            Self::CurrentUser => rust_i18n::t!("startup.scope_current_user").to_string(),
            Self::AllUsers => rust_i18n::t!("startup.scope_all_users").to_string(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum StartupStatus {
    Enabled,
    Disabled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StartupEntry {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub publisher: Option<String>,
    pub source: StartupSource,
    pub scope: StartupScope,
    pub status: StartupStatus,
    pub command: Option<String>,
    pub target_path: Option<String>,
    pub icon_path: Option<String>,
    pub location_label: String,
    pub raw_id: String,
}
