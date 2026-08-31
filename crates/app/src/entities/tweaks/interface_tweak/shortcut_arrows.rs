const BLANK_ICO_PATH: &str = r"C:\Windows\blank.ico";
const BLANK_ICO_REG_VALUE: &str = r"C:\Windows\blank.ico,0";
const BLANK_ICO_BYTES: &[u8] = include_bytes!("../../../../../../assets/icons/blank.ico");

const REG_SHELL_ICONS: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Shell Icons";

#[must_use]
pub fn is_remove_shortcut_arrows_applied() -> bool {
    #[cfg(target_os = "windows")]
    {
        if let Ok(key) = windows_registry::LOCAL_MACHINE.open(REG_SHELL_ICONS) {
            if key.get_string("29").is_ok() {
                return true;
            }
        }
        if let Ok(key) = windows_registry::CURRENT_USER.open(REG_SHELL_ICONS) {
            if key.get_string("29").is_ok() {
                return true;
            }
        }
        false
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_remove_shortcut_arrows(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        if applied {
            // 1. Ensure C:\Windows\blank.ico is written to disk
            if !std::path::Path::new(BLANK_ICO_PATH).exists() {
                let _ = std::fs::write(BLANK_ICO_PATH, BLANK_ICO_BYTES);
            }

            // 2. Set registry value in HKLM and HKCU
            let mut set_success = false;
            if let Ok(key) = windows_registry::LOCAL_MACHINE.create(REG_SHELL_ICONS) {
                if key.set_string("29", BLANK_ICO_REG_VALUE).is_ok() {
                    set_success = true;
                }
            }
            if let Ok(key) = windows_registry::CURRENT_USER.create(REG_SHELL_ICONS) {
                let _ = key.set_string("29", BLANK_ICO_REG_VALUE);
                set_success = true;
            }

            if !set_success {
                return Err("Failed to write to Shell Icons registry".to_string());
            }
        } else {
            // Remove value 29 from HKLM and HKCU
            if let Ok(key) = windows_registry::LOCAL_MACHINE.open(REG_SHELL_ICONS) {
                let _ = key.remove_value("29");
            }
            if let Ok(key) = windows_registry::CURRENT_USER.open(REG_SHELL_ICONS) {
                let _ = key.remove_value("29");
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
