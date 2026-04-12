use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";
const CUSTOM_VALUE: &str = "custom";

const DISABLED_MOUSE_SPEED: &str = "0";
const DISABLED_MOUSE_THRESHOLD_1: &str = "0";
const DISABLED_MOUSE_THRESHOLD_2: &str = "0";
const DISABLED_MOUSE_ACCEL: &str = "0";

const DEFAULT_MOUSE_SPEED: &str = "1";
const DEFAULT_MOUSE_THRESHOLD_1: &str = "6";
const DEFAULT_MOUSE_THRESHOLD_2: &str = "10";

const MOUSE_KEY: RegKey = RegKey {
    hive: Hive::CurrentUser,
    path: r"Control Panel\Mouse",
};

struct MouseSettings {
    mouse_accel: Option<String>,
    mouse_speed: String,
    mouse_threshold_1: String,
    mouse_threshold_2: String,
}

pub struct DisableMouseAccelerationTweak {
    meta: TweakMeta,
}

impl Default for DisableMouseAccelerationTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl DisableMouseAccelerationTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "disable_mouse_acceleration".into(),
                category: "input".into(),
                name: "input.tweaks.disableMouseAcceleration.name".into(),
                short_description: "input.tweaks.disableMouseAcceleration.shortDescription".into(),
                detail_description: "input.tweaks.disableMouseAcceleration.detailDescription"
                    .into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: ENABLED_VALUE.into(),
                risk: RiskLevel::Low,
                risk_description: Some(
                    "input.tweaks.disableMouseAcceleration.riskDescription".into(),
                ),
                conflicts: None,
                requires_action: RequiresAction::RestartPc,
                min_os_build: Some(10240),
                min_os_ubr: None,
                min_required_memory_gb: None,
            },
        }
    }

    fn read_string_or_default(name: &str, default: &str) -> Result<String, AppError> {
        match MOUSE_KEY.get_string(name) {
            Ok(value) => Ok(value),
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
                Ok(default.to_string())
            }
            Err(error) => Err(error),
        }
    }

    fn read_optional_string(name: &str) -> Result<Option<String>, AppError> {
        match MOUSE_KEY.get_string(name) {
            Ok(value) => Ok(Some(value)),
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error),
        }
    }

    fn read_settings(&self) -> Result<MouseSettings, AppError> {
        Ok(MouseSettings {
            mouse_accel: Self::read_optional_string("MouseAccel")?,
            mouse_speed: Self::read_string_or_default("MouseSpeed", DEFAULT_MOUSE_SPEED)?,
            mouse_threshold_1: Self::read_string_or_default(
                "MouseThreshold1",
                DEFAULT_MOUSE_THRESHOLD_1,
            )?,
            mouse_threshold_2: Self::read_string_or_default(
                "MouseThreshold2",
                DEFAULT_MOUSE_THRESHOLD_2,
            )?,
        })
    }

    fn is_enabled(settings: &MouseSettings) -> bool {
        settings.mouse_speed == DISABLED_MOUSE_SPEED
            && settings.mouse_threshold_1 == DISABLED_MOUSE_THRESHOLD_1
            && settings.mouse_threshold_2 == DISABLED_MOUSE_THRESHOLD_2
            && settings
                .mouse_accel
                .as_deref()
                .is_none_or(|value| value == DISABLED_MOUSE_ACCEL)
    }

    fn is_default(settings: &MouseSettings) -> bool {
        settings.mouse_speed == DEFAULT_MOUSE_SPEED
            && settings.mouse_threshold_1 == DEFAULT_MOUSE_THRESHOLD_1
            && settings.mouse_threshold_2 == DEFAULT_MOUSE_THRESHOLD_2
            && settings.mouse_accel.is_none()
    }
}

impl Tweak for DisableMouseAccelerationTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => {
                MOUSE_KEY.set_string("MouseSpeed", DISABLED_MOUSE_SPEED)?;
                MOUSE_KEY.set_string("MouseThreshold1", DISABLED_MOUSE_THRESHOLD_1)?;
                MOUSE_KEY.set_string("MouseThreshold2", DISABLED_MOUSE_THRESHOLD_2)?;
                MOUSE_KEY.set_string("MouseAccel", DISABLED_MOUSE_ACCEL)
            }
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        MOUSE_KEY.set_string("MouseSpeed", DEFAULT_MOUSE_SPEED)?;
        MOUSE_KEY.set_string("MouseThreshold1", DEFAULT_MOUSE_THRESHOLD_1)?;
        MOUSE_KEY.set_string("MouseThreshold2", DEFAULT_MOUSE_THRESHOLD_2)?;
        MOUSE_KEY.delete_value("MouseAccel")
    }

    fn get_status(&self) -> Result<TweakStatus, AppError> {
        let settings = self.read_settings()?;
        let is_enabled = Self::is_enabled(&settings);
        let is_default = Self::is_default(&settings);

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
