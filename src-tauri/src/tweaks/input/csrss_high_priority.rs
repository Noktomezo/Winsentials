use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";
const CUSTOM_VALUE: &str = "custom";

const DEFAULT_CPU_PRIORITY_CLASS: u32 = 2;
const DEFAULT_IO_PRIORITY: u32 = 2;

const TWEAK_CPU_PRIORITY_CLASS: u32 = 4;
const TWEAK_IO_PRIORITY: u32 = 3;
const TWEAK_NO_LAZY_MODE: u32 = 1;
const TWEAK_ALWAYS_ON: u32 = 1;

const MIN_WINDOWS_10_1809_BUILD: u32 = 17763;

const CSRSS_PERF_OPTIONS_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\csrss.exe\PerfOptions",
};

const SYSTEM_PROFILE_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile",
};

struct CsrssHighPriorityState {
    cpu_priority_class: u32,
    io_priority: u32,
    no_lazy_mode: Option<u32>,
    always_on: Option<u32>,
}

enum DwordSnapshot {
    Missing,
    Present(u32),
}

struct CsrssHighPrioritySnapshot {
    cpu_priority_class: DwordSnapshot,
    io_priority: DwordSnapshot,
    no_lazy_mode: DwordSnapshot,
    always_on: DwordSnapshot,
}

pub struct CsrssHighPriorityTweak {
    meta: TweakMeta,
}

impl Default for CsrssHighPriorityTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl CsrssHighPriorityTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "csrss_high_priority".into(),
                category: "input".into(),
                name: "input.tweaks.csrssHighPriority.name".into(),
                short_description: "input.tweaks.csrssHighPriority.shortDescription".into(),
                detail_description: "input.tweaks.csrssHighPriority.detailDescription".into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: ENABLED_VALUE.into(),
                risk: RiskLevel::Medium,
                risk_description: Some("input.tweaks.csrssHighPriority.riskDescription".into()),
                conflicts: None,
                requires_action: RequiresAction::RestartPc,
                min_os_build: Some(MIN_WINDOWS_10_1809_BUILD),
                min_os_ubr: None,
                min_required_memory_gb: None,
            },
        }
    }

    fn read_dword_or_default(key: &RegKey, name: &str, default: u32) -> Result<u32, AppError> {
        match key.get_dword(name) {
            Ok(value) => Ok(value),
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => Ok(default),
            Err(error) => Err(error),
        }
    }

    fn read_optional_dword(key: &RegKey, name: &str) -> Result<Option<u32>, AppError> {
        match key.get_dword(name) {
            Ok(value) => Ok(Some(value)),
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error),
        }
    }

    fn read_state(&self) -> Result<CsrssHighPriorityState, AppError> {
        Ok(CsrssHighPriorityState {
            cpu_priority_class: Self::read_dword_or_default(
                &CSRSS_PERF_OPTIONS_KEY,
                "CpuPriorityClass",
                DEFAULT_CPU_PRIORITY_CLASS,
            )?,
            io_priority: Self::read_dword_or_default(
                &CSRSS_PERF_OPTIONS_KEY,
                "IoPriority",
                DEFAULT_IO_PRIORITY,
            )?,
            no_lazy_mode: Self::read_optional_dword(&SYSTEM_PROFILE_KEY, "NoLazyMode")?,
            always_on: Self::read_optional_dword(&SYSTEM_PROFILE_KEY, "AlwaysOn")?,
        })
    }

    fn is_enabled(state: &CsrssHighPriorityState) -> bool {
        state.cpu_priority_class == TWEAK_CPU_PRIORITY_CLASS
            && state.io_priority == TWEAK_IO_PRIORITY
            && state.no_lazy_mode == Some(TWEAK_NO_LAZY_MODE)
            && state.always_on == Some(TWEAK_ALWAYS_ON)
    }

    fn is_default(state: &CsrssHighPriorityState) -> bool {
        state.cpu_priority_class == DEFAULT_CPU_PRIORITY_CLASS
            && state.io_priority == DEFAULT_IO_PRIORITY
            && state.no_lazy_mode.is_none()
            && state.always_on.is_none()
    }

    fn snapshot_dword(key: &RegKey, name: &str) -> Result<DwordSnapshot, AppError> {
        match key.get_dword(name) {
            Ok(value) => Ok(DwordSnapshot::Present(value)),
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
                Ok(DwordSnapshot::Missing)
            }
            Err(error) => Err(error),
        }
    }

    fn capture_snapshot() -> Result<CsrssHighPrioritySnapshot, AppError> {
        Ok(CsrssHighPrioritySnapshot {
            cpu_priority_class: Self::snapshot_dword(&CSRSS_PERF_OPTIONS_KEY, "CpuPriorityClass")?,
            io_priority: Self::snapshot_dword(&CSRSS_PERF_OPTIONS_KEY, "IoPriority")?,
            no_lazy_mode: Self::snapshot_dword(&SYSTEM_PROFILE_KEY, "NoLazyMode")?,
            always_on: Self::snapshot_dword(&SYSTEM_PROFILE_KEY, "AlwaysOn")?,
        })
    }

    fn restore_dword(key: &RegKey, name: &str, snapshot: &DwordSnapshot) -> Result<(), AppError> {
        match snapshot {
            DwordSnapshot::Missing => key.delete_value(name),
            DwordSnapshot::Present(value) => key.set_dword(name, *value),
        }
    }

    fn write_enabled_values() -> Result<(), AppError> {
        CSRSS_PERF_OPTIONS_KEY.set_dword("CpuPriorityClass", TWEAK_CPU_PRIORITY_CLASS)?;
        CSRSS_PERF_OPTIONS_KEY.set_dword("IoPriority", TWEAK_IO_PRIORITY)?;
        SYSTEM_PROFILE_KEY.set_dword("NoLazyMode", TWEAK_NO_LAZY_MODE)?;
        SYSTEM_PROFILE_KEY.set_dword("AlwaysOn", TWEAK_ALWAYS_ON)
    }

    fn write_default_values() -> Result<(), AppError> {
        CSRSS_PERF_OPTIONS_KEY.set_dword("CpuPriorityClass", DEFAULT_CPU_PRIORITY_CLASS)?;
        CSRSS_PERF_OPTIONS_KEY.set_dword("IoPriority", DEFAULT_IO_PRIORITY)?;
        SYSTEM_PROFILE_KEY.delete_value("NoLazyMode")?;
        SYSTEM_PROFILE_KEY.delete_value("AlwaysOn")
    }

    fn restore_snapshot(snapshot: &CsrssHighPrioritySnapshot) -> Result<(), AppError> {
        let mut errors = Vec::new();

        if let Err(error) = Self::restore_dword(
            &CSRSS_PERF_OPTIONS_KEY,
            "CpuPriorityClass",
            &snapshot.cpu_priority_class,
        ) {
            errors.push(format!("CpuPriorityClass: {error}"));
        }

        if let Err(error) =
            Self::restore_dword(&CSRSS_PERF_OPTIONS_KEY, "IoPriority", &snapshot.io_priority)
        {
            errors.push(format!("IoPriority: {error}"));
        }

        if let Err(error) =
            Self::restore_dword(&SYSTEM_PROFILE_KEY, "NoLazyMode", &snapshot.no_lazy_mode)
        {
            errors.push(format!("NoLazyMode: {error}"));
        }

        if let Err(error) =
            Self::restore_dword(&SYSTEM_PROFILE_KEY, "AlwaysOn", &snapshot.always_on)
        {
            errors.push(format!("AlwaysOn: {error}"));
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(AppError::message(format!(
                "rollback failed: {}",
                errors.join("; ")
            )))
        }
    }

    fn apply_with_rollback<F>(writer: F) -> Result<(), AppError>
    where
        F: FnOnce() -> Result<(), AppError>,
    {
        let snapshot = Self::capture_snapshot()?;

        match writer() {
            Ok(()) => Ok(()),
            Err(error) => match Self::restore_snapshot(&snapshot) {
                Ok(()) => Err(error),
                Err(rollback_error) => Err(AppError::message(format!("{error}; {rollback_error}"))),
            },
        }
    }
}

impl Tweak for CsrssHighPriorityTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => Self::apply_with_rollback(Self::write_enabled_values),
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        Self::apply_with_rollback(Self::write_default_values)
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
