use std::io::ErrorKind;

use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";
const MIN_WINDOWS_10_BUILD: u32 = 10240;

const DEFAULT_ENABLE_DIAGNOSTICS: u32 = 1;
const DISABLED_ENABLE_DIAGNOSTICS: u32 = 0;

const DOTNET_FRAMEWORK_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SOFTWARE\Microsoft\.NETFramework",
};

pub struct DisableDotnetTelemetryTweak {
    meta: TweakMeta,
}

impl Default for DisableDotnetTelemetryTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl DisableDotnetTelemetryTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "disable_dotnet_telemetry".into(),
                category: "privacy".into(),
                name: "privacy.tweaks.disableDotnetTelemetry.name".into(),
                short_description: "privacy.tweaks.disableDotnetTelemetry.shortDescription".into(),
                detail_description: "privacy.tweaks.disableDotnetTelemetry.detailDescription"
                    .into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: ENABLED_VALUE.into(),
                risk: RiskLevel::Low,
                risk_description: Some(
                    "privacy.tweaks.disableDotnetTelemetry.riskDescription".into(),
                ),
                conflicts: None,
                requires_action: RequiresAction::RestartPc,
                min_os_build: Some(MIN_WINDOWS_10_BUILD),
                min_os_ubr: None,
            },
        }
    }

    fn current_enable_diagnostics(&self) -> Result<u32, AppError> {
        match DOTNET_FRAMEWORK_KEY.get_dword("EnableDiagnostics") {
            Ok(value) => Ok(value),
            Err(AppError::Io(error)) if error.kind() == ErrorKind::NotFound => {
                Ok(DEFAULT_ENABLE_DIAGNOSTICS)
            }
            Err(error) => Err(error),
        }
    }

    fn is_enabled(&self) -> Result<bool, AppError> {
        Ok(self.current_enable_diagnostics()? == DISABLED_ENABLE_DIAGNOSTICS)
    }
}

impl Tweak for DisableDotnetTelemetryTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => {
                DOTNET_FRAMEWORK_KEY.set_dword("EnableDiagnostics", DISABLED_ENABLE_DIAGNOSTICS)
            }
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        DOTNET_FRAMEWORK_KEY.set_dword("EnableDiagnostics", DEFAULT_ENABLE_DIAGNOSTICS)
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
