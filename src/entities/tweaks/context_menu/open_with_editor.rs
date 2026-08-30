use std::path::PathBuf;

const REG_UNKNOWN_SHELL: &str = r"Software\Classes\Unknown\shell\OpenWithNotepad";
const REG_UNKNOWN_COMMAND: &str = r"Software\Classes\Unknown\shell\OpenWithNotepad\command";
const REG_STAR_SHELL: &str = r"Software\Classes\*\shell\OpenWithNotepad";
const REG_NFO_EXT: &str = r"Software\Classes\.nfo";
const REG_NFO_COMMAND: &str = r"Software\Classes\nfo_auto_file\shell\open\command";

#[must_use]
pub fn detect_notepad() -> PathBuf {
    if let Ok(sys_root) = std::env::var("SystemRoot") {
        let p = PathBuf::from(format!(r"{sys_root}\System32\notepad.exe"));
        if p.is_file() {
            return p;
        }
        let p2 = PathBuf::from(format!(r"{sys_root}\notepad.exe"));
        if p2.is_file() {
            return p2;
        }
    }
    PathBuf::from("notepad.exe")
}

#[must_use]
pub fn is_open_with_notepad_applied() -> bool {
    #[cfg(target_os = "windows")]
    {
        windows_registry::CURRENT_USER
            .open(REG_UNKNOWN_COMMAND)
            .is_ok()
            || windows_registry::CURRENT_USER
                .open(r"Software\Classes\Unknown\shell\OpenWithApp\command")
                .is_ok()
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_open_with_notepad(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        if applied {
            let notepad_path = detect_notepad();
            let exe_str = notepad_path.to_string_lossy().to_string();
            let menu_label = rust_i18n::t!("tweaks.open_with_notepad_menu_label").to_string();
            let cmd_str = format!("\"{exe_str}\" \"%1\"");

            // Clean up any legacy OpenWithApp keys
            let _ =
                windows_registry::CURRENT_USER.remove_tree(r"Software\Classes\*\shell\OpenWithApp");
            let _ = windows_registry::CURRENT_USER
                .remove_tree(r"Software\Classes\Unknown\shell\OpenWithApp");

            // 1. Unknown context menu (for files with unknown extensions)
            let unk_key = windows_registry::CURRENT_USER
                .create(REG_UNKNOWN_SHELL)
                .map_err(|e| format!("Failed to create Unknown shell key: {e}"))?;
            let _ = unk_key.set_string("", &menu_label);
            let _ = unk_key.set_string("Icon", &exe_str);

            let unk_cmd_key = windows_registry::CURRENT_USER
                .create(REG_UNKNOWN_COMMAND)
                .map_err(|e| format!("Failed to create Unknown command key: {e}"))?;
            let _ = unk_cmd_key.set_string("", &cmd_str);

            // 2. .nfo extension association
            let nfo_key = windows_registry::CURRENT_USER
                .create(REG_NFO_EXT)
                .map_err(|e| format!("Failed to create .nfo key: {e}"))?;
            let _ = nfo_key.set_string("", "nfo_auto_file");

            let nfo_cmd_key = windows_registry::CURRENT_USER
                .create(REG_NFO_COMMAND)
                .map_err(|e| format!("Failed to create nfo_auto_file command key: {e}"))?;
            let _ = nfo_cmd_key.set_string("", &cmd_str);
        } else {
            let _ = windows_registry::CURRENT_USER.remove_tree(REG_UNKNOWN_SHELL);
            let _ = windows_registry::CURRENT_USER.remove_tree(REG_STAR_SHELL);
            let _ =
                windows_registry::CURRENT_USER.remove_tree(r"Software\Classes\*\shell\OpenWithApp");
            let _ = windows_registry::CURRENT_USER
                .remove_tree(r"Software\Classes\Unknown\shell\OpenWithApp");
            let _ = windows_registry::CURRENT_USER.remove_tree(r"Software\Classes\nfo_auto_file");
            let _ = windows_registry::CURRENT_USER.remove_tree(REG_NFO_EXT);
        }
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = applied;
        Ok(())
    }
}
