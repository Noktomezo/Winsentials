use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";

const DEFAULT_SYSTEM_RESPONSIVENESS: u32 = 20;
const OPTIMIZED_SYSTEM_RESPONSIVENESS: u32 = 0;

const DEFAULT_GAMES_SCHEDULING_CATEGORY: &str = "Medium";
const OPTIMIZED_GAMES_SCHEDULING_CATEGORY: &str = "High";
const DEFAULT_GAMES_PRIORITY: u32 = 2;
const OPTIMIZED_GAMES_PRIORITY: u32 = 6;
const DEFAULT_GAMES_GPU_PRIORITY: u32 = 8;
const OPTIMIZED_GAMES_GPU_PRIORITY: u32 = 8;
const DEFAULT_GAMES_SFIO_PRIORITY: &str = "Normal";
const OPTIMIZED_GAMES_SFIO_PRIORITY: &str = "High";

const DEFAULT_PRO_AUDIO_SCHEDULING_CATEGORY: &str = "High";
const OPTIMIZED_PRO_AUDIO_SCHEDULING_CATEGORY: &str = "High";
const DEFAULT_PRO_AUDIO_PRIORITY: u32 = 2;
const OPTIMIZED_PRO_AUDIO_PRIORITY: u32 = 2;
const DEFAULT_PRO_AUDIO_GPU_PRIORITY: u32 = 8;
const OPTIMIZED_PRO_AUDIO_GPU_PRIORITY: u32 = 8;
const DEFAULT_PRO_AUDIO_SFIO_PRIORITY: &str = "Normal";
const OPTIMIZED_PRO_AUDIO_SFIO_PRIORITY: &str = "High";

const SYSTEM_PROFILE_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile",
};

const GAMES_TASK_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile\Tasks\Games",
};

const PRO_AUDIO_TASK_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile\Tasks\Pro Audio",
};

pub struct OptimizeMmcssTweak {
    meta: TweakMeta,
}

impl Default for OptimizeMmcssTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl OptimizeMmcssTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "optimize_mmcss".into(),
                category: "performance".into(),
                name: "performance.tweaks.optimizeMmcss.name".into(),
                short_description: "performance.tweaks.optimizeMmcss.shortDescription".into(),
                detail_description: "performance.tweaks.optimizeMmcss.detailDescription".into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: ENABLED_VALUE.into(),
                risk: RiskLevel::Low,
                risk_description: Some("performance.tweaks.optimizeMmcss.riskDescription".into()),
                conflicts: None,
                requires_action: RequiresAction::RestartPc,
                min_os_build: Some(10240),
                min_os_ubr: None,
            },
        }
    }

    fn set_optimized_values(&self) -> Result<(), AppError> {
        SYSTEM_PROFILE_KEY.set_dword("SystemResponsiveness", OPTIMIZED_SYSTEM_RESPONSIVENESS)?;

        GAMES_TASK_KEY.set_string("Scheduling Category", OPTIMIZED_GAMES_SCHEDULING_CATEGORY)?;
        GAMES_TASK_KEY.set_dword("Priority", OPTIMIZED_GAMES_PRIORITY)?;
        GAMES_TASK_KEY.set_dword("GPU Priority", OPTIMIZED_GAMES_GPU_PRIORITY)?;
        GAMES_TASK_KEY.set_string("SFIO Priority", OPTIMIZED_GAMES_SFIO_PRIORITY)?;

        PRO_AUDIO_TASK_KEY.set_string(
            "Scheduling Category",
            OPTIMIZED_PRO_AUDIO_SCHEDULING_CATEGORY,
        )?;
        PRO_AUDIO_TASK_KEY.set_dword("Priority", OPTIMIZED_PRO_AUDIO_PRIORITY)?;
        PRO_AUDIO_TASK_KEY.set_dword("GPU Priority", OPTIMIZED_PRO_AUDIO_GPU_PRIORITY)?;
        PRO_AUDIO_TASK_KEY.set_string("SFIO Priority", OPTIMIZED_PRO_AUDIO_SFIO_PRIORITY)?;

        Ok(())
    }

    fn set_default_values(&self) -> Result<(), AppError> {
        SYSTEM_PROFILE_KEY.set_dword("SystemResponsiveness", DEFAULT_SYSTEM_RESPONSIVENESS)?;

        GAMES_TASK_KEY.set_string("Scheduling Category", DEFAULT_GAMES_SCHEDULING_CATEGORY)?;
        GAMES_TASK_KEY.set_dword("Priority", DEFAULT_GAMES_PRIORITY)?;
        GAMES_TASK_KEY.set_dword("GPU Priority", DEFAULT_GAMES_GPU_PRIORITY)?;
        GAMES_TASK_KEY.set_string("SFIO Priority", DEFAULT_GAMES_SFIO_PRIORITY)?;

        PRO_AUDIO_TASK_KEY
            .set_string("Scheduling Category", DEFAULT_PRO_AUDIO_SCHEDULING_CATEGORY)?;
        PRO_AUDIO_TASK_KEY.set_dword("Priority", DEFAULT_PRO_AUDIO_PRIORITY)?;
        PRO_AUDIO_TASK_KEY.set_dword("GPU Priority", DEFAULT_PRO_AUDIO_GPU_PRIORITY)?;
        PRO_AUDIO_TASK_KEY.set_string("SFIO Priority", DEFAULT_PRO_AUDIO_SFIO_PRIORITY)?;

        Ok(())
    }

    fn read_dword_or_default(key: &RegKey, name: &str, default: u32) -> Result<u32, AppError> {
        match key.get_dword(name) {
            Ok(value) => Ok(value),
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => Ok(default),
            Err(error) => Err(error),
        }
    }

    fn read_string_or_default(key: &RegKey, name: &str, default: &str) -> Result<String, AppError> {
        match key.get_string(name) {
            Ok(value) => Ok(value),
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
                Ok(default.to_string())
            }
            Err(error) => Err(error),
        }
    }

    fn is_enabled(&self) -> Result<bool, AppError> {
        let system_responsiveness = Self::read_dword_or_default(
            &SYSTEM_PROFILE_KEY,
            "SystemResponsiveness",
            DEFAULT_SYSTEM_RESPONSIVENESS,
        )?;
        let games_scheduling_category = Self::read_string_or_default(
            &GAMES_TASK_KEY,
            "Scheduling Category",
            DEFAULT_GAMES_SCHEDULING_CATEGORY,
        )?;
        let games_priority =
            Self::read_dword_or_default(&GAMES_TASK_KEY, "Priority", DEFAULT_GAMES_PRIORITY)?;
        let games_gpu_priority = Self::read_dword_or_default(
            &GAMES_TASK_KEY,
            "GPU Priority",
            DEFAULT_GAMES_GPU_PRIORITY,
        )?;
        let games_sfio_priority = Self::read_string_or_default(
            &GAMES_TASK_KEY,
            "SFIO Priority",
            DEFAULT_GAMES_SFIO_PRIORITY,
        )?;
        let pro_audio_scheduling_category = Self::read_string_or_default(
            &PRO_AUDIO_TASK_KEY,
            "Scheduling Category",
            DEFAULT_PRO_AUDIO_SCHEDULING_CATEGORY,
        )?;
        let pro_audio_priority = Self::read_dword_or_default(
            &PRO_AUDIO_TASK_KEY,
            "Priority",
            DEFAULT_PRO_AUDIO_PRIORITY,
        )?;
        let pro_audio_gpu_priority = Self::read_dword_or_default(
            &PRO_AUDIO_TASK_KEY,
            "GPU Priority",
            DEFAULT_PRO_AUDIO_GPU_PRIORITY,
        )?;
        let pro_audio_sfio_priority = Self::read_string_or_default(
            &PRO_AUDIO_TASK_KEY,
            "SFIO Priority",
            DEFAULT_PRO_AUDIO_SFIO_PRIORITY,
        )?;

        Ok(system_responsiveness == OPTIMIZED_SYSTEM_RESPONSIVENESS
            && games_scheduling_category == OPTIMIZED_GAMES_SCHEDULING_CATEGORY
            && games_priority == OPTIMIZED_GAMES_PRIORITY
            && games_gpu_priority == OPTIMIZED_GAMES_GPU_PRIORITY
            && games_sfio_priority == OPTIMIZED_GAMES_SFIO_PRIORITY
            && pro_audio_scheduling_category == OPTIMIZED_PRO_AUDIO_SCHEDULING_CATEGORY
            && pro_audio_priority == OPTIMIZED_PRO_AUDIO_PRIORITY
            && pro_audio_gpu_priority == OPTIMIZED_PRO_AUDIO_GPU_PRIORITY
            && pro_audio_sfio_priority == OPTIMIZED_PRO_AUDIO_SFIO_PRIORITY)
    }
}

impl Tweak for OptimizeMmcssTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => self.set_optimized_values(),
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        self.set_default_values()
    }

    fn get_status(&self) -> Result<TweakStatus, AppError> {
        let enabled = self.is_enabled()?;

        Ok(TweakStatus {
            current_value: if enabled {
                ENABLED_VALUE.into()
            } else {
                DISABLED_VALUE.into()
            },
            is_default: !enabled,
        })
    }
}
