use std::io::ErrorKind;
use std::sync::{Mutex, OnceLock};

use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";
const CUSTOM_VALUE: &str = "custom";
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

struct InputDataCollectionState {
    restrict_implicit_text_collection: u32,
    harvested_words: u32,
}

enum DwordSnapshot {
    Missing,
    Present(u32),
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
                min_required_memory_gb: None,
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

    fn snapshot_dword(key: &RegKey, name: &str) -> Result<DwordSnapshot, AppError> {
        match key.get_dword(name) {
            Ok(value) => Ok(DwordSnapshot::Present(value)),
            Err(AppError::Io(error)) if error.kind() == ErrorKind::NotFound => {
                Ok(DwordSnapshot::Missing)
            }
            Err(error) => Err(error),
        }
    }

    fn restore_dword(key: &RegKey, name: &str, snapshot: &DwordSnapshot) -> Result<(), AppError> {
        match snapshot {
            DwordSnapshot::Missing => key.delete_value(name),
            DwordSnapshot::Present(value) => key.set_dword(name, *value),
        }
    }

    fn read_state(&self) -> Result<InputDataCollectionState, AppError> {
        Ok(InputDataCollectionState {
            restrict_implicit_text_collection: Self::read_dword_or_default(
                &INPUT_PERSONALIZATION_KEY,
                "RestrictImplicitTextCollection",
                DEFAULT_TEXT_COLLECTION_VALUE,
            )?,
            harvested_words: Self::read_dword_or_default(
                &TRAINED_DATA_STORE_KEY,
                "HarvestedWords",
                DEFAULT_HARVESTED_WORDS_VALUE,
            )?,
        })
    }

    fn is_enabled(state: &InputDataCollectionState) -> bool {
        state.restrict_implicit_text_collection == DISABLED_TEXT_COLLECTION_VALUE
            && state.harvested_words == DISABLED_HARVESTED_WORDS_VALUE
    }

    fn is_default(state: &InputDataCollectionState) -> bool {
        state.restrict_implicit_text_collection == DEFAULT_TEXT_COLLECTION_VALUE
            && state.harvested_words == DEFAULT_HARVESTED_WORDS_VALUE
    }

    fn apply_with_rollback<F>(&self, writer: F) -> Result<(), AppError>
    where
        F: FnOnce() -> Result<(), AppError>,
    {
        let _lock = input_data_collection_operation_lock()
            .lock()
            .map_err(|_| AppError::message("failed to acquire input data collection lock"))?;
        let restrict_snapshot =
            Self::snapshot_dword(&INPUT_PERSONALIZATION_KEY, "RestrictImplicitTextCollection")?;
        let harvested_words_snapshot =
            Self::snapshot_dword(&TRAINED_DATA_STORE_KEY, "HarvestedWords")?;

        match writer() {
            Ok(()) => Ok(()),
            Err(error) => {
                let mut rollback_errors = Vec::new();

                if let Err(restore_error) = Self::restore_dword(
                    &INPUT_PERSONALIZATION_KEY,
                    "RestrictImplicitTextCollection",
                    &restrict_snapshot,
                ) {
                    rollback_errors.push(restore_error.to_string());
                }

                if let Err(restore_error) = Self::restore_dword(
                    &TRAINED_DATA_STORE_KEY,
                    "HarvestedWords",
                    &harvested_words_snapshot,
                ) {
                    rollback_errors.push(restore_error.to_string());
                }

                if rollback_errors.is_empty() {
                    Err(error)
                } else {
                    Err(AppError::message(format!(
                        "{error}; rollback failed: {}",
                        rollback_errors.join("; ")
                    )))
                }
            }
        }
    }

    fn set_tweak_values(&self) -> Result<(), AppError> {
        self.apply_with_rollback(|| {
            INPUT_PERSONALIZATION_KEY.set_dword(
                "RestrictImplicitTextCollection",
                DISABLED_TEXT_COLLECTION_VALUE,
            )?;
            TRAINED_DATA_STORE_KEY.set_dword("HarvestedWords", DISABLED_HARVESTED_WORDS_VALUE)
        })
    }

    fn set_default_values(&self) -> Result<(), AppError> {
        self.apply_with_rollback(|| {
            INPUT_PERSONALIZATION_KEY.delete_value("RestrictImplicitTextCollection")?;
            TRAINED_DATA_STORE_KEY.delete_value("HarvestedWords")
        })
    }

    fn current_value(&self) -> Result<&'static str, AppError> {
        let state = self.read_state()?;

        Ok(if Self::is_enabled(&state) {
            ENABLED_VALUE
        } else if Self::is_default(&state) {
            DISABLED_VALUE
        } else {
            CUSTOM_VALUE
        })
    }
}

fn input_data_collection_operation_lock() -> &'static Mutex<()> {
    static OPERATION_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    OPERATION_LOCK.get_or_init(|| Mutex::new(()))
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
            ENABLED_VALUE => self.set_tweak_values(),
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        self.set_default_values()
    }

    fn get_status(&self) -> Result<TweakStatus, AppError> {
        let current_value = self.current_value()?;

        Ok(TweakStatus {
            current_value: current_value.into(),
            is_default: current_value == DISABLED_VALUE,
        })
    }
}
