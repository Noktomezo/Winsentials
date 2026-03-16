use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";

const PSCHED_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SOFTWARE\Policies\Microsoft\Windows\Psched",
};

pub struct DisableQosBandwidthLimitTweak {
    meta: TweakMeta,
}

impl Default for DisableQosBandwidthLimitTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl DisableQosBandwidthLimitTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "disable_qos_bandwidth_limit".into(),
                category: "network".into(),
                name: "network.tweaks.disableQosBandwidthLimit.name".into(),
                short_description: "network.tweaks.disableQosBandwidthLimit.shortDescription"
                    .into(),
                detail_description: "network.tweaks.disableQosBandwidthLimit.detailDescription"
                    .into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: ENABLED_VALUE.into(),
                risk: RiskLevel::Low,
                risk_description: Some(
                    "network.tweaks.disableQosBandwidthLimit.riskDescription".into(),
                ),
                requires_action: RequiresAction::RestartPc,
                min_os_build: Some(10240),
                min_os_ubr: None,
            },
        }
    }

    fn is_enabled(&self) -> Result<bool, AppError> {
        match PSCHED_KEY.get_dword("NonBestEffortLimit") {
            Ok(value) => Ok(value == 0),
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }
}

impl Tweak for DisableQosBandwidthLimitTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => PSCHED_KEY.set_dword("NonBestEffortLimit", 0),
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        PSCHED_KEY.delete_value("NonBestEffortLimit")
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
