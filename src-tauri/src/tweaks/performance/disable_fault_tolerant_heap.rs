use std::{env, path::PathBuf, process::Command};

use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";
const DEFAULT_FTH_VALUE: u32 = 1;
const DISABLED_FTH_VALUE: u32 = 0;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

const FTH_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SOFTWARE\Microsoft\FTH",
};

pub struct DisableFaultTolerantHeapTweak {
    meta: TweakMeta,
}

impl Default for DisableFaultTolerantHeapTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl DisableFaultTolerantHeapTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "disable_fault_tolerant_heap".into(),
                category: "performance".into(),
                name: "performance.tweaks.disableFaultTolerantHeap.name".into(),
                short_description: "performance.tweaks.disableFaultTolerantHeap.shortDescription"
                    .into(),
                detail_description: "performance.tweaks.disableFaultTolerantHeap.detailDescription"
                    .into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: ENABLED_VALUE.into(),
                risk: RiskLevel::Low,
                risk_description: Some(
                    "performance.tweaks.disableFaultTolerantHeap.riskDescription".into(),
                ),
                conflicts: None,
                requires_action: RequiresAction::RestartPc,
                min_os_build: Some(10240),
                min_os_ubr: None,
            },
        }
    }

    fn is_enabled(&self) -> Result<bool, AppError> {
        match FTH_KEY.get_dword("Enabled") {
            Ok(value) => Ok(value == DISABLED_FTH_VALUE),
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }

    fn clear_fth_history(&self) -> Result<(), AppError> {
        let resolved_rundll32 = rundll32_path();
        let mut cmd = Command::new(&resolved_rundll32);
        cmd.arg("fthsvc.dll,FthSysprepSpecialize");

        #[cfg(target_os = "windows")]
        cmd.creation_flags(CREATE_NO_WINDOW);

        let output = cmd.output()?;

        if output.status.success() {
            return Ok(());
        }

        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let message = if stderr.is_empty() { stdout } else { stderr };

        Err(AppError::CommandFailed {
            command: format!(
                "{} fthsvc.dll,FthSysprepSpecialize",
                resolved_rundll32.to_string_lossy()
            ),
            stderr: if message.is_empty() {
                "unknown error".into()
            } else {
                message
            },
        })
    }
}

impl Tweak for DisableFaultTolerantHeapTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => {
                self.clear_fth_history()?;
                FTH_KEY.set_dword("Enabled", DISABLED_FTH_VALUE)
            }
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        FTH_KEY.set_dword("Enabled", DEFAULT_FTH_VALUE)
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

fn rundll32_path() -> PathBuf {
    env::var_os("SystemRoot")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(r"C:\Windows"))
        .join("System32")
        .join("rundll32.exe")
}
