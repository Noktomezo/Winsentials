use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";

const UAC_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System",
};

pub struct DisableUserAccountControlTweak {
    meta: TweakMeta,
}

impl Default for DisableUserAccountControlTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl DisableUserAccountControlTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "disable_user_account_control".into(),
                category: "security".into(),
                name: "security.tweaks.disableUserAccountControl.name".into(),
                short_description: "security.tweaks.disableUserAccountControl.shortDescription"
                    .into(),
                detail_description: "security.tweaks.disableUserAccountControl.detailDescription"
                    .into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: DISABLED_VALUE.into(),
                risk: RiskLevel::Low,
                risk_description: Some(
                    "security.tweaks.disableUserAccountControl.riskDescription".into(),
                ),
                conflicts: None,
                requires_action: RequiresAction::RestartPc,
                min_os_build: Some(10240),
                min_os_ubr: None,
                min_required_memory_gb: None,
            },
        }
    }

    fn is_enabled(&self) -> Result<bool, AppError> {
        match UAC_KEY.get_dword("EnableLUA") {
            Ok(value) => Ok(value == 0),
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }
}

impl Tweak for DisableUserAccountControlTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => UAC_KEY.set_dword("EnableLUA", 0),
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        UAC_KEY.set_dword("EnableLUA", 1)
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
}
