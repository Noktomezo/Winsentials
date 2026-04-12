use std::io::ErrorKind;

use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";
const CUSTOM_VALUE: &str = "custom";
const MIN_WINDOWS_10_BUILD: u32 = 10240;

const DISABLED_LOCATION_VALUE: u32 = 1;
const DEFAULT_LOCATION_VALUE: u32 = 0;

const LOCATION_AND_SENSORS_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SOFTWARE\Policies\Microsoft\Windows\LocationAndSensors",
};

pub struct DisableLocationDataCollectionTweak {
    meta: TweakMeta,
}

impl Default for DisableLocationDataCollectionTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl DisableLocationDataCollectionTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "disable_location_data_collection".into(),
                category: "privacy".into(),
                name: "privacy.tweaks.disableLocationDataCollection.name".into(),
                short_description: "privacy.tweaks.disableLocationDataCollection.shortDescription"
                    .into(),
                detail_description:
                    "privacy.tweaks.disableLocationDataCollection.detailDescription".into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: ENABLED_VALUE.into(),
                risk: RiskLevel::Low,
                risk_description: Some(
                    "privacy.tweaks.disableLocationDataCollection.riskDescription".into(),
                ),
                conflicts: None,
                requires_action: RequiresAction::RestartPc,
                min_os_build: Some(MIN_WINDOWS_10_BUILD),
                min_os_ubr: None,
                min_required_memory_gb: None,
            },
        }
    }

    fn current_disabled_value(&self) -> Result<u32, AppError> {
        match LOCATION_AND_SENSORS_KEY.get_dword("DisableLocation") {
            Ok(value) => Ok(value),
            Err(AppError::Io(error)) if error.kind() == ErrorKind::NotFound => {
                Ok(DEFAULT_LOCATION_VALUE)
            }
            Err(error) => Err(error),
        }
    }

    fn current_value(&self) -> Result<&'static str, AppError> {
        Ok(match self.current_disabled_value()? {
            DISABLED_LOCATION_VALUE => ENABLED_VALUE,
            DEFAULT_LOCATION_VALUE => DISABLED_VALUE,
            _ => CUSTOM_VALUE,
        })
    }
}

impl Tweak for DisableLocationDataCollectionTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => {
                LOCATION_AND_SENSORS_KEY.set_dword("DisableLocation", DISABLED_LOCATION_VALUE)
            }
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        LOCATION_AND_SENSORS_KEY.delete_value("DisableLocation")
    }

    fn get_status(&self) -> Result<TweakStatus, AppError> {
        let current_value = self.current_value()?;

        Ok(TweakStatus {
            current_value: current_value.into(),
            is_default: current_value == DISABLED_VALUE,
        })
    }
}
