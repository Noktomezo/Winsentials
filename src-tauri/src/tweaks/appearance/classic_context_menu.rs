use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::shell::restart_explorer;
use crate::tweaks::{
    RequiresAction, RiskLevel, Tweak, TweakConflict, TweakControlType, TweakMeta, TweakStatus,
};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";

const CLASSIC_CONTEXT_MENU_KEY: RegKey = RegKey {
    hive: Hive::CurrentUser,
    path: r"Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}\InprocServer32",
};

const CLASSIC_CONTEXT_MENU_ROOT_KEY: RegKey = RegKey {
    hive: Hive::CurrentUser,
    path: r"Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}",
};

pub struct ClassicContextMenuTweak {
    meta: TweakMeta,
}

impl Default for ClassicContextMenuTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl ClassicContextMenuTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "classic_context_menu".into(),
                category: "appearance".into(),
                name: "appearance.tweaks.classicContextMenu.name".into(),
                short_description: "appearance.tweaks.classicContextMenu.shortDescription".into(),
                detail_description: "appearance.tweaks.classicContextMenu.detailDescription".into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: DISABLED_VALUE.into(),
                risk: RiskLevel::None,
                risk_description: None,
                conflicts: Some(vec![TweakConflict {
                    description: "appearance.tweaks.classicContextMenu.conflicts.windhawk".into(),
                }]),
                requires_action: RequiresAction::RestartApp {
                    app_name: "Explorer".into(),
                },
                min_os_build: Some(22000),
                min_os_ubr: None,
            },
        }
    }

    fn is_enabled(&self) -> Result<bool, AppError> {
        let key = match CLASSIC_CONTEXT_MENU_KEY.open_read() {
            Ok(key) => key,
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(false);
            }
            Err(error) => return Err(error),
        };

        match key.get_value::<String, _>("") {
            Ok(value) => Ok(value.is_empty()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(AppError::from(error)),
        }
    }
}

impl Tweak for ClassicContextMenuTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => CLASSIC_CONTEXT_MENU_KEY.set_string("", ""),
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        CLASSIC_CONTEXT_MENU_ROOT_KEY.delete_subkey_tree()
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
