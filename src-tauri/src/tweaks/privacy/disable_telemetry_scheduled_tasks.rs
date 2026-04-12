use crate::error::AppError;
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};
use windows::Win32::Foundation::{RPC_E_CHANGED_MODE, VARIANT_BOOL};
use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
};
use windows::Win32::System::TaskScheduler::{
    IRegisteredTask, ITaskFolder, ITaskService, TaskScheduler,
};
use windows::Win32::System::Variant::VARIANT;
use windows::core::{BSTR, Error as WindowsError};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";
const CUSTOM_VALUE: &str = "custom";

const MIN_WINDOWS_10_BUILD: u32 = 10240;
const TASK_SCHEDULER_NOT_FOUND_HRESULT: i32 = -2147216615;
const FILE_NOT_FOUND_HRESULT: i32 = -2147024894;

#[derive(Clone, Copy)]
struct ScheduledTaskDefinition {
    name: &'static str,
    path: &'static str,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum ScheduledTaskState {
    Disabled,
    Enabled,
    Missing,
}

const TELEMETRY_TASKS: &[ScheduledTaskDefinition] = &[
    ScheduledTaskDefinition {
        path: r"\Microsoft\Windows\Application Experience\",
        name: "Microsoft Compatibility Appraiser",
    },
    ScheduledTaskDefinition {
        path: r"\Microsoft\Windows\Application Experience\",
        name: "ProgramDataUpdater",
    },
    ScheduledTaskDefinition {
        path: r"\Microsoft\Windows\Customer Experience Improvement Program\",
        name: "Consolidator",
    },
    ScheduledTaskDefinition {
        path: r"\Microsoft\Windows\Customer Experience Improvement Program\",
        name: "UsbCeip",
    },
    ScheduledTaskDefinition {
        path: r"\Microsoft\Windows\Customer Experience Improvement Program\",
        name: "KernelCeipTask",
    },
    ScheduledTaskDefinition {
        path: r"\Microsoft\Windows\Customer Experience Improvement Program\",
        name: "Uploader",
    },
];

pub struct DisableTelemetryScheduledTasksTweak {
    meta: TweakMeta,
}

impl Default for DisableTelemetryScheduledTasksTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl DisableTelemetryScheduledTasksTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "disable_telemetry_scheduled_tasks".into(),
                category: "privacy".into(),
                name: "privacy.tweaks.disableTelemetryScheduledTasks.name".into(),
                short_description: "privacy.tweaks.disableTelemetryScheduledTasks.shortDescription"
                    .into(),
                detail_description:
                    "privacy.tweaks.disableTelemetryScheduledTasks.detailDescription".into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: ENABLED_VALUE.into(),
                risk: RiskLevel::Low,
                risk_description: Some(
                    "privacy.tweaks.disableTelemetryScheduledTasks.riskDescription".into(),
                ),
                conflicts: None,
                requires_action: RequiresAction::None,
                min_os_build: Some(MIN_WINDOWS_10_BUILD),
                min_os_ubr: None,
            },
        }
    }

    fn query_task_state(task: ScheduledTaskDefinition) -> Result<ScheduledTaskState, AppError> {
        let session = TaskSchedulerSession::connect()?;
        let registered_task = match session.get_task(&task.full_path()) {
            Ok(task) => task,
            Err(AppError::Windows(error)) if is_task_missing_error(&error) => {
                return Ok(ScheduledTaskState::Missing);
            }
            Err(error) => return Err(error),
        };

        let is_enabled = unsafe { registered_task.Enabled()?.as_bool() };

        Ok(if is_enabled {
            ScheduledTaskState::Enabled
        } else {
            ScheduledTaskState::Disabled
        })
    }

    fn set_task_enabled(task: ScheduledTaskDefinition, enabled: bool) -> Result<(), AppError> {
        let session = TaskSchedulerSession::connect()?;
        let registered_task = match session.get_task(&task.full_path()) {
            Ok(task) => task,
            Err(AppError::Windows(error)) if is_task_missing_error(&error) => return Ok(()),
            Err(error) => return Err(error),
        };

        unsafe {
            registered_task.SetEnabled(VARIANT_BOOL::from(enabled))?;
        }

        Ok(())
    }

    fn collect_states(&self) -> Result<Vec<ScheduledTaskState>, AppError> {
        TELEMETRY_TASKS
            .iter()
            .copied()
            .map(Self::query_task_state)
            .collect()
    }
}

impl Tweak for DisableTelemetryScheduledTasksTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => TELEMETRY_TASKS
                .iter()
                .copied()
                .try_for_each(|task| Self::set_task_enabled(task, false)),
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        TELEMETRY_TASKS
            .iter()
            .copied()
            .try_for_each(|task| Self::set_task_enabled(task, true))
    }

    fn get_status(&self) -> Result<TweakStatus, AppError> {
        let states = self.collect_states()?;
        let present_states: Vec<ScheduledTaskState> = states
            .into_iter()
            .filter(|state| *state != ScheduledTaskState::Missing)
            .collect();

        let is_enabled = !present_states.is_empty()
            && present_states
                .iter()
                .all(|state| *state == ScheduledTaskState::Disabled);
        let is_default = present_states
            .iter()
            .all(|state| *state == ScheduledTaskState::Enabled);

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

impl ScheduledTaskDefinition {
    fn full_path(self) -> String {
        format!("{}{}", self.path, self.name)
    }
}

struct TaskSchedulerSession {
    _com: ComGuard,
    service: ITaskService,
}

impl TaskSchedulerSession {
    fn connect() -> Result<Self, AppError> {
        let com = ComGuard::new()?;
        let service: ITaskService =
            unsafe { CoCreateInstance(&TaskScheduler, None, CLSCTX_INPROC_SERVER)? };
        unsafe {
            let empty = VARIANT::default();
            service.Connect(&empty, &empty, &empty, &empty)?;
        }

        Ok(Self { _com: com, service })
    }

    fn folder(&self, path: &str) -> Result<ITaskFolder, AppError> {
        unsafe {
            self.service
                .GetFolder(&BSTR::from(path))
                .map_err(AppError::from)
        }
    }

    fn get_task(&self, full_path: &str) -> Result<IRegisteredTask, AppError> {
        let (folder_path, task_name) = split_task_identity(full_path);
        let folder = self.folder(&folder_path)?;
        unsafe {
            folder
                .GetTask(&BSTR::from(task_name))
                .map_err(AppError::from)
        }
    }
}

struct ComGuard {
    owned: bool,
}

impl ComGuard {
    fn new() -> Result<Self, AppError> {
        let status = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        if status.is_ok() {
            return Ok(Self { owned: true });
        }

        if status == RPC_E_CHANGED_MODE {
            return Ok(Self { owned: false });
        }

        status.ok()?;
        unreachable!("successful COM initialization should have returned earlier")
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        if self.owned {
            unsafe {
                CoUninitialize();
            }
        }
    }
}

fn split_task_identity(full_path: &str) -> (String, String) {
    let index = full_path.rfind('\\').unwrap_or(0);
    let name = full_path[(index + 1)..].to_string();
    let task_path = if index == 0 {
        "\\".to_string()
    } else {
        full_path[..index].to_string()
    };
    (task_path, name)
}

fn is_task_missing_error(error: &WindowsError) -> bool {
    matches!(
        error.code().0,
        TASK_SCHEDULER_NOT_FOUND_HRESULT | FILE_NOT_FOUND_HRESULT
    )
}
