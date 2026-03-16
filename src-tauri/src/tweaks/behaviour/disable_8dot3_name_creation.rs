use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";
const DEFAULT_VALUE: u32 = 2;
const DISABLE_CREATION_VALUE: u32 = 1;

const FILE_SYSTEM_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SYSTEM\CurrentControlSet\Control\FileSystem",
};

pub struct Disable8dot3NameCreationTweak {
    meta: TweakMeta,
}

impl Default for Disable8dot3NameCreationTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl Disable8dot3NameCreationTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "disable_8dot3_name_creation".into(),
                category: "behaviour".into(),
                name: "behaviour.tweaks.disable8dot3NameCreation.name".into(),
                short_description: "behaviour.tweaks.disable8dot3NameCreation.shortDescription"
                    .into(),
                detail_description: "behaviour.tweaks.disable8dot3NameCreation.detailDescription"
                    .into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: ENABLED_VALUE.into(),
                risk: RiskLevel::None,
                risk_description: None,
                requires_action: RequiresAction::RestartPc,
                min_os_build: Some(10240),
                min_os_ubr: None,
            },
        }
    }

    fn is_enabled(&self) -> Result<bool, AppError> {
        match FILE_SYSTEM_KEY.get_dword("NtfsDisable8dot3NameCreation") {
            Ok(value) => Ok(value == DISABLE_CREATION_VALUE),
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }
}

impl Tweak for Disable8dot3NameCreationTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => {
                FILE_SYSTEM_KEY.set_dword("NtfsDisable8dot3NameCreation", DISABLE_CREATION_VALUE)
            }
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        FILE_SYSTEM_KEY.set_dword("NtfsDisable8dot3NameCreation", DEFAULT_VALUE)
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
