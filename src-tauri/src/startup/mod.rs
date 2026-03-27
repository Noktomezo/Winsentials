pub mod disabled_store;
pub mod presentation;
pub mod registry;
pub mod scheduled_tasks;
pub mod startup_folder;
pub mod types;

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
    ids.iter().map(|id| startup_entry(id)).collect()
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

fn startup_entry(id: &str) -> Result<StartupEntry, AppError> {
    if id.starts_with("reg:") {
        return registry::entry(id);
    }

    if id.starts_with("folder:") {
        return startup_folder::entry(id);
    }

    if id.starts_with("task:") {
        return scheduled_tasks::entry(id);
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
