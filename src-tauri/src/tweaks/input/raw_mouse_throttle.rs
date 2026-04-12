use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";
const CUSTOM_VALUE: &str = "custom";

const DEFAULT_RAW_MOUSE_THROTTLE_ENABLED: u32 = 1;
const DEFAULT_RAW_MOUSE_THROTTLE_FORCED: u32 = 0;
const DEFAULT_RAW_MOUSE_THROTTLE_DURATION: u32 = 8;
const DEFAULT_RAW_MOUSE_THROTTLE_LEEWAY: u32 = 0;

const TWEAK_RAW_MOUSE_THROTTLE_ENABLED: u32 = 1;
const TWEAK_RAW_MOUSE_THROTTLE_FORCED: u32 = 1;
const TWEAK_RAW_MOUSE_THROTTLE_DURATION: u32 = 20;
const TWEAK_RAW_MOUSE_THROTTLE_LEEWAY: u32 = 0;

const MIN_WINDOWS_10_22H2_BUILD: u32 = 19045;

const MOUSE_KEY: RegKey = RegKey {
    hive: Hive::CurrentUser,
    path: r"Control Panel\Mouse",
};

struct RawMouseThrottleState {
    enabled: u32,
    forced: u32,
    duration: u32,
    leeway: u32,
}

enum DwordSnapshot {
    Missing,
    Present(u32),
}

pub struct RawMouseThrottleTweak {
    meta: TweakMeta,
}

impl Default for RawMouseThrottleTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl RawMouseThrottleTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "raw_mouse_throttle".into(),
                category: "performance".into(),
                name: "performance.tweaks.rawMouseThrottle.name".into(),
                short_description: "performance.tweaks.rawMouseThrottle.shortDescription".into(),
                detail_description: "performance.tweaks.rawMouseThrottle.detailDescription".into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: ENABLED_VALUE.into(),
                risk: RiskLevel::Medium,
                risk_description: Some(
                    "performance.tweaks.rawMouseThrottle.riskDescription".into(),
                ),
                conflicts: None,
                requires_action: RequiresAction::RestartPc,
                min_os_build: Some(MIN_WINDOWS_10_22H2_BUILD),
                min_os_ubr: None,
                min_required_memory_gb: None,
            },
        }
    }

    fn read_dword_or_default(name: &str, default: u32) -> Result<u32, AppError> {
        match MOUSE_KEY.get_dword(name) {
            Ok(value) => Ok(value),
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => Ok(default),
            Err(error) => Err(error),
        }
    }

    fn read_state(&self) -> Result<RawMouseThrottleState, AppError> {
        Ok(RawMouseThrottleState {
            enabled: Self::read_dword_or_default(
                "RawMouseThrottleEnabled",
                DEFAULT_RAW_MOUSE_THROTTLE_ENABLED,
            )?,
            forced: Self::read_dword_or_default(
                "RawMouseThrottleForced",
                DEFAULT_RAW_MOUSE_THROTTLE_FORCED,
            )?,
            duration: Self::read_dword_or_default(
                "RawMouseThrottleDuration",
                DEFAULT_RAW_MOUSE_THROTTLE_DURATION,
            )?,
            leeway: Self::read_dword_or_default(
                "RawMouseThrottleLeeway",
                DEFAULT_RAW_MOUSE_THROTTLE_LEEWAY,
            )?,
        })
    }

    fn is_enabled(state: &RawMouseThrottleState) -> bool {
        state.enabled == TWEAK_RAW_MOUSE_THROTTLE_ENABLED
            && state.forced == TWEAK_RAW_MOUSE_THROTTLE_FORCED
            && state.duration == TWEAK_RAW_MOUSE_THROTTLE_DURATION
            && state.leeway == TWEAK_RAW_MOUSE_THROTTLE_LEEWAY
    }

    fn is_default(state: &RawMouseThrottleState) -> bool {
        state.enabled == DEFAULT_RAW_MOUSE_THROTTLE_ENABLED
            && state.forced == DEFAULT_RAW_MOUSE_THROTTLE_FORCED
            && state.duration == DEFAULT_RAW_MOUSE_THROTTLE_DURATION
            && state.leeway == DEFAULT_RAW_MOUSE_THROTTLE_LEEWAY
    }

    fn snapshot_dword(name: &str) -> Result<DwordSnapshot, AppError> {
        match MOUSE_KEY.get_dword(name) {
            Ok(value) => Ok(DwordSnapshot::Present(value)),
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
                Ok(DwordSnapshot::Missing)
            }
            Err(error) => Err(error),
        }
    }

    fn restore_dword(name: &str, snapshot: &DwordSnapshot) -> Result<(), AppError> {
        match snapshot {
            DwordSnapshot::Missing => MOUSE_KEY.delete_value(name),
            DwordSnapshot::Present(value) => MOUSE_KEY.set_dword(name, *value),
        }
    }

    fn write_values_unchecked(
        enabled: u32,
        forced: u32,
        duration: u32,
        leeway: u32,
    ) -> Result<(), AppError> {
        MOUSE_KEY.set_dword("RawMouseThrottleEnabled", enabled)?;
        MOUSE_KEY.set_dword("RawMouseThrottleForced", forced)?;
        MOUSE_KEY.set_dword("RawMouseThrottleDuration", duration)?;
        MOUSE_KEY.set_dword("RawMouseThrottleLeeway", leeway)
    }

    fn write_values(enabled: u32, forced: u32, duration: u32, leeway: u32) -> Result<(), AppError> {
        let enabled_snapshot = Self::snapshot_dword("RawMouseThrottleEnabled")?;
        let forced_snapshot = Self::snapshot_dword("RawMouseThrottleForced")?;
        let duration_snapshot = Self::snapshot_dword("RawMouseThrottleDuration")?;
        let leeway_snapshot = Self::snapshot_dword("RawMouseThrottleLeeway")?;

        if let Err(error) = Self::write_values_unchecked(enabled, forced, duration, leeway) {
            let _ = Self::restore_dword("RawMouseThrottleEnabled", &enabled_snapshot);
            let _ = Self::restore_dword("RawMouseThrottleForced", &forced_snapshot);
            let _ = Self::restore_dword("RawMouseThrottleDuration", &duration_snapshot);
            let _ = Self::restore_dword("RawMouseThrottleLeeway", &leeway_snapshot);
            return Err(error);
        }

        Ok(())
    }
}

impl Tweak for RawMouseThrottleTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => Self::write_values(
                TWEAK_RAW_MOUSE_THROTTLE_ENABLED,
                TWEAK_RAW_MOUSE_THROTTLE_FORCED,
                TWEAK_RAW_MOUSE_THROTTLE_DURATION,
                TWEAK_RAW_MOUSE_THROTTLE_LEEWAY,
            ),
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        Self::write_values(
            DEFAULT_RAW_MOUSE_THROTTLE_ENABLED,
            DEFAULT_RAW_MOUSE_THROTTLE_FORCED,
            DEFAULT_RAW_MOUSE_THROTTLE_DURATION,
            DEFAULT_RAW_MOUSE_THROTTLE_LEEWAY,
        )
    }

    fn get_status(&self) -> Result<TweakStatus, AppError> {
        let state = self.read_state()?;
        let enabled = Self::is_enabled(&state);
        let is_default = Self::is_default(&state);

        Ok(TweakStatus {
            current_value: if enabled {
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
