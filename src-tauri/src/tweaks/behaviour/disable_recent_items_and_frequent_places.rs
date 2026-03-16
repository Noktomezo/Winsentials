use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::shell::restart_explorer;
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";
const HISTORY_DISABLED: u32 = 0;
const HISTORY_ENABLED: u32 = 1;

const EXPLORER_ADVANCED_KEY: RegKey = RegKey {
    hive: Hive::CurrentUser,
    path: r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced",
};

pub struct DisableRecentItemsAndFrequentPlacesTweak {
    meta: TweakMeta,
}

impl Default for DisableRecentItemsAndFrequentPlacesTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl DisableRecentItemsAndFrequentPlacesTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "disable_recent_items_and_frequent_places".into(),
                category: "behaviour".into(),
                name: "behaviour.tweaks.disableRecentItemsAndFrequentPlaces.name".into(),
                short_description:
                    "behaviour.tweaks.disableRecentItemsAndFrequentPlaces.shortDescription".into(),
                detail_description:
                    "behaviour.tweaks.disableRecentItemsAndFrequentPlaces.detailDescription".into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: ENABLED_VALUE.into(),
                risk: RiskLevel::None,
                risk_description: None,
                requires_action: RequiresAction::RestartApp {
                    app_name: "Explorer".into(),
                },
                min_os_build: Some(10240),
                min_os_ubr: None,
            },
        }
    }

    fn set_values(&self, value: u32) -> Result<(), AppError> {
        EXPLORER_ADVANCED_KEY.set_dword("Start_TrackDocs", value)?;
        EXPLORER_ADVANCED_KEY.set_dword("ShowFrequent", value)?;
        EXPLORER_ADVANCED_KEY.set_dword("ShowRecent", value)?;
        Ok(())
    }

    fn read_values(&self) -> Result<(u32, u32, u32), AppError> {
        let track_docs = match EXPLORER_ADVANCED_KEY.get_dword("Start_TrackDocs") {
            Ok(value) => value,
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
                HISTORY_ENABLED
            }
            Err(error) => return Err(error),
        };

        let show_frequent = match EXPLORER_ADVANCED_KEY.get_dword("ShowFrequent") {
            Ok(value) => value,
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
                HISTORY_ENABLED
            }
            Err(error) => return Err(error),
        };

        let show_recent = match EXPLORER_ADVANCED_KEY.get_dword("ShowRecent") {
            Ok(value) => value,
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
                HISTORY_ENABLED
            }
            Err(error) => return Err(error),
        };

        Ok((track_docs, show_frequent, show_recent))
    }
}

impl Tweak for DisableRecentItemsAndFrequentPlacesTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => self.set_values(HISTORY_DISABLED),
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        self.set_values(HISTORY_ENABLED)
    }

    fn get_status(&self) -> Result<TweakStatus, AppError> {
        let (track_docs, show_frequent, show_recent) = self.read_values()?;
        let is_enabled = track_docs == HISTORY_DISABLED
            && show_frequent == HISTORY_DISABLED
            && show_recent == HISTORY_DISABLED;
        // Partial states (some flags disabled, some not) are neither enabled nor default.
        let is_default = track_docs == HISTORY_ENABLED
            && show_frequent == HISTORY_ENABLED
            && show_recent == HISTORY_ENABLED;

        Ok(TweakStatus {
            current_value: if is_enabled {
                ENABLED_VALUE.into()
            } else {
                DISABLED_VALUE.into()
            },
            is_default,
        })
    }

    fn extra(&self) -> Result<(), AppError> {
        restart_explorer()
    }
}
