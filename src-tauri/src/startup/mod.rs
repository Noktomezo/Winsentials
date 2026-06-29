pub mod disabled_store;
pub mod presentation;
pub mod registry;
pub mod scheduled_tasks;
pub mod shell_hosts;
pub mod startup_folder;
pub mod types;

use rayon::prelude::*;

use crate::com::ComGuard;
use crate::error::AppError;
use crate::startup::types::{
    StartupEntry, StartupEntryDetails, StartupSource, StartupSourceListResponse,
};

pub fn startup_list_registry() -> StartupSourceListResponse {
    source_response(StartupSource::Registry, registry::list_entries())
}

pub fn startup_list_startup_folder() -> StartupSourceListResponse {
    source_response(StartupSource::StartupFolder, startup_folder::list_entries())
}

pub fn startup_list_scheduled_tasks() -> StartupSourceListResponse {
    source_response(
        StartupSource::ScheduledTask,
        scheduled_tasks::list_entries(),
    )
}

pub fn startup_hydrate_entries(ids: &[String]) -> Result<Vec<StartupEntry>, AppError> {
    let mut reg_ids: Vec<String> = Vec::new();
    let mut folder_ids: Vec<String> = Vec::new();
    let mut task_ids: Vec<String> = Vec::new();
    for id in ids {
        if id.starts_with("reg:") {
            reg_ids.push(id.clone());
        } else if id.starts_with("folder:") {
            folder_ids.push(id.clone());
        } else if id.starts_with("task:") {
            task_ids.push(id.clone());
        }
    }

    let groups: Vec<&[String]> = vec![&reg_ids, &folder_ids, &task_ids];
    let results: Vec<Vec<StartupEntry>> = groups
        .par_iter()
        .enumerate()
        .map(|(idx, group)| {
            let _com = ComGuard::new().ok();
            match idx {
                0 => registry::hydrate_entries(group).unwrap_or_default(),
                1 => startup_folder::hydrate_entries(group).unwrap_or_default(),
                2 => scheduled_tasks::hydrate_entries(group).unwrap_or_default(),
                _ => vec![],
            }
        })
        .collect();

    Ok(results.into_iter().flatten().collect())
}

pub fn startup_enable(id: &str) -> Result<StartupEntry, AppError> {
    if id.starts_with("reg:") {
        return registry::enable_entry(id);
    }

    if id.starts_with("folder:") {
        return startup_folder::enable_entry(id);
    }

    if id.starts_with("task:") {
        return scheduled_tasks::enable_entry(id);
    }

    Err(AppError::message(format!("unknown startup entry id: {id}")))
}

pub fn startup_disable(id: &str) -> Result<StartupEntry, AppError> {
    if id.starts_with("reg:") {
        return registry::disable_entry(id);
    }

    if id.starts_with("folder:") {
        return startup_folder::disable_entry(id);
    }

    if id.starts_with("task:") {
        return scheduled_tasks::disable_entry(id);
    }

    Err(AppError::message(format!("unknown startup entry id: {id}")))
}

pub fn startup_delete(id: &str) -> Result<(), AppError> {
    if id.starts_with("reg:") {
        return registry::delete_entry(id);
    }

    if id.starts_with("folder:") {
        return startup_folder::delete_entry(id);
    }

    if id.starts_with("task:") {
        return scheduled_tasks::delete_entry(id);
    }

    Err(AppError::message(format!("unknown startup entry id: {id}")))
}

pub fn startup_details(id: &str) -> Result<StartupEntryDetails, AppError> {
    if id.starts_with("reg:") {
        return registry::entry_details(id);
    }

    if id.starts_with("folder:") {
        return startup_folder::entry_details(id);
    }

    if id.starts_with("task:") {
        return scheduled_tasks::entry_details(id);
    }

    Err(AppError::message(format!("unknown startup entry id: {id}")))
}

fn source_response(
    source: StartupSource,
    result: Result<Vec<StartupEntry>, AppError>,
) -> StartupSourceListResponse {
    match result {
        Ok(entries) => StartupSourceListResponse {
            source,
            entries,
            error: None,
        },
        Err(error) => StartupSourceListResponse {
            source,
            entries: vec![],
            error: Some(error.to_string()),
        },
    }
}
