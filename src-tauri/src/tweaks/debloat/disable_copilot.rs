use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::shell::{install_with_winget, run_powershell};
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";

const STATE_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SOFTWARE\Winsentials\TweakState\disable_microsoft_copilot",
};

pub struct DisableCopilotTweak {
    meta: TweakMeta,
}

impl Default for DisableCopilotTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl DisableCopilotTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "disable_microsoft_copilot".into(),
                category: "debloat".into(),
                name: "debloat.tweaks.disableMicrosoftCopilot.name".into(),
                short_description: "debloat.tweaks.disableMicrosoftCopilot.shortDescription".into(),
                detail_description: "debloat.tweaks.disableMicrosoftCopilot.detailDescription"
                    .into(),
                control: TweakControlType::Action,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: ENABLED_VALUE.into(),
                risk: RiskLevel::Medium,
                risk_description: Some(
                    "debloat.tweaks.disableMicrosoftCopilot.riskDescription".into(),
                ),
                conflicts: None,
                requires_action: RequiresAction::None,
                min_os_build: Some(22631),
                min_os_ubr: None,
                min_required_memory_gb: None,
            },
        }
    }

    fn is_enabled(&self) -> Result<bool, AppError> {
        match STATE_KEY.get_dword("Removed") {
            Ok(1) => Ok(!copilot_packages_present()?),
            Ok(_) => Ok(false),
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
                Ok(!copilot_packages_present()?)
            }
            Err(error) => Err(error),
        }
    }

    fn remove_copilot() -> Result<(), AppError> {
        run_powershell(
            r#"
$ErrorActionPreference = 'Stop'

Get-AppxPackage -AllUsers '*Copilot*' | Remove-AppxPackage -AllUsers -ErrorAction SilentlyContinue

$appx = (Get-AppxPackage 'MicrosoftWindows.Client.CoreAI' -ErrorAction SilentlyContinue).PackageFullName
if ($appx) {
  $sid = $null
  try {
    $sid = [System.Security.Principal.NTAccount]::new($Env:UserName).Translate([System.Security.Principal.SecurityIdentifier]).Value
  } catch {
    try {
      $formatArg = '/' + [char]102 + [char]111
      $whoami = whoami /user $formatArg csv /nh
      if ($whoami) {
        $sid = ($whoami | ConvertFrom-Csv -Header Name,Sid).Sid
      }
    } catch {}
  }

  if ($sid) {
    New-Item "HKLM:\SOFTWARE\Microsoft\Windows\CurrentVersion\Appx\AppxAllUserStore\EndOfLife\$sid\$appx" -Force | Out-Null
  }
  Remove-AppxPackage $appx -ErrorAction SilentlyContinue
}

Write-Host 'Copilot Removed'
"#,
        )?;
        STATE_KEY.set_dword("Removed", 1)
    }

    fn install_copilot() -> Result<(), AppError> {
        let result = install_with_winget("9NHT9RB2F4HD", "msstore");

        if result.is_ok()
            && let Err(error) = STATE_KEY.delete_value("Removed")
        {
            log::warn!("failed to delete Copilot removal marker: {error}");
        }

        result.map(|_| ())
    }
}

fn copilot_packages_present() -> Result<bool, AppError> {
    let output = run_powershell(
        r#"
$packages = Get-AppxPackage -AllUsers '*Copilot*' -ErrorAction SilentlyContinue
if ($packages) { 'present' }
"#,
    )?;

    Ok(!output.trim().is_empty())
}

impl Tweak for DisableCopilotTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => Self::remove_copilot(),
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        Self::install_copilot()
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
