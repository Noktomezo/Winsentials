use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::tweaks::{
    RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakOption, TweakStatus,
};

const DEFAULT_VALUE: &str = "default";
const BALANCED_VALUE: &str = "balanced";
const FAST_VALUE: &str = "fast";
const ULTRA_FAST_VALUE: &str = "ultra_fast";
const CUSTOM_VALUE: &str = "custom";

const KEYBOARD_RESPONSE_KEY: RegKey = RegKey {
    hive: Hive::CurrentUser,
    path: r"Control Panel\Accessibility\Keyboard Response",
};

struct KeyboardRepeatPreset {
    auto_repeat_delay: &'static str,
    auto_repeat_rate: &'static str,
    delay_before_acceptance: &'static str,
    bounce_time: &'static str,
    flags: &'static str,
}

const DEFAULT_PRESET: KeyboardRepeatPreset = KeyboardRepeatPreset {
    auto_repeat_delay: "1000",
    auto_repeat_rate: "500",
    delay_before_acceptance: "1000",
    bounce_time: "0",
    flags: "126",
};

const BALANCED_PRESET: KeyboardRepeatPreset = KeyboardRepeatPreset {
    auto_repeat_delay: "300",
    auto_repeat_rate: "15",
    delay_before_acceptance: "0",
    bounce_time: "0",
    flags: "59",
};

const FAST_PRESET: KeyboardRepeatPreset = KeyboardRepeatPreset {
    auto_repeat_delay: "200",
    auto_repeat_rate: "8",
    delay_before_acceptance: "0",
    bounce_time: "0",
    flags: "59",
};

const ULTRA_FAST_PRESET: KeyboardRepeatPreset = KeyboardRepeatPreset {
    auto_repeat_delay: "150",
    auto_repeat_rate: "5",
    delay_before_acceptance: "0",
    bounce_time: "0",
    flags: "59",
};

struct KeyboardRepeatValues {
    auto_repeat_delay: String,
    auto_repeat_rate: String,
    delay_before_acceptance: String,
    bounce_time: String,
    flags: String,
}

pub struct FastKeyboardRepeatTweak {
    meta: TweakMeta,
}

impl Default for FastKeyboardRepeatTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl FastKeyboardRepeatTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "fast_keyboard_repeat".into(),
                category: "input".into(),
                name: "input.tweaks.fastKeyboardRepeat.name".into(),
                short_description: "input.tweaks.fastKeyboardRepeat.shortDescription".into(),
                detail_description: "input.tweaks.fastKeyboardRepeat.detailDescription".into(),
                control: TweakControlType::Dropdown {
                    options: vec![
                        TweakOption {
                            label: "input.tweaks.fastKeyboardRepeat.options.default".into(),
                            value: DEFAULT_VALUE.into(),
                        },
                        TweakOption {
                            label: "input.tweaks.fastKeyboardRepeat.options.balanced".into(),
                            value: BALANCED_VALUE.into(),
                        },
                        TweakOption {
                            label: "input.tweaks.fastKeyboardRepeat.options.fast".into(),
                            value: FAST_VALUE.into(),
                        },
                        TweakOption {
                            label: "input.tweaks.fastKeyboardRepeat.options.ultraFast".into(),
                            value: ULTRA_FAST_VALUE.into(),
                        },
                    ],
                },
                current_value: DEFAULT_VALUE.into(),
                default_value: DEFAULT_VALUE.into(),
                recommended_value: FAST_VALUE.into(),
                risk: RiskLevel::Low,
                risk_description: Some("input.tweaks.fastKeyboardRepeat.riskDescription".into()),
                conflicts: None,
                requires_action: RequiresAction::Logout,
                min_os_build: Some(10240),
                min_os_ubr: None,
            },
        }
    }

    fn preset(value: &str) -> Option<&'static KeyboardRepeatPreset> {
        match value {
            DEFAULT_VALUE => Some(&DEFAULT_PRESET),
            BALANCED_VALUE => Some(&BALANCED_PRESET),
            FAST_VALUE => Some(&FAST_PRESET),
            ULTRA_FAST_VALUE => Some(&ULTRA_FAST_PRESET),
            _ => None,
        }
    }

    fn apply_preset(&self, preset: &KeyboardRepeatPreset) -> Result<(), AppError> {
        KEYBOARD_RESPONSE_KEY.set_string("AutoRepeatDelay", preset.auto_repeat_delay)?;
        KEYBOARD_RESPONSE_KEY.set_string("AutoRepeatRate", preset.auto_repeat_rate)?;
        KEYBOARD_RESPONSE_KEY
            .set_string("DelayBeforeAcceptance", preset.delay_before_acceptance)?;
        KEYBOARD_RESPONSE_KEY.set_string("BounceTime", preset.bounce_time)?;
        KEYBOARD_RESPONSE_KEY.set_string("Flags", preset.flags)
    }

    fn read_string_or_default(name: &str, default: &str) -> Result<String, AppError> {
        match KEYBOARD_RESPONSE_KEY.get_string(name) {
            Ok(value) => Ok(value),
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
                Ok(default.to_string())
            }
            Err(error) => Err(error),
        }
    }

    fn read_current_values(&self) -> Result<KeyboardRepeatValues, AppError> {
        Ok(KeyboardRepeatValues {
            auto_repeat_delay: Self::read_string_or_default(
                "AutoRepeatDelay",
                DEFAULT_PRESET.auto_repeat_delay,
            )?,
            auto_repeat_rate: Self::read_string_or_default(
                "AutoRepeatRate",
                DEFAULT_PRESET.auto_repeat_rate,
            )?,
            delay_before_acceptance: Self::read_string_or_default(
                "DelayBeforeAcceptance",
                DEFAULT_PRESET.delay_before_acceptance,
            )?,
            bounce_time: Self::read_string_or_default("BounceTime", DEFAULT_PRESET.bounce_time)?,
            flags: Self::read_string_or_default("Flags", DEFAULT_PRESET.flags)?,
        })
    }

    fn matches_preset(values: &KeyboardRepeatValues, preset: &KeyboardRepeatPreset) -> bool {
        values.auto_repeat_delay == preset.auto_repeat_delay
            && values.auto_repeat_rate == preset.auto_repeat_rate
            && values.delay_before_acceptance == preset.delay_before_acceptance
            && values.bounce_time == preset.bounce_time
            && values.flags == preset.flags
    }

    fn current_preset(values: &KeyboardRepeatValues) -> &'static str {
        if Self::matches_preset(values, &DEFAULT_PRESET) {
            DEFAULT_VALUE
        } else if Self::matches_preset(values, &BALANCED_PRESET) {
            BALANCED_VALUE
        } else if Self::matches_preset(values, &FAST_PRESET) {
            FAST_VALUE
        } else if Self::matches_preset(values, &ULTRA_FAST_PRESET) {
            ULTRA_FAST_VALUE
        } else {
            CUSTOM_VALUE
        }
    }
}

impl Tweak for FastKeyboardRepeatTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match Self::preset(value) {
            Some(preset) => self.apply_preset(preset),
            None => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        self.apply_preset(&DEFAULT_PRESET)
    }

    fn get_status(&self) -> Result<TweakStatus, AppError> {
        let values = self.read_current_values()?;
        let current_value = Self::current_preset(&values);

        Ok(TweakStatus {
            current_value: current_value.into(),
            is_default: current_value == DEFAULT_VALUE,
        })
    }
}
