const REG_KEY_CLSID: &str = r"Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}";
const REG_KEY_INPROC: &str =
    r"Software\Classes\CLSID\{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}\InprocServer32";

#[must_use]
pub fn is_classic_context_menu_applied() -> bool {
    #[cfg(target_os = "windows")]
    {
        windows_registry::CURRENT_USER.open(REG_KEY_INPROC).is_ok()
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_classic_context_menu(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        if applied {
            let key = windows_registry::CURRENT_USER
                .create(REG_KEY_INPROC)
                .map_err(|e| format!("Failed to create registry key: {e}"))?;
            key.set_string("", "")
                .map_err(|e| format!("Failed to set registry value: {e}"))?;
        } else {
            // Remove the custom CLSID override tree
            let _ = windows_registry::CURRENT_USER.remove_tree(REG_KEY_CLSID);
        }

        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = applied;
        Ok(())
    }
}

pub fn restart_explorer() {
    crate::shared::process::restart_explorer();
}
