use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::shell::restart_explorer;
use crate::tweaks::{
    RequiresAction, RiskLevel, Tweak, TweakConflict, TweakControlType, TweakMeta, TweakStatus,
};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";
const SHORTCUT_ARROW_ICON: &str = r"%windir%\System32\shell32.dll,-50";

const SHELL_ICONS_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Shell Icons",
};

pub struct RemoveShortcutArrowsTweak {
    meta: TweakMeta,
}

impl Default for RemoveShortcutArrowsTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl RemoveShortcutArrowsTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "remove_shortcut_arrows".into(),
                category: "appearance".into(),
                name: "appearance.tweaks.removeShortcutArrows.name".into(),
                short_description: "appearance.tweaks.removeShortcutArrows.shortDescription".into(),
                detail_description: "appearance.tweaks.removeShortcutArrows.detailDescription"
                    .into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: DISABLED_VALUE.into(),
                risk: RiskLevel::None,
                risk_description: None,
                conflicts: Some(vec![TweakConflict {
                    description:
                        "appearance.tweaks.removeShortcutArrows.conflicts.windhawkTransparentWindows"
                            .into(),
                }]),
                requires_action: RequiresAction::RestartApp {
                    app_name: "Explorer".into(),
                },
                min_os_build: Some(10240),
                min_os_ubr: None,
            },
        }
    }

    fn is_enabled(&self) -> Result<bool, AppError> {
        match SHELL_ICONS_KEY.get_string("29") {
            Ok(value) => Ok(value.eq_ignore_ascii_case(SHORTCUT_ARROW_ICON)),
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }
}

impl Tweak for RemoveShortcutArrowsTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => SHELL_ICONS_KEY.set_string("29", SHORTCUT_ARROW_ICON),
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        SHELL_ICONS_KEY.delete_value("29")
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
