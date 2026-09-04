const REG_EXPLORER: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer";
const REG_NAMING_TEMPLATES: &str =
    r"Software\Microsoft\Windows\CurrentVersion\Explorer\NamingTemplates";

#[must_use]
pub fn is_remove_shortcut_suffix_applied() -> bool {
    #[cfg(target_os = "windows")]
    {
        if let Ok(key) = windows_registry::CURRENT_USER.open(REG_NAMING_TEMPLATES) {
            if let Ok(val) = key.get_string("ShortcutNameTemplate") {
                if val == "%s.lnk" || val == "%s" {
                    return true;
                }
            }
        }
        false
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_remove_shortcut_suffix(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        if applied {
            // Method 1: Set ShortcutNameTemplate = "%s.lnk"
            let naming_key = windows_registry::CURRENT_USER
                .create(REG_NAMING_TEMPLATES)
                .map_err(|e| format!("Failed to create NamingTemplates key: {e}"))?;
            naming_key
                .set_string("ShortcutNameTemplate", "%s.lnk")
                .map_err(|e| format!("Failed to set ShortcutNameTemplate: {e}"))?;

            // Method 2: Set link = [0, 0, 0, 0]
            if let Ok(explorer_key) = windows_registry::CURRENT_USER.create(REG_EXPLORER) {
                let _ =
                    explorer_key.set_bytes("link", windows_registry::Type::Bytes, &[0u8, 0, 0, 0]);
            }
        } else {
            // Remove ShortcutNameTemplate
            let _ = windows_registry::CURRENT_USER.remove_tree(REG_NAMING_TEMPLATES);

            // Remove link override
            if let Ok(explorer_key) = windows_registry::CURRENT_USER.create(REG_EXPLORER) {
                let _ = explorer_key.remove_value("link");
            }
        }

        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = applied;
        Ok(())
    }
}
