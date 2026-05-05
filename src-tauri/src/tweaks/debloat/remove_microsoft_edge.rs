use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::shell::run_powershell;
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
                control: TweakControlType::Toggle,
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
            duct::cmd(
                setup_path.as_os_str(),
                [
                    "--uninstall",
                    "--system-level",
                    "--force-uninstall",
                    "--delete-profile",
                ],
            )
            .run()
            .map_err(|error| AppError::CommandFailed {
                command: setup_path.display().to_string(),
                stderr: error.to_string(),
            })?;
        }

        STATE_KEY.set_dword("Removed", 1)
    }

    fn install_edge() -> Result<(), AppError> {
        let result = run_powershell(
            r#"
$ErrorActionPreference = 'Stop'

try {
  winget settings --enable BypassCertificatePinningForMicrosoftStore
  winget install Microsoft.Edge --source winget --accept-package-agreements --accept-source-agreements --silent
}
finally {
  winget settings --disable BypassCertificatePinningForMicrosoftStore
}

Write-Host 'Microsoft Edge Installed'
"#,
        );

        if result.is_ok() {
            STATE_KEY.delete_value("Removed")?;
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

    installers.sort();
    Ok(installers.pop())
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
