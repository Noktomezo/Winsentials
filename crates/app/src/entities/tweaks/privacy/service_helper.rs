#![allow(unsafe_code)]

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

        let reg_path = format!(r"SYSTEM\CurrentControlSet\Services\{service_name}");
        let new_start: SERVICE_START_TYPE = if disabled { 4 } else { default_start };

        if let Ok(key) = windows_registry::LOCAL_MACHINE.open(&reg_path) {
            let _ = key.set_u32("Start", new_start);
        }

        let wide_name: Vec<u16> = service_name.encode_utf16().chain(Some(0)).collect();
        unsafe {
            let scm = OpenSCManagerW(std::ptr::null(), std::ptr::null(), SC_MANAGER_CONNECT);
            if !scm.is_null() {
                let service = OpenServiceW(
                    scm,
                    wide_name.as_ptr(),
                    SERVICE_CHANGE_CONFIG | SERVICE_STOP,
                );
                if !service.is_null() {
                    let _ = ChangeServiceConfigW(
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
                    if disabled {
                        let mut status: SERVICE_STATUS = std::mem::zeroed();
                        let _ = ControlService(service, SERVICE_CONTROL_STOP, &raw mut status);
                    }
                    CloseServiceHandle(service);
                }
                CloseServiceHandle(scm);
            }
        }
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (service_name, disabled, default_start);
        Ok(())
    }
}
