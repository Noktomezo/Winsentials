const REG_DESKTOP: &str = r"Control Panel\Desktop";

#[must_use]
pub fn is_menu_show_delay_disabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        if let Ok(key) = windows_registry::CURRENT_USER.open(REG_DESKTOP) {
            key.get_string("MenuShowDelay")
                .unwrap_or_else(|_| "400".to_string())
                == "0"
        } else {
            false
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_menu_show_delay_disabled(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let key = windows_registry::CURRENT_USER
            .create(REG_DESKTOP)
            .map_err(|e| format!("Failed to open registry key: {e}"))?;
        let delay_val = if applied { "0" } else { "400" };
        key.set_string("MenuShowDelay", delay_val)
            .map_err(|e| format!("Failed to set MenuShowDelay value: {e}"))?;
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = applied;
        Ok(())
    }
}
