use std::process::Command;

use crate::error::AppError;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

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
    let mut cmd = Command::new("powershell");
    cmd.args(["-NoProfile", "-NonInteractive", "-Command", script]);

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
