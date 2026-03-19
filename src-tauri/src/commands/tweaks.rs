use crate::error::AppError;
use crate::tweaks::{
    TweakMeta, TweakResult, TweakStatus, WindowsVersion, get_windows_build_number, tweak_by_id,
    tweaks_for_category,
};
use rayon::prelude::*;

#[tauri::command]
pub fn tweaks_by_category(category: String) -> Result<Vec<TweakMeta>, AppError> {
    tweaks_for_category(&category)
        .into_par_iter()
        .map(|tweak| {
            let mut meta = tweak.meta().clone();
            meta.current_value = tweak.get_status()?.current_value;
            Ok(meta)
        })
        .collect()
}

#[tauri::command]
pub fn tweak_apply(id: String, value: String) -> Result<TweakResult, AppError> {
    let tweak = tweak_by_id(&id)?;
    tweak.apply(&value)?;
    tweak.get_status().map(|status| TweakResult {
        success: true,
        current_value: status.current_value,
    })
}

#[tauri::command]
pub fn tweak_reset(id: String) -> Result<TweakResult, AppError> {
    let tweak = tweak_by_id(&id)?;
    tweak.reset()?;
    tweak.get_status().map(|status| TweakResult {
        success: true,
        current_value: status.current_value,
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
