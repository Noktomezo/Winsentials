use std::fs;
use std::path::PathBuf;

use crate::error::AppError;
use crate::registry::{Hive, RegKey};
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
                control: TweakControlType::Toggle,
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
            Ok(value) => Ok(value == 1),
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
                Ok(is_onedrive_absent()?)
            }
            Err(error) => Err(error),
        }
    }

    fn remove_onedrive() -> Result<(), AppError> {
        if let Some(path) = std::env::var_os("OneDrive").map(PathBuf::from) {
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
        if setup_path.exists() {
            run_duct(setup_path.to_string_lossy().as_ref(), &["/uninstall"])?;
        }

        let _ = run_duct("taskkill", &["/IM", "FileCoAuth.exe", "/F"]);
        let _ = run_duct("taskkill", &["/IM", "explorer.exe", "/F"]);

        remove_dir_if_exists(local_app_data()?.join("Microsoft").join("OneDrive"))?;
        remove_dir_if_exists(PathBuf::from(r"C:\ProgramData\Microsoft OneDrive"))?;

        if let Some(path) = std::env::var_os("OneDrive").map(PathBuf::from) {
            let _ = run_duct(
                "icacls",
                &[
                    path.to_string_lossy().as_ref(),
                    "/grant",
                    "*S-1-5-32-544:(D,DC)",
                ],
            );
        }

        ONEDRIVE_SERVICE_KEY.set_dword("Start", DISABLED_SERVICE_START)?;
        STATE_KEY.set_dword("Removed", 1)
    }

    fn install_onedrive() -> Result<(), AppError> {
        let result = install_with_winget("Microsoft.Onedrive");

        if result.is_ok() {
            ONEDRIVE_SERVICE_KEY.set_dword("Start", AUTOMATIC_SERVICE_START)?;
            STATE_KEY.delete_value("Removed")?;
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

fn run_duct(program: &str, args: &[&str]) -> Result<(), AppError> {
    duct::cmd(program, args)
        .run()
        .map(|_| ())
        .map_err(|error| AppError::CommandFailed {
            command: program.to_string(),
            stderr: error.to_string(),
        })
}

fn install_with_winget(package_id: &str) -> Result<(), AppError> {
    let enable_result = run_duct(
        "winget",
        &[
            "settings",
            "--enable",
            "BypassCertificatePinningForMicrosoftStore",
        ],
    );
    let install_result = enable_result.and_then(|_| {
        run_duct(
            "winget",
            &[
                "install",
                package_id,
                "--source",
                "winget",
                "--accept-package-agreements",
                "--accept-source-agreements",
                "--silent",
            ],
        )
    });
    let disable_result = run_duct(
        "winget",
        &[
            "settings",
            "--disable",
            "BypassCertificatePinningForMicrosoftStore",
        ],
    );

    install_result.and(disable_result)
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

fn remove_dir_if_exists(path: PathBuf) -> Result<(), AppError> {
    if path.exists() {
        fs::remove_dir_all(path)?;
    }

    Ok(())
}

fn is_onedrive_absent() -> Result<bool, AppError> {
    let setup_path = system_root()?.join("System32").join("OneDriveSetup.exe");
    let local_leftovers = local_app_data()?.join("Microsoft").join("OneDrive");
    let program_data_leftovers = PathBuf::from(r"C:\ProgramData\Microsoft OneDrive");

    Ok(!setup_path.exists() && !local_leftovers.exists() && !program_data_leftovers.exists())
}
