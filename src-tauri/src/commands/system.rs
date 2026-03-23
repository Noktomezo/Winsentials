use crate::error::AppError;

#[tauri::command]
pub fn restart_pc() -> Result<(), AppError> {
    #[cfg(target_os = "windows")]
    system_shutdown::reboot().map_err(|e| AppError::message(e.to_string()))?;
    Ok(())
}
