use std::io::ErrorKind;

use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";
const MIN_WINDOWS_10_BUILD: u32 = 10240;

const DISABLED_TEXT_COLLECTION_VALUE: u32 = 1;
const DEFAULT_TEXT_COLLECTION_VALUE: u32 = 0;
const DISABLED_HARVESTED_WORDS_VALUE: u32 = 0;
const DEFAULT_HARVESTED_WORDS_VALUE: u32 = 1;

const INPUT_PERSONALIZATION_KEY: RegKey = RegKey {
    hive: Hive::CurrentUser,
    path: r"Software\Microsoft\InputPersonalization",
};

const TRAINED_DATA_STORE_KEY: RegKey = RegKey {
    hive: Hive::CurrentUser,
    path: r"Software\Microsoft\InputPersonalization\TrainedDataStore",
};

pub struct DisableInputDataCollectionTweak {
    meta: TweakMeta,
}

impl Default for DisableInputDataCollectionTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl DisableInputDataCollectionTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "disable_input_data_collection".into(),
                category: "privacy".into(),
                name: "privacy.tweaks.disableInputDataCollection.name".into(),
                short_description: "privacy.tweaks.disableInputDataCollection.shortDescription"
                    .into(),
                detail_description: "privacy.tweaks.disableInputDataCollection.detailDescription"
                    .into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: ENABLED_VALUE.into(),
                risk: RiskLevel::Low,
                risk_description: Some(
                    "privacy.tweaks.disableInputDataCollection.riskDescription".into(),
                ),
                conflicts: None,
                requires_action: RequiresAction::RestartPc,
                min_os_build: Some(MIN_WINDOWS_10_BUILD),
                min_os_ubr: None,
            },
        }
    }

    fn read_dword_or_default(key: &RegKey, name: &str, default: u32) -> Result<u32, AppError> {
        match key.get_dword(name) {
            Ok(value) => Ok(value),
            Err(AppError::Io(error)) if error.kind() == ErrorKind::NotFound => Ok(default),
            Err(error) => Err(error),
        }
    }

    fn is_enabled(&self) -> Result<bool, AppError> {
        let restrict_implicit_text_collection = Self::read_dword_or_default(
            &INPUT_PERSONALIZATION_KEY,
            "RestrictImplicitTextCollection",
            DEFAULT_TEXT_COLLECTION_VALUE,
        )?;
        let harvested_words = Self::read_dword_or_default(
            &TRAINED_DATA_STORE_KEY,
            "HarvestedWords",
            DEFAULT_HARVESTED_WORDS_VALUE,
        )?;

        Ok(
            restrict_implicit_text_collection == DISABLED_TEXT_COLLECTION_VALUE
                && harvested_words == DISABLED_HARVESTED_WORDS_VALUE,
        )
    }
}

impl Tweak for DisableInputDataCollectionTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => {
                INPUT_PERSONALIZATION_KEY.set_dword(
                    "RestrictImplicitTextCollection",
                    DISABLED_TEXT_COLLECTION_VALUE,
                )?;
                TRAINED_DATA_STORE_KEY.set_dword("HarvestedWords", DISABLED_HARVESTED_WORDS_VALUE)
            }
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        INPUT_PERSONALIZATION_KEY.delete_value("RestrictImplicitTextCollection")?;
        TRAINED_DATA_STORE_KEY.delete_value("HarvestedWords")
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
