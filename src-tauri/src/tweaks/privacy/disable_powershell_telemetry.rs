use std::io::ErrorKind;

use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";
const MIN_WINDOWS_10_BUILD: u32 = 10240;

const DISABLED_POLICY_VALUE: u32 = 0;

const POWERSHELL_POLICY_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SOFTWARE\Policies\Microsoft\Windows\PowerShell",
};

const POWERSHELL_SCRIPT_BLOCK_LOGGING_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SOFTWARE\Policies\Microsoft\Windows\PowerShell\ScriptBlockLogging",
};

pub struct DisablePowershellTelemetryTweak {
    meta: TweakMeta,
}

impl Default for DisablePowershellTelemetryTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl DisablePowershellTelemetryTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "disable_powershell_telemetry".into(),
                category: "privacy".into(),
                name: "privacy.tweaks.disablePowershellTelemetry.name".into(),
                short_description: "privacy.tweaks.disablePowershellTelemetry.shortDescription"
                    .into(),
                detail_description: "privacy.tweaks.disablePowershellTelemetry.detailDescription"
                    .into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: ENABLED_VALUE.into(),
                risk: RiskLevel::Low,
                risk_description: Some(
                    "privacy.tweaks.disablePowershellTelemetry.riskDescription".into(),
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
        let telemetry = Self::read_dword_or_default(&POWERSHELL_POLICY_KEY, "Telemetry", 1)?;
        let script_block_logging = Self::read_dword_or_default(
            &POWERSHELL_SCRIPT_BLOCK_LOGGING_KEY,
            "EnableScriptBlockLogging",
            1,
        )?;

        Ok(telemetry == DISABLED_POLICY_VALUE && script_block_logging == DISABLED_POLICY_VALUE)
    }
}

impl Tweak for DisablePowershellTelemetryTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => {
                POWERSHELL_POLICY_KEY.set_dword("Telemetry", DISABLED_POLICY_VALUE)?;
                POWERSHELL_SCRIPT_BLOCK_LOGGING_KEY
                    .set_dword("EnableScriptBlockLogging", DISABLED_POLICY_VALUE)
            }
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        POWERSHELL_POLICY_KEY.delete_value("Telemetry")?;
        POWERSHELL_SCRIPT_BLOCK_LOGGING_KEY.delete_value("EnableScriptBlockLogging")
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
