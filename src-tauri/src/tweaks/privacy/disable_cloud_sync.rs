use std::io::ErrorKind;

use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::tweaks::{
    RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakOption, TweakStatus,
};

const DEFAULT_VALUE: &str = "default";
const PARTIAL_VALUE: &str = "partial";
const FULL_VALUE: &str = "full";
const CUSTOM_VALUE: &str = "custom";

const MIN_WINDOWS_10_BUILD: u32 = 10240;

const PARTIAL_SYNC_POLICY_VALUE: u32 = 0;
const FULL_SYNC_POLICY_VALUE: u32 = 5;

const SETTING_SYNC_KEY: RegKey = RegKey {
    hive: Hive::CurrentUser,
    path: r"Software\Microsoft\Windows\CurrentVersion\SettingSync",
};

pub struct DisableCloudSyncTweak {
    meta: TweakMeta,
}

impl Default for DisableCloudSyncTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl DisableCloudSyncTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "disable_cloud_sync".into(),
                category: "privacy".into(),
                name: "privacy.tweaks.disableCloudSync.name".into(),
                short_description: "privacy.tweaks.disableCloudSync.shortDescription".into(),
                detail_description: "privacy.tweaks.disableCloudSync.detailDescription".into(),
                control: TweakControlType::Dropdown {
                    options: vec![
                        TweakOption {
                            label: "privacy.tweaks.disableCloudSync.options.default".into(),
                            value: DEFAULT_VALUE.into(),
                        },
                        TweakOption {
                            label: "privacy.tweaks.disableCloudSync.options.partial".into(),
                            value: PARTIAL_VALUE.into(),
                        },
                        TweakOption {
                            label: "privacy.tweaks.disableCloudSync.options.full".into(),
                            value: FULL_VALUE.into(),
                        },
                    ],
                },
                current_value: DEFAULT_VALUE.into(),
                default_value: DEFAULT_VALUE.into(),
                recommended_value: FULL_VALUE.into(),
                risk: RiskLevel::Medium,
                risk_description: Some("privacy.tweaks.disableCloudSync.riskDescription".into()),
                conflicts: None,
                requires_action: RequiresAction::RestartPc,
                min_os_build: Some(MIN_WINDOWS_10_BUILD),
                min_os_ubr: None,
            },
        }
    }

    fn read_sync_policy(&self) -> Result<Option<u32>, AppError> {
        match SETTING_SYNC_KEY.get_dword("SyncPolicy") {
            Ok(value) => Ok(Some(value)),
            Err(AppError::Io(error)) if error.kind() == ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error),
        }
    }

    fn current_value(&self) -> Result<&'static str, AppError> {
        match self.read_sync_policy()? {
            None => Ok(DEFAULT_VALUE),
            Some(PARTIAL_SYNC_POLICY_VALUE) => Ok(PARTIAL_VALUE),
            Some(FULL_SYNC_POLICY_VALUE) => Ok(FULL_VALUE),
            Some(_) => Ok(CUSTOM_VALUE),
        }
    }
}

impl Tweak for DisableCloudSyncTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            DEFAULT_VALUE => self.reset(),
            PARTIAL_VALUE => SETTING_SYNC_KEY.set_dword("SyncPolicy", PARTIAL_SYNC_POLICY_VALUE),
            FULL_VALUE => SETTING_SYNC_KEY.set_dword("SyncPolicy", FULL_SYNC_POLICY_VALUE),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        SETTING_SYNC_KEY.delete_value("SyncPolicy")
    }

    fn get_status(&self) -> Result<TweakStatus, AppError> {
        let current_value = self.current_value()?;

        Ok(TweakStatus {
            current_value: current_value.into(),
            is_default: current_value == DEFAULT_VALUE,
        })
    }
}
