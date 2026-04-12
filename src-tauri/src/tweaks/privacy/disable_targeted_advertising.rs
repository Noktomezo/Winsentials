use std::io::ErrorKind;

use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";
const MIN_WINDOWS_10_BUILD: u32 = 10240;

const DISABLED_ADVERTISING_ID_VALUE: u32 = 0;
const DEFAULT_ADVERTISING_ID_VALUE: u32 = 1;

const ADVERTISING_INFO_KEY: RegKey = RegKey {
    hive: Hive::CurrentUser,
    path: r"Software\Microsoft\Windows\CurrentVersion\AdvertisingInfo",
};

pub struct DisableTargetedAdvertisingTweak {
    meta: TweakMeta,
}

impl Default for DisableTargetedAdvertisingTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl DisableTargetedAdvertisingTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "disable_targeted_advertising".into(),
                category: "privacy".into(),
                name: "privacy.tweaks.disableTargetedAdvertising.name".into(),
                short_description: "privacy.tweaks.disableTargetedAdvertising.shortDescription"
                    .into(),
                detail_description: "privacy.tweaks.disableTargetedAdvertising.detailDescription"
                    .into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: ENABLED_VALUE.into(),
                risk: RiskLevel::Low,
                risk_description: Some(
                    "privacy.tweaks.disableTargetedAdvertising.riskDescription".into(),
                ),
                conflicts: None,
                requires_action: RequiresAction::RestartPc,
                min_os_build: Some(MIN_WINDOWS_10_BUILD),
                min_os_ubr: None,
                min_required_memory_gb: None,
            },
        }
    }

    fn current_enabled_value(&self) -> Result<u32, AppError> {
        match ADVERTISING_INFO_KEY.get_dword("Enabled") {
            Ok(value) => Ok(value),
            Err(AppError::Io(error)) if error.kind() == ErrorKind::NotFound => {
                Ok(DEFAULT_ADVERTISING_ID_VALUE)
            }
            Err(error) => Err(error),
        }
    }

    fn is_enabled(&self) -> Result<bool, AppError> {
        Ok(self.current_enabled_value()? == DISABLED_ADVERTISING_ID_VALUE)
    }
}

impl Tweak for DisableTargetedAdvertisingTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => {
                ADVERTISING_INFO_KEY.set_dword("Enabled", DISABLED_ADVERTISING_ID_VALUE)
            }
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        ADVERTISING_INFO_KEY.delete_value("Enabled")
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
