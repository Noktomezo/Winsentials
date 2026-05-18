use crate::error::AppError;
use crate::tweaks::{
    TweakMeta, TweakResult, TweakStatus, WindowsVersion, get_windows_build_number, tweak_by_id,
    tweaks_for_category,
};
use rayon::prelude::*;

#[tauri::command]
pub async fn tweaks_by_category(category: String) -> Result<Vec<TweakMeta>, AppError> {
    tauri::async_runtime::spawn_blocking(move || {
        Ok(tweaks_for_category(&category)
            .into_par_iter()
            .map(|tweak| {
                let mut meta = tweak.meta().clone();
                if let Ok(status) = tweak.get_status() {
                    meta.current_value = status.current_value;
                }
                meta
            })
            .collect())
    })
    .await
    .map_err(|error| AppError::message(format!("tweaks_by_category join error: {error}")))?
}

#[tauri::command]
pub async fn tweak_apply(id: String, value: String) -> Result<TweakResult, AppError> {
    tauri::async_runtime::spawn_blocking(move || tweak_apply_blocking(id, value))
        .await
        .map_err(|error| AppError::message(format!("tweak_apply join error: {error}")))?
}

pub fn tweak_apply_blocking(id: String, value: String) -> Result<TweakResult, AppError> {
    let tweak = tweak_by_id(&id)?;
    tweak.apply(&value)?;
    let current_value = tweak
        .get_status()
        .ok()
        .map(|status| status.current_value)
        .unwrap_or_else(|| value.clone());
    Ok(TweakResult {
        success: true,
        current_value,
    })
}

#[tauri::command]
pub async fn tweak_reset(id: String) -> Result<TweakResult, AppError> {
    tauri::async_runtime::spawn_blocking(move || {
        let tweak = tweak_by_id(&id)?;
        tweak.reset()?;
        let current_value = tweak
            .get_status()
            .ok()
            .map(|status| status.current_value)
            .unwrap_or_else(|| tweak.meta().default_value.clone());
        Ok(TweakResult {
            success: true,
            current_value,
        })
    })
    .await
    .map_err(|error| AppError::message(format!("tweak_reset join error: {error}")))?
}

#[tauri::command]
pub async fn tweak_status(id: String) -> Result<TweakStatus, AppError> {
    tauri::async_runtime::spawn_blocking(move || tweak_by_id(&id)?.get_status())
        .await
        .map_err(|error| AppError::message(format!("tweak_status join error: {error}")))?
}

#[tauri::command]
pub async fn tweak_extra(id: String) -> Result<(), AppError> {
    tauri::async_runtime::spawn_blocking(move || tweak_by_id(&id)?.extra())
        .await
        .map_err(|error| AppError::message(format!("tweak_extra join error: {error}")))?
}

#[tauri::command]
pub async fn get_windows_build() -> Result<WindowsVersion, AppError> {
    tauri::async_runtime::spawn_blocking(get_windows_build_number)
        .await
        .map_err(|error| AppError::message(format!("get_windows_build join error: {error}")))?
}
