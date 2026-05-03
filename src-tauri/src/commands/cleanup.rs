use crate::cleanup;
use crate::cleanup::types::{CleanupAccessReport, CleanupCategoryReport, CleanupScheduleReport};
use crate::error::AppError;

#[tauri::command]
pub async fn cleanup_scan_category(category_id: String) -> Result<CleanupCategoryReport, AppError> {
    tauri::async_runtime::spawn_blocking(move || cleanup::cleanup_scan_category(&category_id))
        .await
        .map_err(|error| AppError::message(format!("cleanup_scan_category join error: {error}")))?
}

#[tauri::command]
pub async fn cleanup_clean_category(
    category_id: String,
) -> Result<CleanupCategoryReport, AppError> {
    tauri::async_runtime::spawn_blocking(move || cleanup::cleanup_clean_category(&category_id))
        .await
        .map_err(|error| AppError::message(format!("cleanup_clean_category join error: {error}")))?
}

#[tauri::command]
pub async fn cleanup_prepare_access() -> Result<CleanupAccessReport, AppError> {
    tauri::async_runtime::spawn_blocking(cleanup::cleanup_prepare_access)
        .await
        .map_err(|error| AppError::message(format!("cleanup_prepare_access join error: {error}")))?
}

#[tauri::command]
pub async fn cleanup_schedule_delete_on_reboot(
    paths: Vec<String>,
) -> Result<CleanupScheduleReport, AppError> {
    tauri::async_runtime::spawn_blocking(move || cleanup::cleanup_schedule_delete_on_reboot(&paths))
        .await
        .map_err(|error| {
            AppError::message(format!(
                "cleanup_schedule_delete_on_reboot join error: {error}"
            ))
        })?
}
