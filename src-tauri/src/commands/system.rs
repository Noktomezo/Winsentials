use crate::error::AppError;

#[tauri::command]
pub fn restart_pc() -> Result<(), AppError> {
    #[cfg(target_os = "windows")]
    system_shutdown::reboot().map_err(|e| AppError::message(e.to_string()))?;
    Ok(())
}

#[tauri::command]
pub fn logout_user() -> Result<(), AppError> {
    #[cfg(target_os = "windows")]
    system_shutdown::logout().map_err(|e| AppError::message(e.to_string()))?;
    Ok(())
}
