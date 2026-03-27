use crate::error::AppError;
use crate::startup;
use crate::startup::types::{StartupEntry, StartupEntryDetails, StartupSourceListResponse};

#[tauri::command]
pub async fn startup_list_registry() -> Result<StartupSourceListResponse, AppError> {
    tauri::async_runtime::spawn_blocking(startup::startup_list_registry)
        .await
        .map_err(|error| AppError::message(format!("startup_list_registry join error: {error}")))
}

#[tauri::command]
pub async fn startup_list_startup_folder() -> Result<StartupSourceListResponse, AppError> {
    tauri::async_runtime::spawn_blocking(startup::startup_list_startup_folder)
        .await
        .map_err(|error| {
            AppError::message(format!("startup_list_startup_folder join error: {error}"))
        })
}

#[tauri::command]
pub async fn startup_list_scheduled_tasks() -> Result<StartupSourceListResponse, AppError> {
    tauri::async_runtime::spawn_blocking(startup::startup_list_scheduled_tasks)
        .await
        .map_err(|error| {
            AppError::message(format!("startup_list_scheduled_tasks join error: {error}"))
        })
}

#[tauri::command]
pub async fn startup_hydrate_entries(ids: Vec<String>) -> Result<Vec<StartupEntry>, AppError> {
    tauri::async_runtime::spawn_blocking(move || startup::startup_hydrate_entries(&ids))
        .await
        .map_err(|error| {
            AppError::message(format!("startup_hydrate_entries join error: {error}"))
        })?
}

#[tauri::command]
pub async fn startup_enable(id: String) -> Result<StartupEntry, AppError> {
    tauri::async_runtime::spawn_blocking(move || startup::startup_enable(&id))
        .await
        .map_err(|error| AppError::message(format!("startup_enable join error: {error}")))?
}

#[tauri::command]
pub async fn startup_disable(id: String) -> Result<StartupEntry, AppError> {
    tauri::async_runtime::spawn_blocking(move || startup::startup_disable(&id))
        .await
        .map_err(|error| AppError::message(format!("startup_disable join error: {error}")))?
}

#[tauri::command]
pub async fn startup_delete(id: String) -> Result<(), AppError> {
    tauri::async_runtime::spawn_blocking(move || startup::startup_delete(&id))
        .await
        .map_err(|error| AppError::message(format!("startup_delete join error: {error}")))?
}

#[tauri::command]
pub async fn startup_details(id: String) -> Result<StartupEntryDetails, AppError> {
    tauri::async_runtime::spawn_blocking(move || startup::startup_details(&id))
        .await
        .map_err(|error| AppError::message(format!("startup_details join error: {error}")))?
}
