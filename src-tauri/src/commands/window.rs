use crate::error::AppError;
use serde::Deserialize;
use tauri::{AppHandle, Manager, Runtime};

#[cfg(target_os = "windows")]
use window_vibrancy::{
    apply_acrylic, apply_mica, apply_tabbed, clear_acrylic, clear_blur, clear_mica, clear_tabbed,
};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WebviewMaterial {
    None,
    Acrylic,
    Mica,
    Tabbed,
}

#[cfg(target_os = "windows")]
fn clear_all_backdrops<R: Runtime>(window: &tauri::WebviewWindow<R>) {
    let _ = clear_tabbed(window);
    let _ = clear_mica(window);
    let _ = clear_acrylic(window);
    let _ = clear_blur(window);
}

#[tauri::command]
pub fn set_webview_material<R: Runtime>(
    app: AppHandle<R>,
    material: WebviewMaterial,
    theme: String,
) -> Result<bool, AppError> {
    #[cfg(target_os = "windows")]
    {
        let window = app
            .get_webview_window("main")
            .ok_or_else(|| AppError::message("main window not found"))?;

        clear_all_backdrops(&window);

        let is_dark = match theme.as_str() {
            "light" => Some(false),
            "dark" => Some(true),
            _ => None,
        };

        let acrylic_tint = match theme.as_str() {
            "light" => Some((232, 238, 242, 128)),
            _ => Some((28, 34, 38, 128)),
        };

        let result = match material {
            WebviewMaterial::None => return Ok(true),
            WebviewMaterial::Acrylic => apply_acrylic(&window, acrylic_tint),
            WebviewMaterial::Mica => apply_mica(&window, is_dark),
            WebviewMaterial::Tabbed => apply_tabbed(&window, is_dark),
        };

        match result {
            Ok(()) => Ok(true),
            Err(_) => {
                clear_all_backdrops(&window);
                Ok(false)
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (app, theme);
        Ok(matches!(material, WebviewMaterial::None))
    }
}
