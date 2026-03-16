use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";
const LOCK_SCREEN_TIMEOUT_ATTRIBUTES_VISIBLE: u32 = 2;
const LOCK_SCREEN_TIMEOUT_ATTRIBUTES_HIDDEN: u32 = 1;

const LOCK_SCREEN_TIMEOUT_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SYSTEM\CurrentControlSet\Control\Power\PowerSettings\7516b95f-f776-4464-8c53-06167f40cc99\8ec4b3a5-6868-48c2-be75-4f3044be88a7",
};

pub struct UnlockLockScreenTimeoutSettingTweak {
    meta: TweakMeta,
}

impl Default for UnlockLockScreenTimeoutSettingTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl UnlockLockScreenTimeoutSettingTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "unlock_lock_screen_timeout_setting".into(),
                category: "behaviour".into(),
                name: "behaviour.tweaks.unlockLockScreenTimeoutSetting.name".into(),
                short_description:
                    "behaviour.tweaks.unlockLockScreenTimeoutSetting.shortDescription".into(),
                detail_description:
                    "behaviour.tweaks.unlockLockScreenTimeoutSetting.detailDescription".into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: ENABLED_VALUE.into(),
                risk: RiskLevel::None,
                risk_description: None,
                requires_action: RequiresAction::None,
                min_os_build: Some(9600),
                min_os_ubr: None,
            },
        }
    }

    fn is_enabled(&self) -> Result<bool, AppError> {
        match LOCK_SCREEN_TIMEOUT_KEY.get_dword("Attributes") {
            Ok(value) => Ok(value == LOCK_SCREEN_TIMEOUT_ATTRIBUTES_VISIBLE),
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }
}

impl Tweak for UnlockLockScreenTimeoutSettingTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => LOCK_SCREEN_TIMEOUT_KEY
                .set_dword("Attributes", LOCK_SCREEN_TIMEOUT_ATTRIBUTES_VISIBLE),
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        LOCK_SCREEN_TIMEOUT_KEY.set_dword("Attributes", LOCK_SCREEN_TIMEOUT_ATTRIBUTES_HIDDEN)
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
