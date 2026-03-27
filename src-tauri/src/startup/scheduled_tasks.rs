use std::path::Path;

use windows::Win32::Foundation::{RPC_E_CHANGED_MODE, VARIANT_BOOL};
use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
};
use windows::Win32::System::TaskScheduler::{
    IAction, IActionCollection, IExecAction, IRegisteredTask, ITaskDefinition, ITaskFolder,
    ITaskService, ITriggerCollection, TASK_ACTION_COM_HANDLER, TASK_ACTION_EXEC,
    TASK_ACTION_SEND_EMAIL, TASK_ACTION_SHOW_MESSAGE, TASK_ACTION_TYPE, TASK_ENUM_HIDDEN,
    TASK_TRIGGER_BOOT, TASK_TRIGGER_DAILY, TASK_TRIGGER_EVENT, TASK_TRIGGER_IDLE,
    TASK_TRIGGER_LOGON, TASK_TRIGGER_MONTHLY, TASK_TRIGGER_MONTHLYDOW, TASK_TRIGGER_REGISTRATION,
    TASK_TRIGGER_SESSION_STATE_CHANGE, TASK_TRIGGER_TIME, TASK_TRIGGER_TYPE2, TASK_TRIGGER_WEEKLY,
    TaskScheduler,
};
use windows::Win32::System::Variant::VARIANT;
use windows::core::{BSTR, Interface};

use crate::error::AppError;
use crate::startup::presentation::{
    scheduled_task_presentation, scheduled_task_presentation_light,
};
use crate::startup::types::{
    StartupEntry, StartupEntryDetails, StartupScope, StartupSource, StartupStatus,
};

pub fn list_entries() -> Result<Vec<StartupEntry>, AppError> {
    let session = TaskSchedulerSession::connect()?;
    let root = session.root_folder()?;
    let mut entries = Vec::new();
    collect_folder_entries(&root, &mut entries, false)?;
    Ok(entries)
}

pub fn entry(id: &str) -> Result<StartupEntry, AppError> {
    let full_path = task_full_path_from_id(id)?;
    let session = TaskSchedulerSession::connect()?;
    let task = session.get_task(&full_path)?;
    task_to_entry(&task, true)?.ok_or_else(|| {
        AppError::message(format!(
            "scheduled task is filtered from startup list: {full_path}"
        ))
    })
}

pub fn enable_entry(id: &str) -> Result<StartupEntry, AppError> {
    let full_path = task_full_path_from_id(id)?;
    let session = TaskSchedulerSession::connect()?;
    let task = session.get_task(&full_path)?;
    ensure_task_is_visible(&task, &full_path)?;
    unsafe {
        task.SetEnabled(VARIANT_BOOL(1))?;
    }
    task_to_entry(&task, true)?.ok_or_else(|| {
        AppError::message(format!(
            "scheduled task is filtered from startup list: {full_path}"
        ))
    })
}

pub fn disable_entry(id: &str) -> Result<StartupEntry, AppError> {
    let full_path = task_full_path_from_id(id)?;
    let session = TaskSchedulerSession::connect()?;
    let task = session.get_task(&full_path)?;
    ensure_task_is_visible(&task, &full_path)?;
    unsafe {
        task.SetEnabled(VARIANT_BOOL(0))?;
    }
    task_to_entry(&task, true)?.ok_or_else(|| {
        AppError::message(format!(
            "scheduled task is filtered from startup list: {full_path}"
        ))
    })
}

pub fn delete_entry(id: &str) -> Result<(), AppError> {
    let full_path = task_full_path_from_id(id)?;
    let session = TaskSchedulerSession::connect()?;
    let task = session.get_task(&full_path)?;
    ensure_task_is_visible(&task, &full_path)?;
    let (folder_path, task_name) = split_task_identity(&full_path);
    let folder = session.folder(&folder_path)?;
    unsafe {
        folder.DeleteTask(&BSTR::from(task_name), 0)?;
    }
    Ok(())
}

pub fn entry_details(id: &str) -> Result<StartupEntryDetails, AppError> {
    let full_path = task_full_path_from_id(id)?;
    let session = TaskSchedulerSession::connect()?;
    let task = session.get_task(&full_path)?;
    let entry = task_to_entry(&task, true)?.ok_or_else(|| {
        AppError::message(format!(
            "scheduled task is filtered from startup list: {full_path}"
        ))
    })?;
    let definition = unsafe { task.Definition()? };
    let metadata = TaskMetadata::from_definition(&definition)?;

    Ok(StartupEntryDetails {
        entry,
        registry_hive: None,
        registry_path: None,
        registry_value_name: None,
        startup_folder_path: None,
        startup_file_path: None,
        task_path: Some(full_path),
        task_author: metadata.author,
        task_description: metadata.description,
        task_triggers: metadata.triggers,
        task_actions: metadata
            .actions
            .into_iter()
            .map(|action| action.summary)
            .collect(),
        raw_xml_preview: None,
    })
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

    fn root_folder(&self) -> Result<ITaskFolder, AppError> {
        self.folder("\\")
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

#[derive(Clone)]
struct TaskActionInfo {
    summary: String,
    command: Option<String>,
    arguments: Option<String>,
    working_directory: Option<String>,
}

struct TaskListMetadata {
    author: Option<String>,
    first_action: Option<TaskActionInfo>,
}

struct TaskMetadata {
    author: Option<String>,
    description: Option<String>,
    actions: Vec<TaskActionInfo>,
    triggers: Vec<String>,
}

impl TaskMetadata {
    fn from_definition(definition: &ITaskDefinition) -> Result<Self, AppError> {
        let (author, description) = unsafe {
            let registration = definition.RegistrationInfo()?;
            (
                bstr_out(|value| registration.Author(value))?,
                bstr_out(|value| registration.Description(value))?,
            )
        };

        let actions = unsafe { collect_action_info(&definition.Actions()?)? };
        let triggers = unsafe { collect_trigger_info(&definition.Triggers()?)? };

        Ok(Self {
            author,
            description,
            actions,
            triggers,
        })
    }
}

impl TaskListMetadata {
    fn from_definition(definition: &ITaskDefinition) -> Result<Self, AppError> {
        let author = unsafe {
            let registration = definition.RegistrationInfo()?;
            bstr_out(|value| registration.Author(value))?
        };

        let first_action = unsafe { first_action_info(&definition.Actions()?)? };

        Ok(Self {
            author,
            first_action,
        })
    }
}

fn collect_folder_entries(
    folder: &ITaskFolder,
    entries: &mut Vec<StartupEntry>,
    enrich: bool,
) -> Result<(), AppError> {
    let tasks = unsafe { folder.GetTasks(TASK_ENUM_HIDDEN.0)? };
    push_tasks(&tasks, entries, enrich)?;

    let subfolders = unsafe { folder.GetFolders(0)? };
    let count = unsafe { subfolders.Count()? };
    for index in 1..=count {
        let child = unsafe { subfolders.get_Item(&VARIANT::from(index))? };
        collect_folder_entries(&child, entries, enrich)?;
    }

    Ok(())
}

fn push_tasks(
    tasks: &windows::Win32::System::TaskScheduler::IRegisteredTaskCollection,
    entries: &mut Vec<StartupEntry>,
    enrich: bool,
) -> Result<(), AppError> {
    let count = unsafe { tasks.Count()? };
    for index in 1..=count {
        let task = unsafe { tasks.get_Item(&VARIANT::from(index))? };
        if let Some(entry) = task_to_entry(&task, enrich)? {
            entries.push(entry);
        }
    }

    Ok(())
}

fn task_to_entry(task: &IRegisteredTask, enrich: bool) -> Result<Option<StartupEntry>, AppError> {
    let name = unsafe { optional_bstr(task.Name()?) }
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "Scheduled task".to_string());
    let full_path = unsafe { optional_bstr(task.Path()?) }
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| normalize_task_full_path("\\", &name));
    let enabled = unsafe { task.Enabled()?.as_bool() };

    let metadata = unsafe {
        match task.Definition() {
            Ok(definition) => {
                TaskListMetadata::from_definition(&definition).unwrap_or(TaskListMetadata {
                    author: None,
                    first_action: None,
                })
            }
            Err(_) => TaskListMetadata {
                author: None,
                first_action: None,
            },
        }
    };

    let TaskListMetadata {
        author,
        first_action,
    } = metadata;
    let command = first_action.as_ref().and_then(|action| {
        action
            .command
            .clone()
            .or_else(|| Some(action.summary.clone()))
    });
    let arguments = first_action
        .as_ref()
        .and_then(|action| action.arguments.clone());
    let working_directory = first_action
        .as_ref()
        .and_then(|action| action.working_directory.clone());
    let presentation = if enrich {
        scheduled_task_presentation(
            &name,
            command.as_deref(),
            arguments.as_deref(),
            working_directory.as_deref(),
        )
    } else {
        scheduled_task_presentation_light(
            &name,
            command.as_deref(),
            arguments.as_deref(),
            working_directory.as_deref(),
        )
    };
    let is_system = is_system_task(
        author.as_deref(),
        &full_path,
        command.as_deref(),
        presentation.target_path.as_deref(),
    );

    if is_system {
        return Ok(None);
    }

    Ok(Some(StartupEntry {
        id: format!("task:{full_path}"),
        name,
        display_name: presentation.display_name,
        source: StartupSource::ScheduledTask,
        scope: StartupScope::AllUsers,
        status: if enabled {
            StartupStatus::Enabled
        } else {
            StartupStatus::Disabled
        },
        command,
        target_path: presentation.target_path,
        arguments: presentation.arguments,
        working_directory: presentation.working_directory,
        location_label: full_path.clone(),
        source_display: "Scheduled Task".to_string(),
        run_once: false,
        publisher: presentation.publisher,
        icon_data_url: presentation.icon_data_url,
        registry_path: None,
        task_path: Some(full_path),
        last_error: None,
    }))
}

fn ensure_task_is_visible(task: &IRegisteredTask, full_path: &str) -> Result<(), AppError> {
    task_to_entry(task, false)?.map(|_| ()).ok_or_else(|| {
        AppError::message(format!(
            "scheduled task is filtered from startup list: {full_path}"
        ))
    })
}

fn is_system_task(
    author: Option<&str>,
    full_path: &str,
    command: Option<&str>,
    target_path: Option<&str>,
) -> bool {
    author
        .map(|value| value.to_ascii_lowercase().contains("microsoft"))
        .unwrap_or(false)
        || full_path.to_ascii_lowercase().starts_with(r"\microsoft\")
        || target_path
            .filter(|value| !is_shell_host_path(value))
            .map(is_system_command)
            .unwrap_or(false)
        || (target_path.is_none()
            && command
                .filter(|value| !is_shell_host_path(value))
                .map(is_system_command)
                .unwrap_or(false))
}

fn is_shell_host_path(path: &str) -> bool {
    const HOSTS: &[&str] = &[
        "powershell.exe",
        "pwsh.exe",
        "cmd.exe",
        "wscript.exe",
        "cscript.exe",
        "mshta.exe",
    ];

    Path::new(path)
        .file_name()
        .and_then(|value| value.to_str())
        .map(|value| HOSTS.contains(&value.to_ascii_lowercase().as_str()))
        .unwrap_or(false)
}

unsafe fn collect_action_info(
    actions: &IActionCollection,
) -> Result<Vec<TaskActionInfo>, AppError> {
    let count = count_out(|value| unsafe { actions.Count(value) })?;
    let mut result = Vec::new();

    for index in 1..=count {
        let action = unsafe { actions.get_Item(index)? };
        let action_type = task_action_type_out(|value| unsafe { action.Type(value) })?;
        result.push(unsafe { action_info(&action, action_type)? });
    }

    Ok(result)
}

unsafe fn first_action_info(
    actions: &IActionCollection,
) -> Result<Option<TaskActionInfo>, AppError> {
    let count = count_out(|value| unsafe { actions.Count(value) })?;
    if count <= 0 {
        return Ok(None);
    }

    let action = unsafe { actions.get_Item(1)? };
    let action_type = task_action_type_out(|value| unsafe { action.Type(value) })?;
    Ok(Some(unsafe { action_info(&action, action_type)? }))
}

unsafe fn action_info(
    action: &IAction,
    action_type: TASK_ACTION_TYPE,
) -> Result<TaskActionInfo, AppError> {
    if action_type == TASK_ACTION_EXEC {
        let exec: IExecAction = action.cast()?;
        let command = bstr_out(|value| unsafe { exec.Path(value) })?;
        let arguments = bstr_out(|value| unsafe { exec.Arguments(value) })?;
        let working_directory = bstr_out(|value| unsafe { exec.WorkingDirectory(value) })?;
        let summary = join_command(&command, &arguments).unwrap_or_else(|| "Exec".to_string());

        return Ok(TaskActionInfo {
            summary,
            command,
            arguments,
            working_directory,
        });
    }

    Ok(TaskActionInfo {
        summary: action_type_label(action_type).to_string(),
        command: None,
        arguments: None,
        working_directory: None,
    })
}

unsafe fn collect_trigger_info(triggers: &ITriggerCollection) -> Result<Vec<String>, AppError> {
    let count = count_out(|value| unsafe { triggers.Count(value) })?;
    let mut result = Vec::new();

    for index in 1..=count {
        let trigger = unsafe { triggers.get_Item(index)? };
        result.push(
            trigger_type_label(trigger_type_out(|value| unsafe { trigger.Type(value) })?)
                .to_string(),
        );
    }

    Ok(result)
}

fn normalize_task_full_path(task_path: &str, task_name: &str) -> String {
    if task_path.ends_with('\\') {
        format!("{task_path}{task_name}")
    } else {
        format!(r"{task_path}\{task_name}")
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

fn task_full_path_from_id(id: &str) -> Result<String, AppError> {
    id.strip_prefix("task:")
        .filter(|value| !value.is_empty() && value.starts_with('\\'))
        .map(|value| value.to_string())
        .ok_or_else(|| AppError::message(format!("invalid scheduled task id: {id}")))
}

fn optional_bstr(value: BSTR) -> Option<String> {
    let string = value.to_string();
    let trimmed = string.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn bstr_out(
    read: impl FnOnce(*mut BSTR) -> windows::core::Result<()>,
) -> Result<Option<String>, AppError> {
    let mut value = BSTR::new();
    read(&mut value)?;
    Ok(optional_bstr(value))
}

fn count_out(read: impl FnOnce(*mut i32) -> windows::core::Result<()>) -> Result<i32, AppError> {
    let mut value = 0;
    read(&mut value)?;
    Ok(value)
}

fn task_action_type_out(
    read: impl FnOnce(*mut TASK_ACTION_TYPE) -> windows::core::Result<()>,
) -> Result<TASK_ACTION_TYPE, AppError> {
    let mut value = TASK_ACTION_TYPE(0);
    read(&mut value)?;
    Ok(value)
}

fn trigger_type_out(
    read: impl FnOnce(*mut TASK_TRIGGER_TYPE2) -> windows::core::Result<()>,
) -> Result<TASK_TRIGGER_TYPE2, AppError> {
    let mut value = TASK_TRIGGER_TYPE2(0);
    read(&mut value)?;
    Ok(value)
}

fn join_command(command: &Option<String>, arguments: &Option<String>) -> Option<String> {
    match (command.as_deref(), arguments.as_deref()) {
        (Some(command), Some(arguments)) if !arguments.trim().is_empty() => {
            Some(format!("{command} {arguments}"))
        }
        (Some(command), _) => Some(command.to_string()),
        _ => None,
    }
}

fn action_type_label(action_type: TASK_ACTION_TYPE) -> &'static str {
    if action_type == TASK_ACTION_EXEC {
        "Exec"
    } else if action_type == TASK_ACTION_COM_HANDLER {
        "COM handler"
    } else if action_type == TASK_ACTION_SEND_EMAIL {
        "Send email"
    } else if action_type == TASK_ACTION_SHOW_MESSAGE {
        "Show message"
    } else {
        "Other action"
    }
}

fn trigger_type_label(trigger_type: TASK_TRIGGER_TYPE2) -> &'static str {
    if trigger_type == TASK_TRIGGER_EVENT {
        "Event"
    } else if trigger_type == TASK_TRIGGER_TIME {
        "Time"
    } else if trigger_type == TASK_TRIGGER_DAILY {
        "Daily"
    } else if trigger_type == TASK_TRIGGER_WEEKLY {
        "Weekly"
    } else if trigger_type == TASK_TRIGGER_MONTHLY {
        "Monthly"
    } else if trigger_type == TASK_TRIGGER_MONTHLYDOW {
        "Monthly DOW"
    } else if trigger_type == TASK_TRIGGER_IDLE {
        "Idle"
    } else if trigger_type == TASK_TRIGGER_REGISTRATION {
        "Registration"
    } else if trigger_type == TASK_TRIGGER_BOOT {
        "Boot"
    } else if trigger_type == TASK_TRIGGER_LOGON {
        "Logon"
    } else if trigger_type == TASK_TRIGGER_SESSION_STATE_CHANGE {
        "Session state"
    } else {
        "Other trigger"
    }
}

fn is_system_command(command: &str) -> bool {
    let normalized = command.trim().trim_matches('"').to_ascii_lowercase();
    normalized.contains(r"c:\windows\")
        || normalized.contains(r"%systemroot%")
        || normalized.contains(r"\windows\system32\")
}
