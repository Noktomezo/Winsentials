use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";
const DEFAULT_NO_DRIVE_TYPE_AUTORUN: u32 = 0x91;
const DISABLED_NO_DRIVE_TYPE_AUTORUN: u32 = 0xFF;

const EXPLORER_POLICIES_KEY: RegKey = RegKey {
    hive: Hive::CurrentUser,
    path: r"Software\Microsoft\Windows\CurrentVersion\Policies\Explorer",
};

const AUTOPLAY_HANDLERS_KEY: RegKey = RegKey {
    hive: Hive::CurrentUser,
    path: r"Software\Microsoft\Windows\CurrentVersion\Explorer\AutoplayHandlers",
};

pub struct DisableAutoplayTweak {
    meta: TweakMeta,
}

impl Default for DisableAutoplayTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl DisableAutoplayTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "disable_autoplay".into(),
                category: "security".into(),
                name: "security.tweaks.disableAutoplay.name".into(),
                short_description: "security.tweaks.disableAutoplay.shortDescription".into(),
                detail_description: "security.tweaks.disableAutoplay.detailDescription".into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: ENABLED_VALUE.into(),
                risk: RiskLevel::None,
                risk_description: None,
                conflicts: None,
                requires_action: RequiresAction::RestartPc,
                min_os_build: Some(10240),
                min_os_ubr: None,
                min_required_memory_gb: None,
            },
        }
    }

    fn is_enabled(&self) -> Result<bool, AppError> {
        let no_drive_type_autorun_disabled =
            match EXPLORER_POLICIES_KEY.get_dword("NoDriveTypeAutoRun") {
                Ok(value) => value == DISABLED_NO_DRIVE_TYPE_AUTORUN,
                Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => false,
                Err(error) => return Err(error),
            };

        let autoplay_handlers_disabled = match AUTOPLAY_HANDLERS_KEY.get_dword("DisableAutoplay") {
            Ok(value) => value == 1,
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => false,
            Err(error) => return Err(error),
        };

        Ok(no_drive_type_autorun_disabled && autoplay_handlers_disabled)
    }
}

impl Tweak for DisableAutoplayTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => {
                EXPLORER_POLICIES_KEY
                    .set_dword("NoDriveTypeAutoRun", DISABLED_NO_DRIVE_TYPE_AUTORUN)?;
                AUTOPLAY_HANDLERS_KEY.set_dword("DisableAutoplay", 1)
            }
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        EXPLORER_POLICIES_KEY.set_dword("NoDriveTypeAutoRun", DEFAULT_NO_DRIVE_TYPE_AUTORUN)?;
        AUTOPLAY_HANDLERS_KEY.set_dword("DisableAutoplay", 0)
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
