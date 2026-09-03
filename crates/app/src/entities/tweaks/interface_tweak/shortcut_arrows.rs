use std::path::PathBuf;

use crate::shared::shell_notify::notify_shell_change;

const BLANK_ICO_BYTES: &[u8] = include_bytes!("../../../../../../assets/icons/blank.ico");
const REG_SHELL_ICONS: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Shell Icons";

fn blank_icon_path() -> Result<PathBuf, String> {
    std::env::var_os("WINDIR")
        .map(PathBuf::from)
        .map(|path| path.join("blank.ico"))
        .ok_or_else(|| "WINDIR is unavailable".to_string())
}

fn legacy_blank_icon_path() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .map(|path| path.join("Winsentials").join("icons").join("blank.ico"))
}

fn remove_legacy_blank_icon() {
    let Some(path) = legacy_blank_icon_path() else {
        return;
    };
    let _ = std::fs::remove_file(&path);
    if let Some(icons_dir) = path.parent() {
        let _ = std::fs::remove_dir(icons_dir);
        if let Some(app_dir) = icons_dir.parent() {
            let _ = std::fs::remove_dir(app_dir);
        }
    }
}

#[must_use]
pub fn is_remove_shortcut_arrows_applied() -> bool {
    #[cfg(target_os = "windows")]
    {
        let has_reg = windows_registry::CURRENT_USER
            .open(REG_SHELL_ICONS)
            .is_ok_and(|key| key.get_string("29").is_ok())
            || windows_registry::LOCAL_MACHINE
                .open(REG_SHELL_ICONS)
                .is_ok_and(|key| key.get_string("29").is_ok());

        has_reg && blank_icon_path().is_ok_and(|path| path.exists())
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_remove_shortcut_arrows(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let blank_ico = blank_icon_path()?;

        if applied {
            std::fs::write(&blank_ico, BLANK_ICO_BYTES)
                .map_err(|error| format!("Failed to write blank icon: {error}"))?;

            let reg_value = format!("{},0", blank_ico.display());
            let mut set_success = false;
            if let Ok(key) = windows_registry::CURRENT_USER.create(REG_SHELL_ICONS) {
                if key.set_string("29", &reg_value).is_ok() {
                    set_success = true;
                }
            }
            if let Ok(key) = windows_registry::LOCAL_MACHINE.create(REG_SHELL_ICONS) {
                if key.set_string("29", &reg_value).is_ok() {
                    set_success = true;
                }
            }

            if !set_success {
                let _ = std::fs::remove_file(&blank_ico);
                return Err("Failed to write to Shell Icons registry".to_string());
            }

            remove_legacy_blank_icon();
        } else {
            if let Ok(key) = windows_registry::CURRENT_USER.create(REG_SHELL_ICONS) {
                let _ = key.remove_value("29");
            }
            if let Ok(key) = windows_registry::LOCAL_MACHINE.create(REG_SHELL_ICONS) {
                let _ = key.remove_value("29");
            }
            if blank_ico.exists() {
                let _ = std::fs::remove_file(&blank_ico);
            }
            remove_legacy_blank_icon();
        }

        notify_shell_change();
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = applied;
        Ok(())
    }
}
