use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::shell::restart_explorer;
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";
const MIN_WINDOWS_10_CREATORS_UPDATE_BUILD: u32 = 15063;

const SECURITY_CENTER_NOTIFICATIONS_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SOFTWARE\Policies\Microsoft\Windows Defender Security Center\Notifications",
};

pub struct DisableSecurityCenterNotificationsTweak {
    meta: TweakMeta,
}

impl Default for DisableSecurityCenterNotificationsTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl DisableSecurityCenterNotificationsTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "disable_security_center_notifications".into(),
                category: "security".into(),
                name: "security.tweaks.disableSecurityCenterNotifications.name".into(),
                short_description:
                    "security.tweaks.disableSecurityCenterNotifications.shortDescription".into(),
                detail_description:
                    "security.tweaks.disableSecurityCenterNotifications.detailDescription".into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: DISABLED_VALUE.into(),
                risk: RiskLevel::None,
                risk_description: None,
                requires_action: RequiresAction::RestartApp {
                    app_name: "Explorer".into(),
                },
                min_os_build: Some(MIN_WINDOWS_10_CREATORS_UPDATE_BUILD),
                min_os_ubr: None,
            },
        }
    }

    fn is_enabled(&self) -> Result<bool, AppError> {
        match SECURITY_CENTER_NOTIFICATIONS_KEY.get_dword("DisableNotifications") {
            Ok(value) => Ok(value == 1),
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }
}

impl Tweak for DisableSecurityCenterNotificationsTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => SECURITY_CENTER_NOTIFICATIONS_KEY.set_dword("DisableNotifications", 1),
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        SECURITY_CENTER_NOTIFICATIONS_KEY.delete_value("DisableNotifications")
    }

    fn get_status(&self) -> Result<TweakStatus, AppError> {
        let is_enabled = self.is_enabled()?;

        Ok(TweakStatus {
            current_value: if is_enabled {
                ENABLED_VALUE.into()
            } else {
                DISABLED_VALUE.into()
            },
            is_default: !is_enabled,
        })
    }

    fn extra(&self) -> Result<(), AppError> {
        restart_explorer()
    }
}
