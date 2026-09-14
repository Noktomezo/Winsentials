#![allow(unsafe_code)]

#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::{
    ERROR_SERVICE_DOES_NOT_EXIST, ERROR_SERVICE_NOT_ACTIVE, GetLastError,
};

#[cfg(target_os = "windows")]
const HRESULT_FILE_NOT_FOUND: i32 = -2_147_024_894;

#[must_use]
pub fn is_service_disabled(service_name: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        let reg_path = format!(r"SYSTEM\CurrentControlSet\Services\{service_name}");
        windows_registry::LOCAL_MACHINE
            .open(&reg_path)
            .ok()
            .and_then(|key| key.get_u32("Start").ok())
            == Some(4)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = service_name;
        false
    }
}

pub fn set_service_disabled(
    service_name: &str,
    disabled: bool,
    default_start: u32,
) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::System::Services::{
            ChangeServiceConfigW, CloseServiceHandle, ControlService, OpenSCManagerW, OpenServiceW,
            SC_MANAGER_CONNECT, SERVICE_CHANGE_CONFIG, SERVICE_CONTROL_STOP, SERVICE_NO_CHANGE,
            SERVICE_START_TYPE, SERVICE_STATUS, SERVICE_STOP,
        };

        let wide_name: Vec<u16> = service_name.encode_utf16().chain(Some(0)).collect();
        let new_start: SERVICE_START_TYPE = if disabled { 4 } else { default_start };

        unsafe {
            let scm = OpenSCManagerW(std::ptr::null(), std::ptr::null(), SC_MANAGER_CONNECT);
            if scm.is_null() {
                let err = GetLastError();
                return Err(format!(
                    "Failed to connect to Service Control Manager for '{service_name}': error code {err}"
                ));
            }

            let desired_access = if disabled {
                SERVICE_CHANGE_CONFIG | SERVICE_STOP
            } else {
                SERVICE_CHANGE_CONFIG
            };

            let mut service = OpenServiceW(scm, wide_name.as_ptr(), desired_access);
            if service.is_null() && disabled {
                // Fallback to config change only if stopping rights are restricted.
                service = OpenServiceW(scm, wide_name.as_ptr(), SERVICE_CHANGE_CONFIG);
            }

            if service.is_null() {
                let err = GetLastError();
                CloseServiceHandle(scm);
                if err == ERROR_SERVICE_DOES_NOT_EXIST {
                    return Ok(());
                }
                return Err(format!(
                    "Failed to open service '{service_name}': error code {err}"
                ));
            }

            let config_res = ChangeServiceConfigW(
                service,
                SERVICE_NO_CHANGE,
                new_start,
                SERVICE_NO_CHANGE,
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null_mut(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null(),
            );
            if config_res == 0 {
                let err = GetLastError();
                CloseServiceHandle(service);
                CloseServiceHandle(scm);
                return Err(format!(
                    "Failed to change config for service '{service_name}': error code {err}"
                ));
            }

            if disabled {
                let mut status: SERVICE_STATUS = std::mem::zeroed();
                let ctrl_res = ControlService(service, SERVICE_CONTROL_STOP, &raw mut status);
                if ctrl_res == 0 {
                    let err = GetLastError();
                    if err != ERROR_SERVICE_NOT_ACTIVE {
                        eprintln!(
                            "Note: service '{service_name}' set to disabled, but stop control returned code {err}"
                        );
                    }
                }
            }

            CloseServiceHandle(service);
            CloseServiceHandle(scm);
        }
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (service_name, disabled, default_start);
        Ok(())
    }
}

#[cfg(target_os = "windows")]
pub fn set_reg_u32(
    key: &windows_registry::Key,
    key_path: &str,
    name: &str,
    val: u32,
) -> Result<(), String> {
    key.set_u32(name, val)
        .map_err(|e| format!("Failed to set registry value '{name}' in '{key_path}': {e}"))
}

#[cfg(target_os = "windows")]
pub fn remove_reg_value(
    key: &windows_registry::Key,
    key_path: &str,
    name: &str,
) -> Result<(), String> {
    match key.remove_value(name) {
        Ok(()) => Ok(()),
        Err(e) if e.code().0 == HRESULT_FILE_NOT_FOUND => Ok(()),
        Err(e) => Err(format!(
            "Failed to remove registry value '{name}' from '{key_path}': {e}"
        )),
    }
}
