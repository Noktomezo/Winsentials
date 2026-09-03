const REG_RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const APP_RUN_NAME: &str = "Winsentials";

#[allow(dead_code)]
#[must_use]
pub fn is_autostart_enabled() -> (bool, bool) {
    if let Ok(key) = windows_registry::CURRENT_USER.open(REG_RUN) {
        if let Ok(val) = key.get_string(APP_RUN_NAME) {
            let in_tray = val.contains("--tray") || val.contains("--minimized");
            return (true, in_tray);
        }
    }
    (false, false)
}

pub fn set_autostart(enabled: bool, start_to_tray: bool) -> Result<(), String> {
    if enabled {
        let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
        let cmd = if start_to_tray {
            format!("\"{}\" --tray", exe_path.display())
        } else {
            format!("\"{}\"", exe_path.display())
        };
        let key = windows_registry::CURRENT_USER
            .create(REG_RUN)
            .map_err(|e| e.to_string())?;
        key.set_string(APP_RUN_NAME, &cmd)
            .map_err(|e| e.to_string())?;
    } else if let Ok(key) = windows_registry::CURRENT_USER.create(REG_RUN) {
        let _ = key.remove_value(APP_RUN_NAME);
    }
    Ok(())
}
