use tray_icon::{
    Icon, TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuItem},
};

const APP_LOGO_PNG_BYTES: &[u8] = if cfg!(debug_assertions) {
    include_bytes!("../../assets/app-logo-dev.png")
} else {
    include_bytes!("../../assets/app-logo.png")
};

const APP_NAME: &str = if cfg!(debug_assertions) {
    "Winsentials (Dev)"
} else {
    "Winsentials"
};

fn create_tray_icon() -> Result<Icon, tray_icon::BadIcon> {
    if let Ok(img) =
        image::load_from_memory_with_format(APP_LOGO_PNG_BYTES, image::ImageFormat::Png)
    {
        let resized = img.resize_exact(32, 32, image::imageops::FilterType::Lanczos3);
        let rgba = resized.to_rgba8();
        let (width, height) = rgba.dimensions();
        if let Ok(icon) = Icon::from_rgba(rgba.into_raw(), width, height) {
            return Ok(icon);
        }
    }
    Icon::from_rgba(vec![0u8; 32 * 32 * 4], 32, 32)
}

pub struct TrayManager {
    _tray_icon: Option<TrayIcon>,
    pub open_item_id: String,
    pub quit_item_id: String,
}

impl Default for TrayManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TrayManager {
    #[must_use]
    pub fn new() -> Self {
        let menu = Menu::new();
        let open_title = if cfg!(debug_assertions) {
            "Открыть Winsentials (Dev)"
        } else {
            "Открыть Winsentials"
        };
        let open_item = MenuItem::new(open_title, true, None);
        let quit_item = MenuItem::new("Выход", true, None);

        let open_item_id = open_item.id().as_ref().to_string();
        let quit_item_id = quit_item.id().as_ref().to_string();

        let _ = menu.append(&open_item);
        let _ = menu.append(&quit_item);

        let icon = create_tray_icon().ok();

        let tray_icon = if let Some(icon) = icon {
            TrayIconBuilder::new()
                .with_menu(Box::new(menu))
                .with_tooltip(APP_NAME)
                .with_icon(icon)
                .build()
                .ok()
        } else {
            None
        };

        Self {
            _tray_icon: tray_icon,
            open_item_id,
            quit_item_id,
        }
    }
}

#[allow(unsafe_code)]
unsafe extern "system" fn find_main_window_proc(
    hwnd: windows_sys::Win32::Foundation::HWND,
    lparam: windows_sys::Win32::Foundation::LPARAM,
) -> windows_sys::Win32::Foundation::BOOL {
    let mut pid = 0;
    unsafe {
        windows_sys::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId(hwnd, &raw mut pid);
        if pid == windows_sys::Win32::System::Threading::GetCurrentProcessId() {
            let mut title = [0u16; 256];
            let len = windows_sys::Win32::UI::WindowsAndMessaging::GetWindowTextW(
                hwnd,
                title.as_mut_ptr(),
                256,
            );
            if len > 0 {
                let target = lparam as *mut windows_sys::Win32::Foundation::HWND;
                if !target.is_null() {
                    *target = hwnd;
                    return 0;
                }
            }
        }
    }
    1
}

#[allow(unsafe_code)]
pub fn show_main_window() {
    unsafe {
        let mut target_hwnd: windows_sys::Win32::Foundation::HWND = std::ptr::null_mut();
        windows_sys::Win32::UI::WindowsAndMessaging::EnumWindows(
            Some(find_main_window_proc),
            &raw mut target_hwnd as windows_sys::Win32::Foundation::LPARAM,
        );
        if !target_hwnd.is_null() {
            windows_sys::Win32::UI::WindowsAndMessaging::ShowWindow(
                target_hwnd,
                windows_sys::Win32::UI::WindowsAndMessaging::SW_RESTORE,
            );
            windows_sys::Win32::UI::WindowsAndMessaging::SetForegroundWindow(target_hwnd);
        }
    }
}

#[allow(unsafe_code)]
pub fn hide_main_window() {
    unsafe {
        let mut target_hwnd: windows_sys::Win32::Foundation::HWND = std::ptr::null_mut();
        windows_sys::Win32::UI::WindowsAndMessaging::EnumWindows(
            Some(find_main_window_proc),
            &raw mut target_hwnd as windows_sys::Win32::Foundation::LPARAM,
        );
        if !target_hwnd.is_null() {
            windows_sys::Win32::UI::WindowsAndMessaging::ShowWindow(
                target_hwnd,
                windows_sys::Win32::UI::WindowsAndMessaging::SW_HIDE,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tray_icon_loads_from_png_bytes() {
        let icon_res = create_tray_icon();
        assert!(icon_res.is_ok(), "Tray icon should load from app-logo.png");
    }
}
