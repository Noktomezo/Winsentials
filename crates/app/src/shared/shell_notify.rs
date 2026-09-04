#![allow(unsafe_code)]

use windows_sys::Win32::UI::Shell::{SHCNE_ASSOCCHANGED, SHCNF_IDLIST, SHChangeNotify};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    HWND_BROADCAST, SMTO_ABORTIFHUNG, SendMessageTimeoutW, WM_SETTINGCHANGE,
};

pub fn notify_shell_change() {
    unsafe {
        // 1. Notify Windows Shell about file associations, icon overlays & shell namespaces asynchronously
        #[allow(clippy::cast_possible_wrap)]
        SHChangeNotify(
            SHCNE_ASSOCCHANGED as i32,
            SHCNF_IDLIST,
            std::ptr::null(),
            std::ptr::null(),
        );

        // 2. Broadcast WM_SETTINGCHANGE to top-level windows without blocking
        let mut result = 0;
        let mut wide_str: Vec<u16> = "Shell".encode_utf16().collect();
        wide_str.push(0);
        SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            0,
            wide_str.as_ptr() as isize,
            SMTO_ABORTIFHUNG,
            100,
            &raw mut result,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_notify_shell_change_does_not_panic() {
        notify_shell_change();
    }
}
