use std::io::ErrorKind;

use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";
const CUSTOM_VALUE: &str = "custom";
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

struct PowershellTelemetryState {
    telemetry: u32,
    script_block_logging: u32,
}

enum DwordSnapshot {
    Missing,
    Present(u32),
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
                recommended_value: DISABLED_VALUE.into(),
                risk: RiskLevel::High,
                risk_description: Some(
                    "privacy.tweaks.disablePowershellTelemetry.riskDescription".into(),
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

    fn read_state(&self) -> Result<PowershellTelemetryState, AppError> {
        Ok(PowershellTelemetryState {
            telemetry: Self::read_dword_or_default(&POWERSHELL_POLICY_KEY, "Telemetry", 1)?,
            script_block_logging: Self::read_dword_or_default(
                &POWERSHELL_SCRIPT_BLOCK_LOGGING_KEY,
                "EnableScriptBlockLogging",
                1,
            )?,
        })
    }

    fn is_enabled(state: &PowershellTelemetryState) -> bool {
        state.telemetry == DISABLED_POLICY_VALUE
            && state.script_block_logging == DISABLED_POLICY_VALUE
    }

    fn is_default(state: &PowershellTelemetryState) -> bool {
        state.telemetry == 1 && state.script_block_logging == 1
    }

    fn apply_with_rollback<F>(&self, writer: F) -> Result<(), AppError>
    where
        F: FnOnce() -> Result<(), AppError>,
    {
        let telemetry_snapshot = Self::snapshot_dword(&POWERSHELL_POLICY_KEY, "Telemetry")?;
        let script_block_logging_snapshot = Self::snapshot_dword(
            &POWERSHELL_SCRIPT_BLOCK_LOGGING_KEY,
            "EnableScriptBlockLogging",
        )?;

        match writer() {
            Ok(()) => Ok(()),
            Err(error) => {
                let mut rollback_errors = Vec::new();

                if let Err(restore_error) =
                    Self::restore_dword(&POWERSHELL_POLICY_KEY, "Telemetry", &telemetry_snapshot)
                {
                    rollback_errors.push(restore_error.to_string());
                }

                if let Err(restore_error) = Self::restore_dword(
                    &POWERSHELL_SCRIPT_BLOCK_LOGGING_KEY,
                    "EnableScriptBlockLogging",
                    &script_block_logging_snapshot,
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
            POWERSHELL_POLICY_KEY.set_dword("Telemetry", DISABLED_POLICY_VALUE)?;
            POWERSHELL_SCRIPT_BLOCK_LOGGING_KEY
                .set_dword("EnableScriptBlockLogging", DISABLED_POLICY_VALUE)
        })
    }

    fn set_default_values(&self) -> Result<(), AppError> {
        self.apply_with_rollback(|| {
            POWERSHELL_POLICY_KEY.delete_value("Telemetry")?;
            POWERSHELL_SCRIPT_BLOCK_LOGGING_KEY.delete_value("EnableScriptBlockLogging")
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

impl Tweak for DisablePowershellTelemetryTweak {
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
