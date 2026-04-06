use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};
use std::sync::{Mutex, OnceLock};

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

#[derive(Clone)]
enum DwordSnapshot {
    Missing,
    Present(u32),
}

#[derive(Clone)]
enum StringSnapshot {
    Missing,
    Present(String),
}

#[derive(Clone)]
struct MmcssSnapshot {
    system_responsiveness: DwordSnapshot,
    games_scheduling_category: StringSnapshot,
    games_priority: DwordSnapshot,
    games_gpu_priority: DwordSnapshot,
    games_sfio_priority: StringSnapshot,
    pro_audio_scheduling_category: StringSnapshot,
    pro_audio_priority: DwordSnapshot,
    pro_audio_gpu_priority: DwordSnapshot,
    pro_audio_sfio_priority: StringSnapshot,
}

struct MmcssValues {
    system_responsiveness: u32,
    games_scheduling_category: String,
    games_priority: u32,
    games_gpu_priority: u32,
    games_sfio_priority: String,
    pro_audio_scheduling_category: String,
    pro_audio_priority: u32,
    pro_audio_gpu_priority: u32,
    pro_audio_sfio_priority: String,
}

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
        let _lock = mmcss_operation_lock()
            .lock()
            .map_err(|_| AppError::message("failed to acquire MMCSS operation lock"))?;
        let snapshot = self.capture_snapshot()?;
        self.apply_with_rollback(snapshot, || {
            SYSTEM_PROFILE_KEY
                .set_dword("SystemResponsiveness", OPTIMIZED_SYSTEM_RESPONSIVENESS)?;

            GAMES_TASK_KEY
                .set_string("Scheduling Category", OPTIMIZED_GAMES_SCHEDULING_CATEGORY)?;
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
        })
    }

    fn set_default_values(&self) -> Result<(), AppError> {
        let _lock = mmcss_operation_lock()
            .lock()
            .map_err(|_| AppError::message("failed to acquire MMCSS operation lock"))?;
        let snapshot = self.capture_snapshot()?;
        self.apply_with_rollback(snapshot, || {
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
        })
    }

    fn capture_snapshot(&self) -> Result<MmcssSnapshot, AppError> {
        Ok(MmcssSnapshot {
            system_responsiveness: Self::snapshot_dword(
                &SYSTEM_PROFILE_KEY,
                "SystemResponsiveness",
            )?,
            games_scheduling_category: Self::snapshot_string(
                &GAMES_TASK_KEY,
                "Scheduling Category",
            )?,
            games_priority: Self::snapshot_dword(&GAMES_TASK_KEY, "Priority")?,
            games_gpu_priority: Self::snapshot_dword(&GAMES_TASK_KEY, "GPU Priority")?,
            games_sfio_priority: Self::snapshot_string(&GAMES_TASK_KEY, "SFIO Priority")?,
            pro_audio_scheduling_category: Self::snapshot_string(
                &PRO_AUDIO_TASK_KEY,
                "Scheduling Category",
            )?,
            pro_audio_priority: Self::snapshot_dword(&PRO_AUDIO_TASK_KEY, "Priority")?,
            pro_audio_gpu_priority: Self::snapshot_dword(&PRO_AUDIO_TASK_KEY, "GPU Priority")?,
            pro_audio_sfio_priority: Self::snapshot_string(&PRO_AUDIO_TASK_KEY, "SFIO Priority")?,
        })
    }

    fn apply_with_rollback<F>(&self, snapshot: MmcssSnapshot, writer: F) -> Result<(), AppError>
    where
        F: FnOnce() -> Result<(), AppError>,
    {
        match writer() {
            Ok(()) => Ok(()),
            Err(error) => {
                let rollback_errors = Self::restore_snapshot(&snapshot);

                if rollback_errors.is_empty() {
                    Err(error)
                } else {
                    Err(AppError::message(format!(
                        "{error}; rollback failed: {}",
                        rollback_errors.join("; ")
                    )))
                }
            }
        }
    }

    fn restore_snapshot(snapshot: &MmcssSnapshot) -> Vec<String> {
        let mut errors = Vec::new();

        Self::push_restore_error(
            &mut errors,
            Self::restore_dword(
                &SYSTEM_PROFILE_KEY,
                "SystemResponsiveness",
                &snapshot.system_responsiveness,
            ),
        );
        Self::push_restore_error(
            &mut errors,
            Self::restore_string(
                &GAMES_TASK_KEY,
                "Scheduling Category",
                &snapshot.games_scheduling_category,
            ),
        );
        Self::push_restore_error(
            &mut errors,
            Self::restore_dword(&GAMES_TASK_KEY, "Priority", &snapshot.games_priority),
        );
        Self::push_restore_error(
            &mut errors,
            Self::restore_dword(
                &GAMES_TASK_KEY,
                "GPU Priority",
                &snapshot.games_gpu_priority,
            ),
        );
        Self::push_restore_error(
            &mut errors,
            Self::restore_string(
                &GAMES_TASK_KEY,
                "SFIO Priority",
                &snapshot.games_sfio_priority,
            ),
        );
        Self::push_restore_error(
            &mut errors,
            Self::restore_string(
                &PRO_AUDIO_TASK_KEY,
                "Scheduling Category",
                &snapshot.pro_audio_scheduling_category,
            ),
        );
        Self::push_restore_error(
            &mut errors,
            Self::restore_dword(
                &PRO_AUDIO_TASK_KEY,
                "Priority",
                &snapshot.pro_audio_priority,
            ),
        );
        Self::push_restore_error(
            &mut errors,
            Self::restore_dword(
                &PRO_AUDIO_TASK_KEY,
                "GPU Priority",
                &snapshot.pro_audio_gpu_priority,
            ),
        );
        Self::push_restore_error(
            &mut errors,
            Self::restore_string(
                &PRO_AUDIO_TASK_KEY,
                "SFIO Priority",
                &snapshot.pro_audio_sfio_priority,
            ),
        );

        errors
    }

    fn push_restore_error(errors: &mut Vec<String>, result: Result<(), AppError>) {
        if let Err(error) = result {
            errors.push(error.to_string());
        }
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

    fn snapshot_string(key: &RegKey, name: &str) -> Result<StringSnapshot, AppError> {
        match key.get_string(name) {
            Ok(value) => Ok(StringSnapshot::Present(value)),
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
                Ok(StringSnapshot::Missing)
            }
            Err(error) => Err(error),
        }
    }

    fn restore_dword(key: &RegKey, name: &str, snapshot: &DwordSnapshot) -> Result<(), AppError> {
        match snapshot {
            DwordSnapshot::Missing => key.delete_value(name),
            DwordSnapshot::Present(value) => key.set_dword(name, *value),
        }
    }

    fn restore_string(key: &RegKey, name: &str, snapshot: &StringSnapshot) -> Result<(), AppError> {
        match snapshot {
            StringSnapshot::Missing => key.delete_value(name),
            StringSnapshot::Present(value) => key.set_string(name, value),
        }
    }

    fn read_current_values(&self) -> Result<MmcssValues, AppError> {
        Ok(MmcssValues {
            system_responsiveness: Self::read_dword_or_default(
                &SYSTEM_PROFILE_KEY,
                "SystemResponsiveness",
                DEFAULT_SYSTEM_RESPONSIVENESS,
            )?,
            games_scheduling_category: Self::read_string_or_default(
                &GAMES_TASK_KEY,
                "Scheduling Category",
                DEFAULT_GAMES_SCHEDULING_CATEGORY,
            )?,
            games_priority: Self::read_dword_or_default(
                &GAMES_TASK_KEY,
                "Priority",
                DEFAULT_GAMES_PRIORITY,
            )?,
            games_gpu_priority: Self::read_dword_or_default(
                &GAMES_TASK_KEY,
                "GPU Priority",
                DEFAULT_GAMES_GPU_PRIORITY,
            )?,
            games_sfio_priority: Self::read_string_or_default(
                &GAMES_TASK_KEY,
                "SFIO Priority",
                DEFAULT_GAMES_SFIO_PRIORITY,
            )?,
            pro_audio_scheduling_category: Self::read_string_or_default(
                &PRO_AUDIO_TASK_KEY,
                "Scheduling Category",
                DEFAULT_PRO_AUDIO_SCHEDULING_CATEGORY,
            )?,
            pro_audio_priority: Self::read_dword_or_default(
                &PRO_AUDIO_TASK_KEY,
                "Priority",
                DEFAULT_PRO_AUDIO_PRIORITY,
            )?,
            pro_audio_gpu_priority: Self::read_dword_or_default(
                &PRO_AUDIO_TASK_KEY,
                "GPU Priority",
                DEFAULT_PRO_AUDIO_GPU_PRIORITY,
            )?,
            pro_audio_sfio_priority: Self::read_string_or_default(
                &PRO_AUDIO_TASK_KEY,
                "SFIO Priority",
                DEFAULT_PRO_AUDIO_SFIO_PRIORITY,
            )?,
        })
    }

    fn is_default_values(values: &MmcssValues) -> bool {
        values.system_responsiveness == DEFAULT_SYSTEM_RESPONSIVENESS
            && values.games_scheduling_category == DEFAULT_GAMES_SCHEDULING_CATEGORY
            && values.games_priority == DEFAULT_GAMES_PRIORITY
            && values.games_gpu_priority == DEFAULT_GAMES_GPU_PRIORITY
            && values.games_sfio_priority == DEFAULT_GAMES_SFIO_PRIORITY
            && values.pro_audio_scheduling_category == DEFAULT_PRO_AUDIO_SCHEDULING_CATEGORY
            && values.pro_audio_priority == DEFAULT_PRO_AUDIO_PRIORITY
            && values.pro_audio_gpu_priority == DEFAULT_PRO_AUDIO_GPU_PRIORITY
            && values.pro_audio_sfio_priority == DEFAULT_PRO_AUDIO_SFIO_PRIORITY
    }

    fn is_optimized_values(values: &MmcssValues) -> bool {
        values.system_responsiveness == OPTIMIZED_SYSTEM_RESPONSIVENESS
            && values.games_scheduling_category == OPTIMIZED_GAMES_SCHEDULING_CATEGORY
            && values.games_priority == OPTIMIZED_GAMES_PRIORITY
            && values.games_gpu_priority == OPTIMIZED_GAMES_GPU_PRIORITY
            && values.games_sfio_priority == OPTIMIZED_GAMES_SFIO_PRIORITY
            && values.pro_audio_scheduling_category == OPTIMIZED_PRO_AUDIO_SCHEDULING_CATEGORY
            && values.pro_audio_priority == OPTIMIZED_PRO_AUDIO_PRIORITY
            && values.pro_audio_gpu_priority == OPTIMIZED_PRO_AUDIO_GPU_PRIORITY
            && values.pro_audio_sfio_priority == OPTIMIZED_PRO_AUDIO_SFIO_PRIORITY
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
}

fn mmcss_operation_lock() -> &'static Mutex<()> {
    static MMCSS_OPERATION_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    MMCSS_OPERATION_LOCK.get_or_init(|| Mutex::new(()))
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
        let _lock = mmcss_operation_lock()
            .lock()
            .map_err(|_| AppError::message("failed to acquire MMCSS operation lock"))?;
        let values = self.read_current_values()?;
        let enabled = Self::is_optimized_values(&values);
        let is_default = Self::is_default_values(&values);

        Ok(TweakStatus {
            current_value: if enabled {
                ENABLED_VALUE.into()
            } else if is_default {
                DISABLED_VALUE.into()
            } else {
                "custom".into()
            },
            is_default,
        })
    }
}
