use crate::shared::shell_notify::notify_shell_change;

const REG_FILE_MENU: &str = r"Software\Classes\*\shell\TakeOwnership";
const REG_FILE_COMMAND: &str = r"Software\Classes\*\shell\TakeOwnership\command";
const REG_DIR_MENU: &str = r"Software\Classes\Directory\shell\TakeOwnership";
const REG_DIR_COMMAND: &str = r"Software\Classes\Directory\shell\TakeOwnership\command";

fn file_command() -> &'static str {
    r#"cmd.exe /c takeown /f "%1" && icacls "%1" /grant *S-1-5-32-544:F"#
}

fn directory_command() -> &'static str {
    r#"cmd.exe /c takeown /f "%1" /r /d y && icacls "%1" /grant *S-1-5-32-544:F /t /c"#
}

#[must_use]
pub fn is_take_ownership_applied() -> bool {
    #[cfg(target_os = "windows")]
    {
        let file_ok = windows_registry::CURRENT_USER
            .open(REG_FILE_COMMAND)
            .is_ok_and(|key| key.get_string("").is_ok_and(|cmd| cmd == file_command()));

        let dir_ok = windows_registry::CURRENT_USER
            .open(REG_DIR_COMMAND)
            .is_ok_and(|key| {
                key.get_string("")
                    .is_ok_and(|cmd| cmd == directory_command())
            });

        file_ok && dir_ok
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_take_ownership(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        if applied {
            let label = rust_i18n::t!("tweaks.take_ownership_menu_item").to_string();

            // 1. Files
            let file_key = windows_registry::CURRENT_USER
                .create(REG_FILE_MENU)
                .map_err(|e| format!("Failed to create file TakeOwnership key: {e}"))?;
            let _ = file_key.set_string("", &label);
            let _ = file_key.set_string("HasLUAShield", "");
            let _ = file_key.set_string("NoWorkingDirectory", "");

            let file_cmd_key = windows_registry::CURRENT_USER
                .create(REG_FILE_COMMAND)
                .map_err(|e| format!("Failed to create file TakeOwnership command key: {e}"))?;
            let _ = file_cmd_key.set_string("", file_command());
            let _ = file_cmd_key.set_string("IsolatedCommand", file_command());

            // 2. Directories
            let dir_key = windows_registry::CURRENT_USER
                .create(REG_DIR_MENU)
                .map_err(|e| format!("Failed to create directory TakeOwnership key: {e}"))?;
            let _ = dir_key.set_string("", &label);
            let _ = dir_key.set_string("HasLUAShield", "");
            let _ = dir_key.set_string("NoWorkingDirectory", "");

            let dir_cmd_key = windows_registry::CURRENT_USER
                .create(REG_DIR_COMMAND)
                .map_err(|e| {
                    format!("Failed to create directory TakeOwnership command key: {e}")
                })?;
            let _ = dir_cmd_key.set_string("", directory_command());
            let _ = dir_cmd_key.set_string("IsolatedCommand", directory_command());
        } else {
            let _ = windows_registry::CURRENT_USER.remove_tree(REG_FILE_MENU);
            let _ = windows_registry::CURRENT_USER.remove_tree(REG_DIR_MENU);
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
