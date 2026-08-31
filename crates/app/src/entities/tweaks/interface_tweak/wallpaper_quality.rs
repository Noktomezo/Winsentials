const REG_DESKTOP: &str = r"Control Panel\Desktop";
const VAL_JPEG_IMPORT_QUALITY: &str = "JPEGImportQuality";

#[must_use]
pub fn is_disable_jpeg_compression_applied() -> bool {
    #[cfg(target_os = "windows")]
    {
        if let Ok(key) = windows_registry::CURRENT_USER.open(REG_DESKTOP) {
            key.get_u32(VAL_JPEG_IMPORT_QUALITY).unwrap_or(0) == 100
        } else {
            false
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_disable_jpeg_compression(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let key = windows_registry::CURRENT_USER
            .create(REG_DESKTOP)
            .map_err(|e| format!("Failed to open registry key: {e}"))?;

        if applied {
            key.set_u32(VAL_JPEG_IMPORT_QUALITY, 100)
                .map_err(|e| format!("Failed to set {VAL_JPEG_IMPORT_QUALITY}: {e}"))?;
        } else {
            let _ = key.remove_value(VAL_JPEG_IMPORT_QUALITY);
        }

        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = applied;
        Ok(())
    }
}
