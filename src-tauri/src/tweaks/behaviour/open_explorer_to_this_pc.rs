use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::shell::restart_explorer;
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";

const EXPLORER_ADVANCED_KEY: RegKey = RegKey {
    hive: Hive::CurrentUser,
    path: r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced",
};

pub struct OpenExplorerToThisPcTweak {
    meta: TweakMeta,
}

impl Default for OpenExplorerToThisPcTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl OpenExplorerToThisPcTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "open_explorer_to_this_pc".into(),
                category: "behaviour".into(),
                name: "behaviour.tweaks.openExplorerToThisPc.name".into(),
                short_description: "behaviour.tweaks.openExplorerToThisPc.shortDescription".into(),
                detail_description: "behaviour.tweaks.openExplorerToThisPc.detailDescription"
                    .into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: DISABLED_VALUE.into(),
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

    fn is_enabled(&self) -> Result<bool, AppError> {
        match EXPLORER_ADVANCED_KEY.get_dword("LaunchTo") {
            Ok(value) => Ok(value == 1),
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }
}

impl Tweak for OpenExplorerToThisPcTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => EXPLORER_ADVANCED_KEY.set_dword("LaunchTo", 1),
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        EXPLORER_ADVANCED_KEY.set_dword("LaunchTo", 2)
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
