use std::process::Command;

use crate::error::AppError;
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";
const CUSTOM_VALUE: &str = "custom";

const USE_PLATFORM_CLOCK: &str = "useplatformclock";
const USE_PLATFORM_TICK: &str = "useplatformtick";
const DISABLE_DYNAMIC_TICK: &str = "disabledynamictick";

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

struct KernelTimingState {
    use_platform_clock: Option<String>,
    use_platform_tick: Option<String>,
    disable_dynamic_tick: Option<String>,
}

pub struct ConfigureKernelTimingChainTweak {
    meta: TweakMeta,
}

impl Default for ConfigureKernelTimingChainTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigureKernelTimingChainTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "configure_kernel_timing_chain".into(),
                category: "performance".into(),
                name: "performance.tweaks.configureKernelTimingChain.name".into(),
                short_description: "performance.tweaks.configureKernelTimingChain.shortDescription"
                    .into(),
                detail_description:
                    "performance.tweaks.configureKernelTimingChain.detailDescription".into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: ENABLED_VALUE.into(),
                risk: RiskLevel::Medium,
                risk_description: Some(
                    "performance.tweaks.configureKernelTimingChain.riskDescription".into(),
                ),
                conflicts: None,
                requires_action: RequiresAction::RestartPc,
                min_os_build: Some(10240),
                min_os_ubr: None,
                min_required_memory_gb: None,
            },
        }
    }

    fn run_bcdedit(args: &[&str]) -> Result<String, AppError> {
        let mut cmd = Command::new("bcdedit");
        cmd.args(args);

        #[cfg(target_os = "windows")]
        cmd.creation_flags(CREATE_NO_WINDOW);

        let output = cmd.output()?;

        if output.status.success() {
            return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
        }

        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let message = if stderr.is_empty() { stdout } else { stderr };

        Err(AppError::CommandFailed {
            command: format!("bcdedit {}", args.join(" ")),
            stderr: if message.is_empty() {
                "unknown error".into()
            } else {
                message
            },
        })
    }

    fn parse_boot_value(output: &str, key: &str) -> Option<String> {
        output.lines().find_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return None;
            }

            let mut parts = trimmed.split_whitespace();
            let candidate = parts.next()?;
            if !candidate.eq_ignore_ascii_case(key) {
                return None;
            }

            Some(parts.collect::<Vec<_>>().join(" ").to_ascii_lowercase())
        })
    }

    fn read_state(&self) -> Result<KernelTimingState, AppError> {
        let output = Self::run_bcdedit(&["/enum", "{current}"])?;

        Ok(KernelTimingState {
            use_platform_clock: Self::parse_boot_value(&output, USE_PLATFORM_CLOCK),
            use_platform_tick: Self::parse_boot_value(&output, USE_PLATFORM_TICK),
            disable_dynamic_tick: Self::parse_boot_value(&output, DISABLE_DYNAMIC_TICK),
        })
    }

    fn is_enabled(state: &KernelTimingState) -> bool {
        state.use_platform_clock.as_deref() == Some("no")
            && state.use_platform_tick.as_deref() == Some("yes")
            && state.disable_dynamic_tick.as_deref() == Some("yes")
    }

    fn is_default(state: &KernelTimingState) -> bool {
        state.use_platform_clock.is_none()
            && state.use_platform_tick.is_none()
            && state.disable_dynamic_tick.is_none()
    }

    fn delete_if_present(state_value: &Option<String>, key: &str) -> Result<(), AppError> {
        if state_value.is_some() {
            Self::run_bcdedit(&["/deletevalue", "{current}", key])?;
        }

        Ok(())
    }

    fn set_value(key: &str, value: &str) -> Result<(), AppError> {
        Self::run_bcdedit(&["/set", "{current}", key, value])?;
        Ok(())
    }

    fn restore_value(
        current_value: &Option<String>,
        original_value: &Option<String>,
        key: &str,
    ) -> Result<(), AppError> {
        match original_value {
            Some(value) => Self::set_value(key, value),
            None => Self::delete_if_present(current_value, key),
        }
    }

    fn apply_enabled_state() -> Result<(), AppError> {
        Self::set_value(USE_PLATFORM_CLOCK, "no")?;
        Self::set_value(USE_PLATFORM_TICK, "yes")?;
        Self::set_value(DISABLE_DYNAMIC_TICK, "yes")?;
        Ok(())
    }

    fn rollback_state(original_state: &KernelTimingState) -> Result<(), AppError> {
        let current_state = Self::read_current_state()?;
        let mut errors = Vec::new();

        if let Err(error) = Self::restore_value(
            &current_state.use_platform_clock,
            &original_state.use_platform_clock,
            USE_PLATFORM_CLOCK,
        ) {
            errors.push(format!("{USE_PLATFORM_CLOCK}: {error}"));
        }

        if let Err(error) = Self::restore_value(
            &current_state.use_platform_tick,
            &original_state.use_platform_tick,
            USE_PLATFORM_TICK,
        ) {
            errors.push(format!("{USE_PLATFORM_TICK}: {error}"));
        }

        if let Err(error) = Self::restore_value(
            &current_state.disable_dynamic_tick,
            &original_state.disable_dynamic_tick,
            DISABLE_DYNAMIC_TICK,
        ) {
            errors.push(format!("{DISABLE_DYNAMIC_TICK}: {error}"));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(AppError::message(format!(
                "rollback failed for {}",
                errors.join("; ")
            )))
        }
    }

    fn read_current_state() -> Result<KernelTimingState, AppError> {
        let output = Self::run_bcdedit(&["/enum", "{current}"])?;

        Ok(KernelTimingState {
            use_platform_clock: Self::parse_boot_value(&output, USE_PLATFORM_CLOCK),
            use_platform_tick: Self::parse_boot_value(&output, USE_PLATFORM_TICK),
            disable_dynamic_tick: Self::parse_boot_value(&output, DISABLE_DYNAMIC_TICK),
        })
    }
}

impl Tweak for ConfigureKernelTimingChainTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => {
                let original_state = self.read_state()?;

                match Self::apply_enabled_state() {
                    Ok(()) => Ok(()),
                    Err(apply_error) => match Self::rollback_state(&original_state) {
                        Ok(()) => Err(apply_error),
                        Err(rollback_error) => Err(AppError::message(format!(
                            "failed to apply {}: {apply_error}; rollback failed: {rollback_error}",
                            self.id()
                        ))),
                    },
                }
            }
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        let state = self.read_state()?;

        Self::delete_if_present(&state.use_platform_clock, USE_PLATFORM_CLOCK)?;
        Self::delete_if_present(&state.use_platform_tick, USE_PLATFORM_TICK)?;
        Self::delete_if_present(&state.disable_dynamic_tick, DISABLE_DYNAMIC_TICK)
    }

    fn get_status(&self) -> Result<TweakStatus, AppError> {
        let state = self.read_state()?;
        let is_enabled = Self::is_enabled(&state);
        let is_default = Self::is_default(&state);

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
