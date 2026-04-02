use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";

const NCSI_INTERNET_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SYSTEM\CurrentControlSet\Services\NlaSvc\Parameters\Internet",
};

const NCSI_POLICY_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SOFTWARE\Policies\Microsoft\Windows\NetworkConnectivityStatusIndicator",
};

pub struct DisableNcsiActiveProbingTweak {
    meta: TweakMeta,
}

impl Default for DisableNcsiActiveProbingTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl DisableNcsiActiveProbingTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "disable_ncsi_active_probing".into(),
                category: "network".into(),
                name: "network.tweaks.disableNcsiActiveProbing.name".into(),
                short_description: "network.tweaks.disableNcsiActiveProbing.shortDescription"
                    .into(),
                detail_description: "network.tweaks.disableNcsiActiveProbing.detailDescription"
                    .into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: DISABLED_VALUE.into(),
                risk: RiskLevel::Medium,
                risk_description: Some(
                    "network.tweaks.disableNcsiActiveProbing.riskDescription".into(),
                ),
                conflicts: None,
                requires_action: RequiresAction::RestartPc,
                min_os_build: Some(10240),
                min_os_ubr: None,
            },
        }
    }

    fn is_enabled(&self) -> Result<bool, AppError> {
        let probing_disabled = match NCSI_INTERNET_KEY.get_dword("EnableActiveProbing") {
            Ok(value) => value == 0,
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => false,
            Err(error) => return Err(error),
        };

        let policy_disabled = match NCSI_POLICY_KEY.get_dword("NoActiveProbe") {
            Ok(value) => value == 1,
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => false,
            Err(error) => return Err(error),
        };

        Ok(probing_disabled || policy_disabled)
    }
}

impl Tweak for DisableNcsiActiveProbingTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => {
                NCSI_INTERNET_KEY.set_dword("EnableActiveProbing", 0)?;
                NCSI_POLICY_KEY.set_dword("NoActiveProbe", 1)
            }
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        NCSI_INTERNET_KEY.set_dword("EnableActiveProbing", 1)?;
        NCSI_POLICY_KEY.delete_value("NoActiveProbe")
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
