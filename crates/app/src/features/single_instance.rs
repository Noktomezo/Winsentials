#![allow(unsafe_code)]

use windows_sys::Win32::Foundation::{
    BOOL, CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE, HWND, LPARAM,
};
use windows_sys::Win32::System::Threading::CreateMutexW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    BringWindowToTop, EnumWindows, GetWindowTextW, SW_RESTORE, SW_SHOW, SetForegroundWindow,
    ShowWindow,
};

pub struct SingleInstanceGuard {
    handle: HANDLE,
}

// Safety: The HANDLE is an OS mutex handle managed by this struct and closed on Drop.
unsafe impl Send for SingleInstanceGuard {}
unsafe impl Sync for SingleInstanceGuard {}

impl Drop for SingleInstanceGuard {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            unsafe {
                CloseHandle(self.handle);
            }
        }
    }
}

#[must_use]
pub fn try_acquire_named(mutex_name: &str) -> Option<SingleInstanceGuard> {
    let mut wide_name: Vec<u16> = mutex_name.encode_utf16().collect();
    if !wide_name.ends_with(&[0]) {
        wide_name.push(0);
    }

    unsafe {
        let handle = CreateMutexW(std::ptr::null_mut(), 1, wide_name.as_ptr());
        if handle.is_null() {
            return None;
        }

        if GetLastError() == ERROR_ALREADY_EXISTS {
            CloseHandle(handle);
            return None;
        }

        Some(SingleInstanceGuard { handle })
    }
}

#[must_use]
pub fn try_acquire_single_instance() -> Option<SingleInstanceGuard> {
    let mutex_name = if cfg!(debug_assertions) {
        "Local\\Winsentials_App_SingleInstance_Dev"
    } else {
        "Local\\Winsentials_App_SingleInstance_Release"
    };

    let guard = try_acquire_named(mutex_name);
    if guard.is_none() {
        activate_existing_instance();
    }
    guard
}

unsafe extern "system" fn find_existing_window_proc(hwnd: HWND, _lparam: LPARAM) -> BOOL {
    let target_title = if cfg!(debug_assertions) {
        "Winsentials (Dev)"
    } else {
        "Winsentials"
    };

    let mut buf = [0u16; 256];
    let len = unsafe { GetWindowTextW(hwnd, buf.as_mut_ptr(), 256) };
    if len > 0 {
        let len_usize = usize::try_from(len).unwrap_or(0);
        let title = String::from_utf16_lossy(&buf[..len_usize]);
        if title == target_title {
            unsafe {
                ShowWindow(hwnd, SW_RESTORE);
                ShowWindow(hwnd, SW_SHOW);
                BringWindowToTop(hwnd);
                SetForegroundWindow(hwnd);
            }
            return 0; // Stop enumeration once the target window is found and activated
        }
    }
    1 // Continue enumeration
}

pub fn activate_existing_instance() {
    unsafe {
        EnumWindows(Some(find_existing_window_proc), 0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_instance_guard_lifecycle() {
        let test_name = "Local\\Winsentials_Unique_Test_Mutex";
        let guard1 = try_acquire_named(test_name);
        assert!(
            guard1.is_some(),
            "First instance acquisition should succeed"
        );

        let guard2 = try_acquire_named(test_name);
        assert!(guard2.is_none(), "Second instance acquisition should fail");

        drop(guard1);

        let guard3 = try_acquire_named(test_name);
        assert!(guard3.is_some(), "Acquisition after drop should succeed");
    }
}
