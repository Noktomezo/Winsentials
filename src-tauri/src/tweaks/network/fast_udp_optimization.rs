use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";
const CUSTOM_VALUE: &str = "custom";

const DEFAULT_THRESHOLD_VALUE: u32 = 1024;
const OPTIMIZED_THRESHOLD_VALUE: u32 = 65536;
const MIN_WINDOWS_10_BUILD: u32 = 10240;

const AFD_PARAMETERS_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SYSTEM\CurrentControlSet\Services\AFD\Parameters",
};

const FAST_SEND_VALUE_NAME: &str = "FastSendDatagramThreshold";
const FAST_COPY_VALUE_NAME: &str = "FastCopyReceiveThreshold";

pub struct FastUdpOptimizationTweak {
    meta: TweakMeta,
}

impl Default for FastUdpOptimizationTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl FastUdpOptimizationTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "fast_udp_optimization".into(),
                category: "network".into(),
                name: "network.tweaks.fastUdpOptimization.name".into(),
                short_description: "network.tweaks.fastUdpOptimization.shortDescription".into(),
                detail_description: "network.tweaks.fastUdpOptimization.detailDescription".into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: ENABLED_VALUE.into(),
                risk: RiskLevel::Low,
                risk_description: Some("network.tweaks.fastUdpOptimization.riskDescription".into()),
                conflicts: None,
                requires_action: RequiresAction::RestartPc,
                min_os_build: Some(MIN_WINDOWS_10_BUILD),
                min_os_ubr: None,
                min_required_memory_gb: None,
            },
        }
    }

    fn read_dword_or_default(value_name: &str) -> Result<u32, AppError> {
        match AFD_PARAMETERS_KEY.get_dword(value_name) {
            Ok(value) => Ok(value),
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
                Ok(DEFAULT_THRESHOLD_VALUE)
            }
            Err(error) => Err(error),
        }
    }

    fn read_state(&self) -> Result<(u32, u32), AppError> {
        Ok((
            Self::read_dword_or_default(FAST_SEND_VALUE_NAME)?,
            Self::read_dword_or_default(FAST_COPY_VALUE_NAME)?,
        ))
    }

    fn write_values(&self, fast_send: u32, fast_copy: u32) -> Result<(), AppError> {
        let (original_fast_send, original_fast_copy) = self.read_state()?;

        let write_result = (|| -> Result<(), AppError> {
            AFD_PARAMETERS_KEY.set_dword(FAST_SEND_VALUE_NAME, fast_send)?;
            AFD_PARAMETERS_KEY.set_dword(FAST_COPY_VALUE_NAME, fast_copy)?;
            Ok(())
        })();

        if let Err(error) = write_result {
            let _ = AFD_PARAMETERS_KEY.set_dword(FAST_SEND_VALUE_NAME, original_fast_send);
            let _ = AFD_PARAMETERS_KEY.set_dword(FAST_COPY_VALUE_NAME, original_fast_copy);
            return Err(error);
        }

        Ok(())
    }

    fn current_value(&self) -> Result<&'static str, AppError> {
        let (fast_send, fast_copy) = self.read_state()?;

        Ok(
            if fast_send == OPTIMIZED_THRESHOLD_VALUE && fast_copy == OPTIMIZED_THRESHOLD_VALUE {
                ENABLED_VALUE
            } else if fast_send == DEFAULT_THRESHOLD_VALUE && fast_copy == DEFAULT_THRESHOLD_VALUE {
                DISABLED_VALUE
            } else {
                CUSTOM_VALUE
            },
        )
    }
}

impl Tweak for FastUdpOptimizationTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => {
                self.write_values(OPTIMIZED_THRESHOLD_VALUE, OPTIMIZED_THRESHOLD_VALUE)
            }
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        self.write_values(DEFAULT_THRESHOLD_VALUE, DEFAULT_THRESHOLD_VALUE)
    }

    fn get_status(&self) -> Result<TweakStatus, AppError> {
        let current_value = self.current_value()?;

        Ok(TweakStatus {
            current_value: current_value.into(),
            is_default: current_value == DISABLED_VALUE,
        })
    }
}
