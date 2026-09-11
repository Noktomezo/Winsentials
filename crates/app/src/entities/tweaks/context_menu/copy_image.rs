#![allow(unsafe_code)]

use std::path::{Path, PathBuf};

use crate::shared::shell_notify::notify_shell_change;

const REG_IMAGE_MENU: &str =
    r"Software\Classes\SystemFileAssociations\image\shell\Winsentials.CopyImage";
const REG_IMAGE_COMMAND: &str =
    r"Software\Classes\SystemFileAssociations\image\shell\Winsentials.CopyImage\command";

fn current_exe() -> Option<PathBuf> {
    std::env::current_exe().ok()
}

fn command(exe: &Path) -> String {
    format!(r#""{}" --copy-image "%1""#, exe.display())
}

fn remove_legacy_files() {
    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA").map(PathBuf::from) {
        let shell_dir = local_app_data.join("Winsentials").join("shell");
        let _ = std::fs::remove_file(shell_dir.join("copy-image.ps1"));
        let _ = std::fs::remove_file(shell_dir.join("copy-image.vbs"));
        let scripts_dir = local_app_data.join("Winsentials").join("scripts");
        let _ = std::fs::remove_file(scripts_dir.join("copy-image.ps1"));
    }
}

#[must_use]
pub fn is_copy_image_applied() -> bool {
    #[cfg(target_os = "windows")]
    {
        current_exe().is_some_and(|exe| {
            let expected = command(&exe);
            windows_registry::CURRENT_USER
                .open(REG_IMAGE_COMMAND)
                .is_ok_and(|key| {
                    key.get_string("")
                        .is_ok_and(|value| value.eq_ignore_ascii_case(&expected))
                })
        })
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_copy_image(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let exe =
            current_exe().ok_or_else(|| "Failed to get current executable path".to_string())?;
        let cmd = command(&exe);

        if applied {
            let menu = windows_registry::CURRENT_USER
                .create(REG_IMAGE_MENU)
                .map_err(|error| format!("Failed to create image menu item: {error}"))?;
            menu.set_string("MUIVerb", rust_i18n::t!("tweaks.copy_image_menu_item"))
                .and_then(|()| menu.set_string("Icon", "imageres.dll,-5302"))
                .map_err(|error| format!("Failed to configure image menu item: {error}"))?;

            windows_registry::CURRENT_USER
                .create(REG_IMAGE_COMMAND)
                .and_then(|key| key.set_string("", &cmd))
                .map_err(|error| format!("Failed to configure image menu command: {error}"))?;

            remove_legacy_files();
        } else {
            let _ = windows_registry::CURRENT_USER.remove_tree(REG_IMAGE_MENU);
            remove_legacy_files();
        }

        notify_shell_change();
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = applied;
        Ok(())
    }
}

/// Headless CLI handler for copying an image file to the Windows clipboard natively.
/// Executed via `Winsentials.exe --copy-image "<path>"`.
/// Completes in ~5ms without launching GPUI, PowerShell, or `VBScript`.
pub fn handle_cli_copy_image(file_path: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        copy_image_to_clipboard_windows(file_path)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = file_path;
        false
    }
}

#[cfg(target_os = "windows")]
fn copy_image_to_clipboard_windows(file_path: &str) -> bool {
    let path = Path::new(file_path);
    if !path.exists() {
        return false;
    }

    let Ok(file_bytes) = std::fs::read(path) else {
        return false;
    };

    let Ok(img) = image::load_from_memory(&file_bytes) else {
        return false;
    };

    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    if width == 0 || height == 0 {
        return false;
    }

    // 1. Prepare PNG bytes (preserve alpha channel for Discord, Telegram, web browsers)
    let png_bytes = if file_bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        file_bytes
    } else {
        let mut cursor = std::io::Cursor::new(Vec::new());
        if img.write_to(&mut cursor, image::ImageFormat::Png).is_ok() {
            cursor.into_inner()
        } else {
            Vec::new()
        }
    };

    // 2. Prepare CF_DIBV5 (standard 32-bit BGRA with transparency for Windows apps)
    let dib_bytes = build_dibv5_bytes(&rgba, width, height);

    // 3. Prepare CF_HDROP (allows pasting as file into File Explorer)
    let hdrop_bytes = build_hdrop_bytes(path);

    // 4. Open clipboard and write data
    write_to_windows_clipboard(&png_bytes, &dib_bytes, &hdrop_bytes)
}

#[cfg(target_os = "windows")]
#[allow(clippy::struct_field_names)]
#[repr(C, packed)]
struct BitmapV5Header {
    b_v5_size: u32,
    b_v5_width: i32,
    b_v5_height: i32,
    b_v5_planes: u16,
    b_v5_bit_count: u16,
    b_v5_compression: u32,
    b_v5_size_image: u32,
    b_v5_x_pels_per_meter: i32,
    b_v5_y_pels_per_meter: i32,
    b_v5_clr_used: u32,
    b_v5_clr_important: u32,
    b_v5_red_mask: u32,
    b_v5_green_mask: u32,
    b_v5_blue_mask: u32,
    b_v5_alpha_mask: u32,
    b_v5_cs_type: u32,
    b_v5_endpoints: [u32; 9],
    b_v5_gamma_red: u32,
    b_v5_gamma_green: u32,
    b_v5_gamma_blue: u32,
    b_v5_intent: u32,
    b_v5_profile_data: u32,
    b_v5_profile_size: u32,
    b_v5_reserved: u32,
}

#[cfg(target_os = "windows")]
#[allow(clippy::cast_possible_wrap)]
fn build_dibv5_bytes(rgba: &image::RgbaImage, width: u32, height: u32) -> Vec<u8> {
    let header_size = std::mem::size_of::<BitmapV5Header>();
    let pixel_bytes = (width as usize) * (height as usize) * 4;
    let mut buf = Vec::with_capacity(header_size + pixel_bytes);

    let header = BitmapV5Header {
        b_v5_size: header_size as u32,
        b_v5_width: width as i32,
        b_v5_height: height as i32, // Positive = bottom-up DIB
        b_v5_planes: 1,
        b_v5_bit_count: 32,
        b_v5_compression: 3, // BI_BITFIELDS
        b_v5_size_image: pixel_bytes as u32,
        b_v5_x_pels_per_meter: 0,
        b_v5_y_pels_per_meter: 0,
        b_v5_clr_used: 0,
        b_v5_clr_important: 0,
        b_v5_red_mask: 0x00FF_0000,
        b_v5_green_mask: 0x0000_FF00,
        b_v5_blue_mask: 0x0000_00FF,
        b_v5_alpha_mask: 0xFF00_0000,
        b_v5_cs_type: 0x7352_4742, // 'sRGB'
        b_v5_endpoints: [0; 9],
        b_v5_gamma_red: 0,
        b_v5_gamma_green: 0,
        b_v5_gamma_blue: 0,
        b_v5_intent: 4, // LCS_GM_IMAGES
        b_v5_profile_data: 0,
        b_v5_profile_size: 0,
        b_v5_reserved: 0,
    };

    let header_slice =
        unsafe { std::slice::from_raw_parts((&raw const header).cast::<u8>(), header_size) };
    buf.extend_from_slice(header_slice);

    // Bottom-up BGRA rows
    for y in (0..height).rev() {
        for x in 0..width {
            let pixel = rgba.get_pixel(x, y);
            let [r, g, b, a] = pixel.0;
            buf.extend_from_slice(&[b, g, r, a]);
        }
    }

    buf
}

#[cfg(target_os = "windows")]
#[repr(C)]
struct DropFiles {
    p_files: u32,
    pt_x: i32,
    pt_y: i32,
    f_nc: i32,
    f_wide: i32,
}

#[cfg(target_os = "windows")]
fn build_hdrop_bytes(path: &Path) -> Vec<u8> {
    let abs_path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let path_str = abs_path.display().to_string();
    // Strip Windows extended-length prefix if present (\\?\)
    let clean_path = path_str.strip_prefix(r"\\?\").unwrap_or(&path_str);

    let mut utf16: Vec<u16> = clean_path.encode_utf16().collect();
    utf16.push(0); // null terminator for string
    utf16.push(0); // double null terminator for DROPFILES list

    let header_size = std::mem::size_of::<DropFiles>();
    let header = DropFiles {
        p_files: header_size as u32,
        pt_x: 0,
        pt_y: 0,
        f_nc: 0,
        f_wide: 1, // Unicode
    };

    let mut buf = Vec::with_capacity(header_size + utf16.len() * 2);
    let header_slice =
        unsafe { std::slice::from_raw_parts((&raw const header).cast::<u8>(), header_size) };
    buf.extend_from_slice(header_slice);
    for u in utf16 {
        buf.extend_from_slice(&u.to_le_bytes());
    }
    buf
}

#[cfg(target_os = "windows")]
#[link(name = "user32")]
unsafe extern "system" {
    fn MessageBeep(utype: u32) -> i32;
}

#[cfg(target_os = "windows")]
const MB_ICONASTERISK: u32 = 0x0000_0040;
#[cfg(target_os = "windows")]
const CF_HDROP: u32 = 15;
#[cfg(target_os = "windows")]
const CF_DIBV5: u32 = 17;

#[cfg(target_os = "windows")]
fn write_to_windows_clipboard(png_bytes: &[u8], dib_bytes: &[u8], hdrop_bytes: &[u8]) -> bool {
    use windows_sys::Win32::System::DataExchange::{
        CloseClipboard, EmptyClipboard, OpenClipboard, RegisterClipboardFormatW,
    };

    // Retry OpenClipboard up to 10 times with 50ms intervals
    let mut opened = false;
    for _ in 0..10 {
        if unsafe { OpenClipboard(std::ptr::null_mut()) } != 0 {
            opened = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    if !opened {
        return false;
    }

    unsafe {
        EmptyClipboard();

        // 1. PNG format (for Telegram, Discord, browsers)
        if !png_bytes.is_empty() {
            let png_format = RegisterClipboardFormatW(windows_sys::w!("PNG"));
            if png_format != 0 {
                copy_bytes_to_clipboard(png_format, png_bytes);
            }
        }

        // 2. CF_DIBV5 (format 17, 32-bit bitmap with alpha)
        copy_bytes_to_clipboard(CF_DIBV5, dib_bytes);

        // 3. CF_HDROP (format 15, allows pasting file into Explorer)
        copy_bytes_to_clipboard(CF_HDROP, hdrop_bytes);

        CloseClipboard();
        MessageBeep(MB_ICONASTERISK);
    }

    true
}

#[cfg(target_os = "windows")]
unsafe fn copy_bytes_to_clipboard(format: u32, data: &[u8]) {
    use windows_sys::Win32::System::DataExchange::SetClipboardData;
    use windows_sys::Win32::System::Memory::{
        GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalUnlock,
    };

    let hmem = unsafe { GlobalAlloc(GMEM_MOVEABLE, data.len()) };
    if hmem.is_null() {
        return;
    }
    let ptr = unsafe { GlobalLock(hmem) };
    if ptr.is_null() {
        return;
    }
    unsafe {
        std::ptr::copy_nonoverlapping(data.as_ptr(), ptr.cast::<u8>(), data.len());
        GlobalUnlock(hmem);
        SetClipboardData(format, hmem);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_formats_with_copy_image_flag() {
        let exe = Path::new(r"C:\Program Files\Winsentials\Winsentials.exe");
        assert_eq!(
            command(exe),
            r#""C:\Program Files\Winsentials\Winsentials.exe" --copy-image "%1""#
        );
    }
}
