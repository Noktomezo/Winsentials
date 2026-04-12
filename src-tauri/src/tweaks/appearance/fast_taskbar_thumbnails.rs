use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::shell::set_mouse_hover_time;
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";
const FAST_HOVER_TIME: &str = "100";
const DEFAULT_HOVER_TIME: &str = "400";

const MOUSE_KEY: RegKey = RegKey {
    hive: Hive::CurrentUser,
    path: r"Control Panel\Mouse",
};

pub struct FastTaskbarThumbnailsTweak {
    meta: TweakMeta,
}

impl Default for FastTaskbarThumbnailsTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl FastTaskbarThumbnailsTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "fast_taskbar_thumbnails".into(),
                category: "appearance".into(),
                name: "appearance.tweaks.fastTaskbarThumbnails.name".into(),
                short_description: "appearance.tweaks.fastTaskbarThumbnails.shortDescription"
                    .into(),
                detail_description: "appearance.tweaks.fastTaskbarThumbnails.detailDescription"
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
                min_required_memory_gb: None,
            },
        }
    }

    fn is_enabled(&self) -> Result<bool, AppError> {
        match MOUSE_KEY.get_string("MouseHoverTime") {
            Ok(value) => Ok(value == FAST_HOVER_TIME),
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }
}

impl Tweak for FastTaskbarThumbnailsTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => {
                MOUSE_KEY.set_string("MouseHoverTime", FAST_HOVER_TIME)?;
                set_mouse_hover_time(100)
            }
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        MOUSE_KEY.set_string("MouseHoverTime", DEFAULT_HOVER_TIME)?;
        set_mouse_hover_time(400)
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
