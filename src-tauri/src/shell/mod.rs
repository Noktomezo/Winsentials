use std::process::Command;

use crate::error::AppError;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
use windows::Win32::UI::Shell::{SHCNE_ASSOCCHANGED, SHCNF_IDLIST, SHChangeNotify};

#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{
    SPI_SETMOUSEHOVERTIME, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE,
    SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS, SetCaretBlinkTime, SystemParametersInfoW,
};

// Prevents a console window from flashing when spawning child processes.
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn run_netsh(args: &[&str]) -> Result<String, AppError> {
    let mut cmd = Command::new("netsh");
    cmd.args(args);

    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let output = cmd.output()?;

    if output.status.success() {
        return String::from_utf8(output.stdout).map_err(AppError::from);
    }

    Err(AppError::CommandFailed {
        command: "netsh".to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
    })
}

pub fn run_powershell(script: &str) -> Result<String, AppError> {
    let wrapped_script = format!(
        "[Console]::InputEncoding = [System.Text.UTF8Encoding]::new($false); \
[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false); \
$OutputEncoding = [Console]::OutputEncoding; \
{}",
        script
    );
    let mut cmd = Command::new("powershell");
    cmd.args(["-NoProfile", "-NonInteractive", "-Command", &wrapped_script]);

    #[cfg(target_os = "windows")]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let output = cmd.output()?;

    if output.status.success() {
        return String::from_utf8(output.stdout).map_err(AppError::from);
    }

    Err(AppError::CommandFailed {
        command: "powershell".to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
    })
}

pub fn restart_explorer() -> Result<(), AppError> {
    run_powershell(
        "Stop-Process -Name explorer -Force -ErrorAction SilentlyContinue; Start-Process explorer.exe",
    )?;

    Ok(())
}

/// Broadcasts a shell namespace change notification so Explorer refreshes its
/// navigation pane without restarting. The last two parameters must be null
/// raw pointers — the `windows` crate signature is `*const c_void`, not `Option`.
pub fn refresh_shell_namespace() -> Result<(), AppError> {
    #[cfg(target_os = "windows")]
    unsafe {
        SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, None, None);
    }
    Ok(())
}

/// Sets the caret (text cursor) blink interval in milliseconds.
/// Takes effect immediately for all running apps via the Win32 API.
pub fn set_caret_blink_time(ms: u32) -> Result<(), AppError> {
    #[cfg(target_os = "windows")]
    unsafe {
        SetCaretBlinkTime(ms)
            .map_err(|e| AppError::message(format!("SetCaretBlinkTime failed: {e}")))?;
    }
    Ok(())
}

/// Sets the mouse hover delay in milliseconds system-wide.
/// Broadcasts the change so running apps pick it up immediately.
pub fn set_mouse_hover_time(ms: u32) -> Result<(), AppError> {
    #[cfg(target_os = "windows")]
    unsafe {
        let flags = SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(SPIF_UPDATEINIFILE.0 | SPIF_SENDCHANGE.0);
        SystemParametersInfoW(SPI_SETMOUSEHOVERTIME, ms, None, flags)
            .map_err(|e| AppError::message(format!("SystemParametersInfoW failed: {e}")))?;
    }
    Ok(())
}
