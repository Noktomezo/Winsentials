#![allow(unsafe_code)]

#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::{CloseHandle, INVALID_HANDLE_VALUE};
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, PROCESSENTRY32W, Process32FirstW, Process32NextW, TH32CS_SNAPPROCESS,
};
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_TERMINATE, TerminateProcess};
#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::Shell::ShellExecuteW;
#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

/// Creates a `duct::Expression` configured with `CREATE_NO_WINDOW` (`0x0800_0000`) on Windows,
/// completely suppressing console/terminal window creation when launching console executables
/// from a GUI subsystem application.
pub fn hidden_cmd<T, U>(program: T, args: U) -> duct::Expression
where
    T: duct::IntoExecutablePath,
    U: IntoIterator,
    U::Item: Into<std::ffi::OsString>,
{
    let expr = duct::cmd(program, args);
    #[cfg(target_os = "windows")]
    let expr = expr.before_spawn(|cmd| {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x0800_0000);
        Ok(())
    });
    expr
}

/// Terminates all processes matching `target_name` (case-insensitive, e.g. "explorer.exe", "ctfmon.exe").
/// Returns the number of processes successfully terminated.
pub fn kill_process_by_name(target_name: &str) -> usize {
    #[cfg(target_os = "windows")]
    {
        let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
        if snapshot == INVALID_HANDLE_VALUE {
            return 0;
        }

        let mut entry: PROCESSENTRY32W = unsafe { std::mem::zeroed() };
        entry.dwSize = u32::try_from(std::mem::size_of::<PROCESSENTRY32W>()).unwrap_or(0);

        let mut killed = 0;
        let target_lower = target_name.to_lowercase();

        unsafe {
            if Process32FirstW(snapshot, &raw mut entry) != 0 {
                loop {
                    let name_len = entry
                        .szExeFile
                        .iter()
                        .position(|&c| c == 0)
                        .unwrap_or(entry.szExeFile.len());
                    let exe_name = String::from_utf16_lossy(&entry.szExeFile[..name_len]);

                    if exe_name.to_lowercase() == target_lower {
                        let proc_handle = OpenProcess(PROCESS_TERMINATE, 0, entry.th32ProcessID);
                        if !proc_handle.is_null() {
                            if TerminateProcess(proc_handle, 1) != 0 {
                                killed += 1;
                            }
                            CloseHandle(proc_handle);
                        }
                    }

                    if Process32NextW(snapshot, &raw mut entry) == 0 {
                        break;
                    }
                }
            }
            CloseHandle(snapshot);
        }

        killed
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = target_name;
        0
    }
}

/// Restarts Windows Explorer cleanly using native Win32 APIs without spawning cmd.exe or taskkill.exe.
pub fn restart_explorer() {
    #[cfg(target_os = "windows")]
    {
        kill_process_by_name("explorer.exe");

        // Allow Windows a brief moment to release file system handles
        std::thread::sleep(std::time::Duration::from_millis(300));

        let wide_explorer: Vec<u16> = "explorer.exe".encode_utf16().chain(Some(0)).collect();
        unsafe {
            ShellExecuteW(
                std::ptr::null_mut(),
                std::ptr::null(),
                wide_explorer.as_ptr(),
                std::ptr::null(),
                std::ptr::null(),
                SW_SHOWNORMAL,
            );
        }
    }
}
