use std::io::ErrorKind;

use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";
const MIN_WINDOWS_10_BUILD: u32 = 10240;

const DISABLED_WER_VALUE: u32 = 1;
const DEFAULT_WER_VALUE: u32 = 0;

const WINDOWS_ERROR_REPORTING_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SOFTWARE\Microsoft\Windows\Windows Error Reporting",
};

pub struct DisableWindowsErrorReportingTweak {
    meta: TweakMeta,
}

impl Default for DisableWindowsErrorReportingTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl DisableWindowsErrorReportingTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "disable_windows_error_reporting".into(),
                category: "privacy".into(),
                name: "privacy.tweaks.disableWindowsErrorReporting.name".into(),
                short_description: "privacy.tweaks.disableWindowsErrorReporting.shortDescription"
                    .into(),
                detail_description: "privacy.tweaks.disableWindowsErrorReporting.detailDescription"
                    .into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: ENABLED_VALUE.into(),
                risk: RiskLevel::Low,
                risk_description: Some(
                    "privacy.tweaks.disableWindowsErrorReporting.riskDescription".into(),
                ),
                conflicts: None,
                requires_action: RequiresAction::RestartPc,
                min_os_build: Some(MIN_WINDOWS_10_BUILD),
                min_os_ubr: None,
            },
        }
    }

    fn current_disabled_value(&self) -> Result<u32, AppError> {
        match WINDOWS_ERROR_REPORTING_KEY.get_dword("Disabled") {
            Ok(value) => Ok(value),
            Err(AppError::Io(error)) if error.kind() == ErrorKind::NotFound => {
                Ok(DEFAULT_WER_VALUE)
            }
            Err(error) => Err(error),
        }
    }

    fn is_enabled(&self) -> Result<bool, AppError> {
        Ok(self.current_disabled_value()? == DISABLED_WER_VALUE)
    }
}

impl Tweak for DisableWindowsErrorReportingTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => WINDOWS_ERROR_REPORTING_KEY.set_dword("Disabled", DISABLED_WER_VALUE),
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        WINDOWS_ERROR_REPORTING_KEY.delete_value("Disabled")
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
