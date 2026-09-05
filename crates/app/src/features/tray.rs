use tray_icon::{
    Icon, TrayIcon, TrayIconBuilder,
    menu::{Menu, MenuItem},
};

pub const WINDOW_WIDTH: f32 = 900.0;
pub const WINDOW_HEIGHT: f32 = 700.0;

const APP_LOGO_PNG_BYTES: &[u8] = if cfg!(debug_assertions) {
    include_bytes!("../../../../assets/app-logo-dev.png")
} else {
    include_bytes!("../../../../assets/app-logo.png")
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

#[repr(i32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[allow(dead_code)]
enum PreferredAppMode {
    Default = 0,
    AllowDark = 1,
    ForceDark = 2,
    ForceLight = 3,
}

#[allow(unsafe_code)]
pub fn init_theme_aware_menus() {
    unsafe {
        let uxtheme =
            windows_sys::Win32::System::LibraryLoader::LoadLibraryA(c"uxtheme.dll".as_ptr().cast());
        if !uxtheme.is_null() {
            let set_preferred_app_mode =
                windows_sys::Win32::System::LibraryLoader::GetProcAddress(uxtheme, 135 as _);
            if let Some(func) = set_preferred_app_mode {
                let func: unsafe extern "system" fn(i32) -> i32 = std::mem::transmute(func);
                func(PreferredAppMode::AllowDark as i32);
            }

            let flush_menu_themes =
                windows_sys::Win32::System::LibraryLoader::GetProcAddress(uxtheme, 136 as _);
            if let Some(func) = flush_menu_themes {
                let func: unsafe extern "system" fn() = std::mem::transmute(func);
                func();
            }
        }
    }
}

impl TrayManager {
    #[must_use]
    pub fn new() -> Self {
        init_theme_aware_menus();

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
                .with_menu_on_left_click(false)
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

#[allow(
    unsafe_code,
    clippy::too_many_lines,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation
)]
pub fn show_main_window() {
    unsafe {
        let mut target_hwnd: windows_sys::Win32::Foundation::HWND = std::ptr::null_mut();
        windows_sys::Win32::UI::WindowsAndMessaging::EnumWindows(
            Some(find_main_window_proc),
            &raw mut target_hwnd as windows_sys::Win32::Foundation::LPARAM,
        );
        if !target_hwnd.is_null() {
            let dpi = windows_sys::Win32::UI::HiDpi::GetDpiForWindow(target_hwnd);
            let dpi = if dpi == 0 { 96 } else { dpi };
            let scale = dpi as f32 / 96.0;
            let target_w = (WINDOW_WIDTH * scale).round() as i32;
            let target_h = (WINDOW_HEIGHT * scale).round() as i32;

            let is_iconic = windows_sys::Win32::UI::WindowsAndMessaging::IsIconic(target_hwnd) != 0;
            if is_iconic {
                windows_sys::Win32::UI::WindowsAndMessaging::ShowWindow(
                    target_hwnd,
                    windows_sys::Win32::UI::WindowsAndMessaging::SW_RESTORE,
                );
            } else {
                windows_sys::Win32::UI::WindowsAndMessaging::ShowWindow(
                    target_hwnd,
                    windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOW,
                );
            }

            let mut rect = windows_sys::Win32::Foundation::RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            };
            windows_sys::Win32::UI::WindowsAndMessaging::GetWindowRect(target_hwnd, &raw mut rect);
            let cur_w = rect.right - rect.left;
            let cur_h = rect.bottom - rect.top;

            let is_invalid_pos =
                rect.left <= -10000 || rect.top <= -10000 || cur_w <= 0 || cur_h <= 0;
            if is_invalid_pos {
                let hmonitor = windows_sys::Win32::Graphics::Gdi::MonitorFromWindow(
                    target_hwnd,
                    windows_sys::Win32::Graphics::Gdi::MONITOR_DEFAULTTONEAREST,
                );
                let mut mi = windows_sys::Win32::Graphics::Gdi::MONITORINFO {
                    cbSize: std::mem::size_of::<windows_sys::Win32::Graphics::Gdi::MONITORINFO>()
                        as u32,
                    rcMonitor: windows_sys::Win32::Foundation::RECT {
                        left: 0,
                        top: 0,
                        right: 0,
                        bottom: 0,
                    },
                    rcWork: windows_sys::Win32::Foundation::RECT {
                        left: 0,
                        top: 0,
                        right: 0,
                        bottom: 0,
                    },
                    dwFlags: 0,
                };
                if windows_sys::Win32::Graphics::Gdi::GetMonitorInfoW(hmonitor, &raw mut mi) != 0 {
                    let work_w = mi.rcWork.right - mi.rcWork.left;
                    let work_h = mi.rcWork.bottom - mi.rcWork.top;
                    let x = mi.rcWork.left + (work_w - target_w).max(0) / 2;
                    let y = mi.rcWork.top + (work_h - target_h).max(0) / 2;
                    windows_sys::Win32::UI::WindowsAndMessaging::SetWindowPos(
                        target_hwnd,
                        0 as _,
                        x,
                        y,
                        target_w,
                        target_h,
                        windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOZORDER
                            | windows_sys::Win32::UI::WindowsAndMessaging::SWP_FRAMECHANGED
                            | windows_sys::Win32::UI::WindowsAndMessaging::SWP_SHOWWINDOW,
                    );
                } else {
                    windows_sys::Win32::UI::WindowsAndMessaging::SetWindowPos(
                        target_hwnd,
                        0 as _,
                        100,
                        100,
                        target_w,
                        target_h,
                        windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOZORDER
                            | windows_sys::Win32::UI::WindowsAndMessaging::SWP_FRAMECHANGED
                            | windows_sys::Win32::UI::WindowsAndMessaging::SWP_SHOWWINDOW,
                    );
                }
            } else if cur_w != target_w || cur_h != target_h {
                windows_sys::Win32::UI::WindowsAndMessaging::SetWindowPos(
                    target_hwnd,
                    0 as _,
                    rect.left,
                    rect.top,
                    target_w,
                    target_h,
                    windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOMOVE
                        | windows_sys::Win32::UI::WindowsAndMessaging::SWP_NOZORDER
                        | windows_sys::Win32::UI::WindowsAndMessaging::SWP_FRAMECHANGED
                        | windows_sys::Win32::UI::WindowsAndMessaging::SWP_SHOWWINDOW,
                );
            }

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
