use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::shell::restart_explorer;
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";
const REMOVE_SUFFIX_VALUE: [u8; 4] = [0x00, 0x00, 0x00, 0x00];

const EXPLORER_KEY: RegKey = RegKey {
    hive: Hive::CurrentUser,
    path: r"Software\Microsoft\Windows\CurrentVersion\Explorer",
};

pub struct RemoveShortcutSuffixTweak {
    meta: TweakMeta,
}

impl Default for RemoveShortcutSuffixTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl RemoveShortcutSuffixTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "remove_shortcut_suffix".into(),
                category: "appearance".into(),
                name: "appearance.tweaks.removeShortcutSuffix.name".into(),
                short_description: "appearance.tweaks.removeShortcutSuffix.shortDescription".into(),
                detail_description: "appearance.tweaks.removeShortcutSuffix.detailDescription"
                    .into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: DISABLED_VALUE.into(),
                risk: RiskLevel::None,
                risk_description: None,
                conflicts: None,
                requires_action: RequiresAction::RestartApp {
                    app_name: "Explorer".into(),
                },
                min_os_build: Some(10240),
                min_os_ubr: None,
            },
        }
    }

    fn is_enabled(&self) -> Result<bool, AppError> {
        match EXPLORER_KEY.get_binary("link") {
            Ok(value) => Ok(value.first().copied() == Some(0x00)),
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }
}

impl Tweak for RemoveShortcutSuffixTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => EXPLORER_KEY.set_binary("link", &REMOVE_SUFFIX_VALUE),
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        EXPLORER_KEY.delete_value("link")
    }

    fn get_status(&self) -> Result<TweakStatus, AppError> {
        Ok(TweakStatus {
            current_value: if self.is_enabled()? {
                ENABLED_VALUE.into()
            } else {
                DISABLED_VALUE.into()
            },
            is_default: !self.is_enabled()?,
        })
    }

    fn extra(&self) -> Result<(), AppError> {
        restart_explorer()
    }
}
