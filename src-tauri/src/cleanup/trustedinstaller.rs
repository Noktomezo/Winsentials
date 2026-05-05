use std::error::Error;
use std::ffi::c_void;
use std::mem::size_of;
use std::path::Path;
use std::ptr::null_mut;
use std::thread;
use std::time::Duration;

use windows::Win32::Foundation::{
    CloseHandle, ERROR_NOT_ALL_ASSIGNED, GetLastError, HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT,
};
use windows::Win32::Security::{
    AdjustTokenPrivileges, DuplicateTokenEx, LUID_AND_ATTRIBUTES, LookupPrivilegeValueW,
    SE_PRIVILEGE_ENABLED, SecurityImpersonation, TOKEN_ADJUST_DEFAULT, TOKEN_ADJUST_PRIVILEGES,
    TOKEN_ADJUST_SESSIONID, TOKEN_ASSIGN_PRIMARY, TOKEN_DUPLICATE, TOKEN_IMPERSONATE,
    TOKEN_PRIVILEGES, TOKEN_QUERY, TokenPrimary,
};
use windows::Win32::System::Environment::{CreateEnvironmentBlock, DestroyEnvironmentBlock};
use windows::Win32::System::Services::{
    CloseServiceHandle, OpenSCManagerW, OpenServiceW, QueryServiceStatusEx, SC_HANDLE,
    SC_MANAGER_CONNECT, SC_STATUS_PROCESS_INFO, SERVICE_QUERY_STATUS, SERVICE_RUNNING,
    SERVICE_START, SERVICE_START_PENDING, SERVICE_STATUS_PROCESS, StartServiceW,
};
use windows::Win32::System::Threading::{
    CREATE_NO_WINDOW, CREATE_UNICODE_ENVIRONMENT, CreateProcessAsUserW, GetCurrentProcess,
    GetExitCodeProcess, OpenProcess, OpenProcessToken, PROCESS_INFORMATION,
    PROCESS_QUERY_LIMITED_INFORMATION, STARTF_USESHOWWINDOW, STARTUPINFOW, TerminateProcess,
    WaitForSingleObject,
};
use windows::Win32::UI::WindowsAndMessaging::SW_HIDE;
use windows::core::{PCWSTR, PWSTR, w};

struct OwnedHandle(HANDLE);

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            unsafe {
                let _ = CloseHandle(self.0);
            }
        }
    }
}

struct OwnedService(SC_HANDLE);

impl Drop for OwnedService {
    fn drop(&mut self) {
        if !self.0.is_invalid() {
            unsafe {
                let _ = CloseServiceHandle(self.0);
            }
        }
    }
}

/// Starts `exe` hidden as `NT SERVICE\TrustedInstaller`.
///
/// The function starts the TrustedInstaller service, duplicates its primary
/// token and creates a child process with that token. The caller still needs to
/// run elevated and hold the Windows privileges required by
/// `CreateProcessAsUserW`.
pub fn run_as_trustedinstaller(exe: &str, args: &[&str]) -> Result<(), Box<dyn Error>> {
    if !Path::new(exe).is_absolute() {
        return Err("TrustedInstaller launches must use an absolute executable path".into());
    }

    unsafe {
        // Needed to open the TrustedInstaller process/token from an admin app.
        enable_privilege(w!("SeDebugPrivilege"))?;
        enable_privilege(w!("SeIncreaseQuotaPrivilege"))?;
        let _ = enable_privilege(w!("SeAssignPrimaryTokenPrivilege"));

        let trustedinstaller_pid = start_trustedinstaller_and_get_pid()?;
        let trustedinstaller_process = OwnedHandle(OpenProcess(
            PROCESS_QUERY_LIMITED_INFORMATION,
            false,
            trustedinstaller_pid,
        )?);

        let mut source_token = HANDLE::default();
        OpenProcessToken(
            trustedinstaller_process.0,
            TOKEN_DUPLICATE | TOKEN_ASSIGN_PRIMARY | TOKEN_QUERY,
            &mut source_token,
        )?;
        let source_token = OwnedHandle(source_token);

        let mut primary_token = HANDLE::default();
        DuplicateTokenEx(
            source_token.0,
            TOKEN_ASSIGN_PRIMARY
                | TOKEN_DUPLICATE
                | TOKEN_IMPERSONATE
                | TOKEN_QUERY
                | TOKEN_ADJUST_DEFAULT
                | TOKEN_ADJUST_SESSIONID,
            None,
            SecurityImpersonation,
            TokenPrimary,
            &mut primary_token,
        )?;
        let primary_token = OwnedHandle(primary_token);

        let exe_w = wide_null(exe);
        let mut command_line = wide_null(&make_command_line(exe, args));

        // Build the TrustedInstaller environment when possible. Absolute paths
        // still work if this fails, so we gracefully fall back to inherited env.
        let mut environment: *mut c_void = null_mut();
        let has_environment =
            CreateEnvironmentBlock(&mut environment, Some(primary_token.0), false).is_ok();

        let startup_info = STARTUPINFOW {
            cb: size_of::<STARTUPINFOW>() as u32,
            dwFlags: STARTF_USESHOWWINDOW,
            wShowWindow: SW_HIDE.0 as u16,
            ..Default::default()
        };
        let mut process_info = PROCESS_INFORMATION::default();
        let creation_flags = if has_environment {
            CREATE_NO_WINDOW | CREATE_UNICODE_ENVIRONMENT
        } else {
            CREATE_NO_WINDOW
        };

        let create_result = CreateProcessAsUserW(
            Some(primary_token.0),
            PCWSTR(exe_w.as_ptr()),
            Some(PWSTR(command_line.as_mut_ptr())),
            None,
            None,
            false,
            creation_flags,
            if has_environment {
                Some(environment.cast_const())
            } else {
                None
            },
            PCWSTR::null(),
            &startup_info,
            &mut process_info,
        );

        if has_environment {
            let _ = DestroyEnvironmentBlock(environment);
        }

        create_result?;

        let process = OwnedHandle(process_info.hProcess);
        let _thread = OwnedHandle(process_info.hThread);

        // Cleanup callers need deterministic results, so wait for the hidden
        // TrustedInstaller child to finish and surface a non-zero exit code.
        match WaitForSingleObject(process.0, 5 * 60 * 1000) {
            WAIT_OBJECT_0 => {}
            WAIT_TIMEOUT => {
                let _ = TerminateProcess(process.0, 1);
                let _ = WaitForSingleObject(process.0, 5000);
                return Err("TrustedInstaller process timed out".into());
            }
            _ => return Err(windows::core::Error::from_thread().into()),
        }
        let mut exit_code = 0u32;
        GetExitCodeProcess(process.0, &mut exit_code)?;
        if exit_code != 0 {
            return Err(format!("TrustedInstaller process exited with code {exit_code}").into());
        }

        Ok(())
    }
}

unsafe fn enable_privilege(name: PCWSTR) -> windows::core::Result<()> {
    let mut token = HANDLE::default();
    unsafe {
        OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            &mut token,
        )?;
    }
    let token = OwnedHandle(token);

    let mut luid = Default::default();
    unsafe {
        LookupPrivilegeValueW(None, name, &mut luid)?;
    }

    let privileges = TOKEN_PRIVILEGES {
        PrivilegeCount: 1,
        Privileges: [LUID_AND_ATTRIBUTES {
            Luid: luid,
            Attributes: SE_PRIVILEGE_ENABLED,
        }],
    };

    unsafe {
        AdjustTokenPrivileges(token.0, false, Some(&privileges), 0, None, None)?;
        if GetLastError() == ERROR_NOT_ALL_ASSIGNED {
            return Err(windows::core::Error::from_thread());
        }
    }

    Ok(())
}

unsafe fn start_trustedinstaller_and_get_pid() -> Result<u32, Box<dyn Error>> {
    let service_manager = unsafe { OwnedService(OpenSCManagerW(None, None, SC_MANAGER_CONNECT)?) };
    let service = unsafe {
        OwnedService(OpenServiceW(
            service_manager.0,
            w!("TrustedInstaller"),
            SERVICE_START | SERVICE_QUERY_STATUS,
        )?)
    };

    let mut status = unsafe { query_service_status(service.0)? };
    if status.dwCurrentState != SERVICE_RUNNING {
        // Ignore races: another caller may start the service between query/start.
        let _ = unsafe { StartServiceW(service.0, None) };

        for _ in 0..80 {
            status = unsafe { query_service_status(service.0)? };
            if status.dwCurrentState == SERVICE_RUNNING && status.dwProcessId != 0 {
                return Ok(status.dwProcessId);
            }
            if status.dwCurrentState != SERVICE_START_PENDING {
                // TrustedInstaller can briefly report intermediary states; keep polling.
            }
            thread::sleep(Duration::from_millis(250));
        }

        return Err("timed out waiting for TrustedInstaller service to start".into());
    }

    if status.dwProcessId == 0 {
        return Err("TrustedInstaller is running but did not report a process id".into());
    }

    Ok(status.dwProcessId)
}

unsafe fn query_service_status(
    service: SC_HANDLE,
) -> windows::core::Result<SERVICE_STATUS_PROCESS> {
    let mut status = SERVICE_STATUS_PROCESS::default();
    let mut bytes_needed = 0u32;
    let buffer = unsafe {
        std::slice::from_raw_parts_mut(
            (&mut status as *mut SERVICE_STATUS_PROCESS).cast::<u8>(),
            size_of::<SERVICE_STATUS_PROCESS>(),
        )
    };

    unsafe {
        QueryServiceStatusEx(
            service,
            SC_STATUS_PROCESS_INFO,
            Some(buffer),
            &mut bytes_needed,
        )?;
    }

    Ok(status)
}

fn wide_null(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}

fn make_command_line(exe: &str, args: &[&str]) -> String {
    std::iter::once(exe)
        .chain(args.iter().copied())
        .map(quote_windows_arg)
        .collect::<Vec<_>>()
        .join(" ")
}

fn quote_windows_arg(arg: &str) -> String {
    if !arg.is_empty() && !arg.chars().any(|ch| ch.is_whitespace() || ch == '"') {
        return arg.to_string();
    }

    let mut quoted = String::from("\"");
    let mut backslashes = 0;
    for ch in arg.chars() {
        match ch {
            '\\' => backslashes += 1,
            '"' => {
                quoted.push_str(&"\\".repeat(backslashes * 2 + 1));
                quoted.push('"');
                backslashes = 0;
            }
            _ => {
                quoted.push_str(&"\\".repeat(backslashes));
                backslashes = 0;
                quoted.push(ch);
            }
        }
    }
    quoted.push_str(&"\\".repeat(backslashes * 2));
    quoted.push('"');
    quoted
}
