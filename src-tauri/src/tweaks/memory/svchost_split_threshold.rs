use std::io::ErrorKind;

use sysinfo::System;

use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";
const CUSTOM_VALUE: &str = "custom";

const DEFAULT_THRESHOLD_KB: u32 = 3_670_016;
const MIN_REQUIRED_MEMORY_GB: u64 = 8;
const MIN_REQUIRED_THRESHOLD_KB: u64 = MIN_REQUIRED_MEMORY_GB * 1024 * 1024;

const CONTROL_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SYSTEM\CurrentControlSet\Control",
};

pub struct SvcHostSplitThresholdTweak {
    meta: TweakMeta,
}

impl Default for SvcHostSplitThresholdTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl SvcHostSplitThresholdTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "svchost_split_threshold".into(),
                category: "memory".into(),
                name: "memory.tweaks.svcHostSplitThreshold.name".into(),
                short_description: "memory.tweaks.svcHostSplitThreshold.shortDescription".into(),
                detail_description: "memory.tweaks.svcHostSplitThreshold.detailDescription".into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: ENABLED_VALUE.into(),
                risk: RiskLevel::Medium,
                risk_description: Some(
                    "memory.tweaks.svcHostSplitThreshold.riskDescription".into(),
                ),
                conflicts: None,
                requires_action: RequiresAction::RestartPc,
                min_os_build: Some(17763),
                min_os_ubr: None,
            },
        }
    }

    fn read_threshold_kb(&self) -> Result<u32, AppError> {
        match CONTROL_KEY.get_dword("SvcHostSplitThresholdInKB") {
            Ok(value) => Ok(value),
            Err(AppError::Io(error)) if error.kind() == ErrorKind::NotFound => {
                Ok(DEFAULT_THRESHOLD_KB)
            }
            Err(error) => Err(error),
        }
    }

    fn recommended_threshold_kb(&self) -> Result<u32, AppError> {
        let system = System::new_all();
        let threshold_kb = system.total_memory() / 1024;

        u32::try_from(threshold_kb).map_err(|_| {
            AppError::message(format!(
                "installed memory is too large for DWORD threshold: {threshold_kb} KB"
            ))
        })
    }

    fn can_apply(threshold_kb: u32) -> bool {
        u64::from(threshold_kb) >= MIN_REQUIRED_THRESHOLD_KB
    }
}

impl Tweak for SvcHostSplitThresholdTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => {
                let threshold_kb = self.recommended_threshold_kb()?;

                if !Self::can_apply(threshold_kb) {
                    return Err(AppError::message(
                        "SvcHost split threshold requires at least 8 GB of installed memory.",
                    ));
                }

                CONTROL_KEY.set_dword("SvcHostSplitThresholdInKB", threshold_kb)
            }
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        CONTROL_KEY.set_dword("SvcHostSplitThresholdInKB", DEFAULT_THRESHOLD_KB)
    }

    fn get_status(&self) -> Result<TweakStatus, AppError> {
        let current_threshold_kb = self.read_threshold_kb()?;
        let recommended_threshold_kb = self.recommended_threshold_kb()?;
        let is_default = current_threshold_kb == DEFAULT_THRESHOLD_KB;
        let is_enabled = !is_default
            && Self::can_apply(recommended_threshold_kb)
            && current_threshold_kb >= recommended_threshold_kb;

        Ok(TweakStatus {
            current_value: if is_enabled {
                ENABLED_VALUE.into()
            } else if is_default {
                DISABLED_VALUE.into()
            } else {
                CUSTOM_VALUE.into()
            },
            is_default,
        })
    }
}
