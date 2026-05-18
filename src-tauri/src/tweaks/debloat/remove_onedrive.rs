use std::fs;
use std::path::PathBuf;

use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::shell::{install_with_winget, run_duct};
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";

const DISABLED_SERVICE_START: u32 = 4;
const AUTOMATIC_SERVICE_START: u32 = 2;

const ONEDRIVE_SERVICE_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SYSTEM\CurrentControlSet\Services\OneSyncSvc",
};

const STATE_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SOFTWARE\Winsentials\TweakState\remove_microsoft_onedrive",
};

pub struct RemoveOneDriveTweak {
    meta: TweakMeta,
}

impl Default for RemoveOneDriveTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl RemoveOneDriveTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "remove_microsoft_onedrive".into(),
                category: "debloat".into(),
                name: "debloat.tweaks.removeMicrosoftOneDrive.name".into(),
                short_description: "debloat.tweaks.removeMicrosoftOneDrive.shortDescription".into(),
                detail_description: "debloat.tweaks.removeMicrosoftOneDrive.detailDescription"
                    .into(),
                control: TweakControlType::Action,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: ENABLED_VALUE.into(),
                risk: RiskLevel::Medium,
                risk_description: Some(
                    "debloat.tweaks.removeMicrosoftOneDrive.riskDescription".into(),
                ),
                conflicts: None,
                requires_action: RequiresAction::None,
                min_os_build: Some(10240),
                min_os_ubr: None,
                min_required_memory_gb: None,
            },
        }
    }

    fn is_enabled(&self) -> Result<bool, AppError> {
        match STATE_KEY.get_dword("Removed") {
            Ok(1) => Ok(is_onedrive_absent()?),
            Ok(_) => Ok(false),
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
                Ok(is_onedrive_absent()?)
            }
            Err(error) => Err(error),
        }
    }

    fn remove_onedrive() -> Result<(), AppError> {
        let onedrive_path = std::env::var_os("OneDrive").map(PathBuf::from);

        if let Some(path) = &onedrive_path {
            let _ = run_duct(
                "icacls",
                &[
                    path.to_string_lossy().as_ref(),
                    "/deny",
                    "*S-1-5-32-544:(D,DC)",
                ],
            );
        }

        let setup_path = system_root()?.join("System32").join("OneDriveSetup.exe");
        let uninstall_result = if setup_path.exists() {
            run_duct(setup_path.to_string_lossy().as_ref(), &["/uninstall"])
        } else {
            Ok(())
        };

        if let Some(path) = &onedrive_path {
            let _ = run_duct(
                "icacls",
                &[
                    path.to_string_lossy().as_ref(),
                    "/remove:d",
                    "*S-1-5-32-544",
                ],
            );
        }

        uninstall_result?;

        let _ = run_duct("taskkill", &["/IM", "FileCoAuth.exe", "/F"]);

        remove_dir_if_exists(local_app_data()?.join("Microsoft").join("OneDrive"))?;
        remove_dir_if_exists(program_data()?.join("Microsoft OneDrive"))?;

        ONEDRIVE_SERVICE_KEY.set_dword("Start", DISABLED_SERVICE_START)?;
        STATE_KEY.set_dword("Removed", 1)
    }

    fn install_onedrive() -> Result<(), AppError> {
        let result = install_with_winget("Microsoft.OneDrive", "winget");

        if result.is_ok() {
            match ONEDRIVE_SERVICE_KEY.set_dword("Start", AUTOMATIC_SERVICE_START) {
                Ok(()) => {}
                Err(error) => {
                    log::warn!(
                        "Failed to restore OneDrive service start type after install: {error}"
                    );
                }
            }
            match STATE_KEY.delete_value("Removed") {
                Ok(()) => {}
                Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
                    log::warn!("OneDrive removal marker was already absent: {error}");
                }
                Err(error) => return Err(error),
            }
        }

        result
    }
}

impl Tweak for RemoveOneDriveTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => Self::remove_onedrive(),
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        Self::install_onedrive()
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

fn system_root() -> Result<PathBuf, AppError> {
    std::env::var_os("SystemRoot")
        .map(PathBuf::from)
        .ok_or_else(|| AppError::message("SystemRoot is not set"))
}

fn local_app_data() -> Result<PathBuf, AppError> {
    std::env::var_os("LocalAppData")
        .map(PathBuf::from)
        .ok_or_else(|| AppError::message("LocalAppData is not set"))
}

fn program_data() -> Result<PathBuf, AppError> {
    std::env::var_os("ProgramData")
        .map(PathBuf::from)
        .ok_or_else(|| AppError::message("ProgramData is not set"))
}

fn remove_dir_if_exists(path: PathBuf) -> Result<(), AppError> {
    if path.exists() {
        fs::remove_dir_all(path)?;
    }

    Ok(())
}

fn is_onedrive_absent() -> Result<bool, AppError> {
    let client_path = local_app_data()?
        .join("Microsoft")
        .join("OneDrive")
        .join("OneDrive.exe");
    let program_data_leftovers = program_data()?.join("Microsoft OneDrive");

    Ok(!client_path.exists() && !program_data_leftovers.exists())
}
