use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";

const EXPLORER_SERIALIZE_KEY: RegKey = RegKey {
    hive: Hive::CurrentUser,
    path: r"Software\Microsoft\Windows\CurrentVersion\Explorer\Serialize",
};

pub struct DisableStartupDelayTweak {
    meta: TweakMeta,
}

impl Default for DisableStartupDelayTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl DisableStartupDelayTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "disable_startup_delay".into(),
                category: "behaviour".into(),
                name: "behaviour.tweaks.disableStartupDelay.name".into(),
                short_description: "behaviour.tweaks.disableStartupDelay.shortDescription".into(),
                detail_description: "behaviour.tweaks.disableStartupDelay.detailDescription".into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: DISABLED_VALUE.into(),
                risk: RiskLevel::Low,
                risk_description: Some(
                    "behaviour.tweaks.disableStartupDelay.riskDescription".into(),
                ),
                conflicts: None,
                requires_action: RequiresAction::Logout,
                min_os_build: Some(10240),
                min_os_ubr: None,
            },
        }
    }

    fn is_enabled(&self) -> Result<bool, AppError> {
        let startup_delay_disabled = match EXPLORER_SERIALIZE_KEY.get_dword("StartupDelayInMSec") {
            Ok(value) => value == 0,
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => false,
            Err(error) => return Err(error),
        };

        let idle_wait_disabled = match EXPLORER_SERIALIZE_KEY.get_dword("WaitForIdleState") {
            Ok(value) => value == 0,
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => false,
            Err(error) => return Err(error),
        };

        Ok(startup_delay_disabled && idle_wait_disabled)
    }
}

impl Tweak for DisableStartupDelayTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => {
                EXPLORER_SERIALIZE_KEY.set_dword("StartupDelayInMSec", 0)?;
                EXPLORER_SERIALIZE_KEY.set_dword("WaitForIdleState", 0)
            }
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        EXPLORER_SERIALIZE_KEY.delete_value("StartupDelayInMSec")?;
        EXPLORER_SERIALIZE_KEY.delete_value("WaitForIdleState")
    }

    fn get_status(&self) -> Result<TweakStatus, AppError> {
        let enabled = self.is_enabled()?;

        Ok(TweakStatus {
            current_value: if enabled {
                ENABLED_VALUE.into()
            } else {
                DISABLED_VALUE.into()
            },
            is_default: !enabled,
        })
    }
}
