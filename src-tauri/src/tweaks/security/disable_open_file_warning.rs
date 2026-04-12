use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";

const ATTACHMENTS_KEY: RegKey = RegKey {
    hive: Hive::CurrentUser,
    path: r"Software\Microsoft\Windows\CurrentVersion\Policies\Attachments",
};

pub struct DisableOpenFileWarningTweak {
    meta: TweakMeta,
}

impl Default for DisableOpenFileWarningTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl DisableOpenFileWarningTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "disable_open_file_warning".into(),
                category: "security".into(),
                name: "security.tweaks.disableOpenFileWarning.name".into(),
                short_description: "security.tweaks.disableOpenFileWarning.shortDescription".into(),
                detail_description: "security.tweaks.disableOpenFileWarning.detailDescription"
                    .into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: DISABLED_VALUE.into(),
                risk: RiskLevel::Low,
                risk_description: None,
                conflicts: None,
                requires_action: RequiresAction::None,
                min_os_build: Some(10240),
                min_os_ubr: None,
                min_required_memory_gb: None,
            },
        }
    }

    fn is_enabled(&self) -> Result<bool, AppError> {
        match ATTACHMENTS_KEY.get_dword("SaveZoneInformation") {
            Ok(value) => Ok(value == 1),
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }
}

impl Tweak for DisableOpenFileWarningTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => ATTACHMENTS_KEY.set_dword("SaveZoneInformation", 1),
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        ATTACHMENTS_KEY.set_dword("SaveZoneInformation", 2)
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
}
