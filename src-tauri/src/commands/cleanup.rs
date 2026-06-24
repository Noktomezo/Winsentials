use crate::cleanup;
use crate::cleanup::types::{CategoryCleanRequest, CleanupCategoryReport, CleanupScheduleReport};
use crate::cleanup::winapp_db::{self, WinappDbStatus};
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
    exclude_entry_ids: Vec<String>,
) -> Result<CleanupCategoryReport, AppError> {
    tauri::async_runtime::spawn_blocking(move || {
        cleanup::cleanup_clean_category(&category_id, &exclude_entry_ids)
    })
    .await
    .map_err(|error| AppError::message(format!("cleanup_clean_category join error: {error}")))?
}

#[tauri::command]
pub async fn cleanup_scan_all() -> Result<Vec<CleanupCategoryReport>, AppError> {
    tauri::async_runtime::spawn_blocking(cleanup::cleanup_scan_all)
        .await
        .map_err(|error| AppError::message(format!("cleanup_scan_all join error: {error}")))?
}

#[tauri::command]
pub async fn cleanup_clean_all(
    requests: Vec<CategoryCleanRequest>,
) -> Result<Vec<CleanupCategoryReport>, AppError> {
    tauri::async_runtime::spawn_blocking(move || cleanup::cleanup_clean_all(&requests))
        .await
        .map_err(|error| AppError::message(format!("cleanup_clean_all join error: {error}")))?
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

#[tauri::command]
pub async fn cleanup_update_winapp_db() -> Result<WinappDbStatus, AppError> {
    winapp_db::download_winapp2().await?;
    match winapp_db::download_winappx().await {
        Ok(_) => {}
        Err(error) => log::warn!("Winappx.ini download skipped: {error}"),
    }
    Ok(winapp_db::winapp_db_status())
}

#[tauri::command]
pub fn cleanup_winapp_db_status() -> WinappDbStatus {
    winapp_db::winapp_db_status()
}

#[tauri::command]
pub async fn cleanup_set_custom_winapp2_path(
    path: Option<String>,
) -> Result<WinappDbStatus, AppError> {
    let path = path.map(std::path::PathBuf::from);
    tauri::async_runtime::spawn_blocking(move || winapp_db::set_custom_winapp2_path(path))
        .await
        .map_err(|error| {
            AppError::message(format!(
                "cleanup_set_custom_winapp2_path join error: {error}"
            ))
        })??;
    winapp_db::refresh_cache();
    Ok(winapp_db::winapp_db_status())
}
