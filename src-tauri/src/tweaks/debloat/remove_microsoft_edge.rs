use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::shell::{install_with_winget, run_duct, run_powershell};
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";

const STATE_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SOFTWARE\Winsentials\TweakState\remove_microsoft_edge",
};

pub struct RemoveMicrosoftEdgeTweak {
    meta: TweakMeta,
}

impl Default for RemoveMicrosoftEdgeTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl RemoveMicrosoftEdgeTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "remove_microsoft_edge".into(),
                category: "debloat".into(),
                name: "debloat.tweaks.removeMicrosoftEdge.name".into(),
                short_description: "debloat.tweaks.removeMicrosoftEdge.shortDescription".into(),
                detail_description: "debloat.tweaks.removeMicrosoftEdge.detailDescription".into(),
                control: TweakControlType::Action,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: ENABLED_VALUE.into(),
                risk: RiskLevel::High,
                risk_description: Some("debloat.tweaks.removeMicrosoftEdge.riskDescription".into()),
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
                Ok(edge_setup_path()?.is_none())
            }
            Err(error) => Err(error),
        }
    }

    fn remove_edge() -> Result<(), AppError> {
        run_powershell(
            r#"
$ErrorActionPreference = 'Stop'

New-Item -Path "$Env:SystemRoot\SystemApps\Microsoft.MicrosoftEdge_8wekyb3d8bbwe\MicrosoftEdge.exe" -Force | Out-Null
"#,
        )?;

        if let Some(setup_path) = edge_setup_path()? {
            let level_flag = edge_install_level_flag(&setup_path);
            let setup = setup_path.to_string_lossy();
            run_duct(
                setup.as_ref(),
                &[
                    "--uninstall",
                    level_flag,
                    "--force-uninstall",
                    "--delete-profile",
                ],
            )?;
        }

        STATE_KEY.set_dword("Removed", 1)
    }

    fn install_edge() -> Result<(), AppError> {
        let cached_result = match edge_setup_path()? {
            Some(setup_path) => {
                let setup = setup_path.to_string_lossy();
                run_duct(
                    setup.as_ref(),
                    &["--install", edge_install_level_flag(&setup_path)],
                )
            }
            None => Err(AppError::message(
                "Microsoft Edge cached setup.exe not found",
            )),
        };

        let result = cached_result.or_else(|error| {
            log::warn!("failed to install Edge from cached setup.exe: {error}");
            install_with_winget("Microsoft.Edge", "winget")
        });

        if result.is_ok() {
            match STATE_KEY.delete_value("Removed") {
                Ok(()) => {}
                Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error),
            }
        }

        result.map(|_| ())
    }
}

fn edge_setup_path() -> Result<Option<std::path::PathBuf>, AppError> {
    let mut candidates = Vec::new();

    if let Some(program_files_x86) = std::env::var_os("ProgramFiles(x86)") {
        candidates.push(std::path::PathBuf::from(program_files_x86));
    }

    if let Some(program_files) = std::env::var_os("ProgramFiles") {
        candidates.push(std::path::PathBuf::from(program_files));
    }

    if let Some(local_app_data) = std::env::var_os("LocalAppData") {
        candidates.push(std::path::PathBuf::from(local_app_data));
    }

    let mut installers = Vec::new();
    for base in candidates {
        let app_dir = base.join("Microsoft").join("Edge").join("Application");
        let Ok(versions) = std::fs::read_dir(app_dir) else {
            continue;
        };

        for version in versions.flatten() {
            let setup = version.path().join("Installer").join("setup.exe");
            if setup.is_file() {
                installers.push(setup);
            }
        }
    }

    installers.sort_by_key(|path| edge_setup_version(path));
    Ok(installers.pop())
}

fn edge_setup_version(path: &std::path::Path) -> Vec<u32> {
    path.parent()
        .and_then(|path| path.parent())
        .and_then(|path| path.file_name())
        .and_then(|name| name.to_str())
        .map(|version| {
            version
                .split('.')
                .map(|segment| segment.parse::<u32>().unwrap_or(0))
                .collect()
        })
        .unwrap_or_default()
}

fn edge_install_level_flag(path: &std::path::Path) -> &'static str {
    let user_profile = std::env::var_os("USERPROFILE").map(std::path::PathBuf::from);
    if user_profile
        .as_ref()
        .is_some_and(|profile| path.starts_with(profile))
    {
        "--user-level"
    } else {
        "--system-level"
    }
}

impl Tweak for RemoveMicrosoftEdgeTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => Self::remove_edge(),
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        Self::install_edge()
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
