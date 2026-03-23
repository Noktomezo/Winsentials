use crate::error::AppError;
use tauri::{AppHandle, Manager, Runtime};

#[cfg(target_os = "windows")]
use window_vibrancy::{apply_acrylic, clear_acrylic};

#[tauri::command]
pub fn set_chrome_acrylic<R: Runtime>(
    app: AppHandle<R>,
    enabled: bool,
    theme: String,
) -> Result<bool, AppError> {
    #[cfg(target_os = "windows")]
    {
        let window = app
            .get_webview_window("main")
            .ok_or_else(|| AppError::message("main window not found"))?;

        if enabled {
            let tint = match theme.as_str() {
                "light" => Some((232, 238, 242, 128)),
                _ => Some((28, 34, 38, 128)),
            };

            apply_acrylic(&window, tint).map_err(|error| AppError::message(error.to_string()))?;
            return Ok(true);
        }

        clear_acrylic(&window).map_err(|error| AppError::message(error.to_string()))?;
        Ok(false)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (app, enabled, theme);
        Ok(false)
    }
}
