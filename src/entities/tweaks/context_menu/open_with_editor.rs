use std::path::PathBuf;

use crate::shared::shell_notify::notify_shell_change;

const REG_UNKNOWN_SHELL_ROOT: &str = r"Software\Classes\Unknown\shell";
const REG_UNKNOWN_OPEN: &str = r"Software\Classes\Unknown\shell\open";
const REG_UNKNOWN_COMMAND: &str = r"Software\Classes\Unknown\shell\open\command";
const REG_UNKNOWN_OPENAS_COMMAND: &str = r"Software\Classes\Unknown\shell\openas\command";

const REG_DOT_EXT: &str = r"Software\Classes\.";
const REG_DOT_FILE_EXTS: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.";
const REG_DOT_OPEN_WITH_PROGIDS: &str =
    r"Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.\OpenWithProgids";

const REG_NFO_EXT: &str = r"Software\Classes\.nfo";
const REG_NFO_FILE_EXTS: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.nfo";
const REG_NFO_OPEN_WITH_PROGIDS: &str =
    r"Software\Microsoft\Windows\CurrentVersion\Explorer\FileExts\.nfo\OpenWithProgids";

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
        let unknown_applied = windows_registry::CURRENT_USER
            .open(REG_UNKNOWN_COMMAND)
            .is_ok_and(|key| key.get_string("").is_ok_and(|cmd| !cmd.is_empty()));

        let dot_applied = windows_registry::CURRENT_USER
            .open(REG_DOT_EXT)
            .is_ok_and(|key| key.get_string("").is_ok_and(|val| val == "txtfile"));

        let nfo_applied = windows_registry::CURRENT_USER
            .open(REG_NFO_EXT)
            .is_ok_and(|key| key.get_string("").is_ok_and(|val| val == "txtfile"));

        unknown_applied && dot_applied && nfo_applied
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

            // 1. Unknown extensions: default "open" and silence "openas" dialog
            let shell_root_key = windows_registry::CURRENT_USER
                .create(REG_UNKNOWN_SHELL_ROOT)
                .map_err(|e| format!("Failed to create Unknown shell key: {e}"))?;
            let _ = shell_root_key.set_string("", "open");

            let open_key = windows_registry::CURRENT_USER
                .create(REG_UNKNOWN_OPEN)
                .map_err(|e| format!("Failed to create Unknown open key: {e}"))?;
            let _ = open_key.set_string("", &menu_label);
            let _ = open_key.set_string("Icon", &exe_str);

            let cmd_key = windows_registry::CURRENT_USER
                .create(REG_UNKNOWN_COMMAND)
                .map_err(|e| format!("Failed to create Unknown command key: {e}"))?;
            let _ = cmd_key.set_string("", &cmd_str);

            let openas_cmd_key = windows_registry::CURRENT_USER
                .create(REG_UNKNOWN_OPENAS_COMMAND)
                .map_err(|e| format!("Failed to create Unknown openas command key: {e}"))?;
            let _ = openas_cmd_key.set_string("", &cmd_str);

            // 2. Files with no extension (.)
            let dot_key = windows_registry::CURRENT_USER
                .create(REG_DOT_EXT)
                .map_err(|e| format!("Failed to create . key: {e}"))?;
            let _ = dot_key.set_string("", "txtfile");
            let _ = dot_key.set_string("Content Type", "text/plain");
            let _ = dot_key.set_string("PerceivedType", "text");

            let _ = windows_registry::CURRENT_USER.remove_tree(REG_DOT_FILE_EXTS);
            if let Ok(dot_progids) =
                windows_registry::CURRENT_USER.create(REG_DOT_OPEN_WITH_PROGIDS)
            {
                let _ = dot_progids.set_bytes("txtfile", windows_registry::Type::Other(0), &[]);
            }

            // 3. .nfo extension
            let nfo_key = windows_registry::CURRENT_USER
                .create(REG_NFO_EXT)
                .map_err(|e| format!("Failed to create .nfo key: {e}"))?;
            let _ = nfo_key.set_string("", "txtfile");
            let _ = nfo_key.set_string("Content Type", "text/plain");
            let _ = nfo_key.set_string("PerceivedType", "text");

            let _ = windows_registry::CURRENT_USER.remove_tree(REG_NFO_FILE_EXTS);
            if let Ok(nfo_progids) =
                windows_registry::CURRENT_USER.create(REG_NFO_OPEN_WITH_PROGIDS)
            {
                let _ = nfo_progids.set_bytes("txtfile", windows_registry::Type::Other(0), &[]);
            }

            // Clean up any legacy keys
            let _ =
                windows_registry::CURRENT_USER.remove_tree(r"Software\Classes\*\shell\OpenWithApp");
            let _ = windows_registry::CURRENT_USER
                .remove_tree(r"Software\Classes\Unknown\shell\OpenWithApp");
            let _ = windows_registry::CURRENT_USER
                .remove_tree(r"Software\Classes\Unknown\shell\OpenWithNotepad");
            let _ = windows_registry::CURRENT_USER.remove_tree(r"Software\Classes\nfo_auto_file");
        } else {
            // 1. Revert Unknown files
            let _ = windows_registry::CURRENT_USER
                .remove_tree(r"Software\Classes\Unknown\shell\openas");
            let _ = windows_registry::CURRENT_USER.remove_tree(REG_UNKNOWN_OPEN);
            let _ = windows_registry::CURRENT_USER.remove_tree(REG_UNKNOWN_SHELL_ROOT);
            let _ = windows_registry::CURRENT_USER.remove_tree(r"Software\Classes\Unknown");
            let _ =
                windows_registry::CURRENT_USER.remove_tree(r"Software\Classes\*\shell\OpenWithApp");
            let _ = windows_registry::CURRENT_USER
                .remove_tree(r"Software\Classes\Unknown\shell\OpenWithApp");
            let _ = windows_registry::CURRENT_USER
                .remove_tree(r"Software\Classes\Unknown\shell\OpenWithNotepad");

            // 2. Revert Files without extension (.)
            let _ = windows_registry::CURRENT_USER.remove_tree(REG_DOT_EXT);
            let _ = windows_registry::CURRENT_USER.remove_tree(REG_DOT_FILE_EXTS);

            // 3. Revert .nfo
            let _ = windows_registry::CURRENT_USER.remove_tree(REG_NFO_EXT);
            let _ = windows_registry::CURRENT_USER.remove_tree(REG_NFO_FILE_EXTS);
            let _ = windows_registry::CURRENT_USER.remove_tree(r"Software\Classes\nfo_auto_file");
        }

        // Notify Windows Shell to refresh file associations immediately
        notify_shell_change();

        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = applied;
        Ok(())
    }
}
