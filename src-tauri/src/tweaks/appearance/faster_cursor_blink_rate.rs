use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::shell::set_caret_blink_time;
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";
const FAST_CURSOR_BLINK_RATE: &str = "200";
const DEFAULT_CURSOR_BLINK_RATE: &str = "530";

const DESKTOP_KEY: RegKey = RegKey {
    hive: Hive::CurrentUser,
    path: r"Control Panel\Desktop",
};

pub struct FasterCursorBlinkRateTweak {
    meta: TweakMeta,
}

impl Default for FasterCursorBlinkRateTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl FasterCursorBlinkRateTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "faster_cursor_blink_rate".into(),
                category: "appearance".into(),
                name: "appearance.tweaks.fasterCursorBlinkRate.name".into(),
                short_description: "appearance.tweaks.fasterCursorBlinkRate.shortDescription"
                    .into(),
                detail_description: "appearance.tweaks.fasterCursorBlinkRate.detailDescription"
                    .into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: DISABLED_VALUE.into(),
                risk: RiskLevel::None,
                risk_description: None,
                conflicts: None,
                requires_action: RequiresAction::None,
                min_os_build: Some(10240),
                min_os_ubr: None,
            },
        }
    }

    fn is_enabled(&self) -> Result<bool, AppError> {
        match DESKTOP_KEY.get_string("CursorBlinkRate") {
            Ok(value) => Ok(value == FAST_CURSOR_BLINK_RATE),
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }
}

impl Tweak for FasterCursorBlinkRateTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => {
                DESKTOP_KEY.set_string("CursorBlinkRate", FAST_CURSOR_BLINK_RATE)?;
                set_caret_blink_time(200)
            }
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        DESKTOP_KEY.set_string("CursorBlinkRate", DEFAULT_CURSOR_BLINK_RATE)?;
        set_caret_blink_time(530)
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
