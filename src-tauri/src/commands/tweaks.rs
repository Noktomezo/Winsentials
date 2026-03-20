use crate::error::AppError;
use crate::tweaks::{
    TweakMeta, TweakResult, TweakStatus, WindowsVersion, get_windows_build_number, tweak_by_id,
    tweaks_for_category,
};
use rayon::prelude::*;

#[tauri::command]
pub fn tweaks_by_category(category: String) -> Result<Vec<TweakMeta>, AppError> {
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
}

#[tauri::command]
pub fn tweak_apply(id: String, value: String) -> Result<TweakResult, AppError> {
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
pub fn tweak_reset(id: String) -> Result<TweakResult, AppError> {
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
}

#[tauri::command]
pub fn tweak_status(id: String) -> Result<TweakStatus, AppError> {
    tweak_by_id(&id)?.get_status()
}

#[tauri::command]
pub fn tweak_extra(id: String) -> Result<(), AppError> {
    tweak_by_id(&id)?.extra()
}

#[tauri::command]
pub fn get_windows_build() -> Result<WindowsVersion, AppError> {
    get_windows_build_number()
}
