use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";

const DEFAULT_START_VALUE: u32 = 2;
const DISABLED_START_VALUE: u32 = 4;
const MIN_WINDOWS_10_1809_BUILD: u32 = 17763;

const NDU_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SYSTEM\CurrentControlSet\Services\Ndu",
};

pub struct DisableNduTweak {
    meta: TweakMeta,
}

impl Default for DisableNduTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl DisableNduTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "disable_ndu".into(),
                category: "network".into(),
                name: "network.tweaks.disableNdu.name".into(),
                short_description: "network.tweaks.disableNdu.shortDescription".into(),
                detail_description: "network.tweaks.disableNdu.detailDescription".into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: ENABLED_VALUE.into(),
                risk: RiskLevel::Low,
                risk_description: Some("network.tweaks.disableNdu.riskDescription".into()),
                conflicts: None,
                requires_action: RequiresAction::RestartPc,
                min_os_build: Some(MIN_WINDOWS_10_1809_BUILD),
                min_os_ubr: None,
            },
        }
    }

    fn read_start_or_default() -> Result<u32, AppError> {
        match NDU_KEY.get_dword("Start") {
            Ok(value) => Ok(value),
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
                Ok(DEFAULT_START_VALUE)
            }
            Err(error) => Err(error),
        }
    }
}

impl Tweak for DisableNduTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => NDU_KEY.set_dword("Start", DISABLED_START_VALUE),
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        NDU_KEY.set_dword("Start", DEFAULT_START_VALUE)
    }

    fn get_status(&self) -> Result<TweakStatus, AppError> {
        let start_value = Self::read_start_or_default()?;
        let enabled = start_value == DISABLED_START_VALUE;

        Ok(TweakStatus {
            current_value: if enabled {
                ENABLED_VALUE.into()
            } else {
                DISABLED_VALUE.into()
            },
            is_default: start_value == DEFAULT_START_VALUE,
        })
    }
}
