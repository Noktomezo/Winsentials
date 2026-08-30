use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use crate::entities::startup::vendor::extract_clean_exe_path;

/// Resolves or extracts a PNG icon for the given executable / target path and caches it.
#[must_use]
pub fn resolve_entry_icon(target_path: Option<&str>, command: Option<&str>) -> Option<String> {
    let clean_path = target_path
        .and_then(|p| {
            if Path::new(p).exists() {
                Some(PathBuf::from(p))
            } else {
                extract_clean_exe_path(p)
            }
        })
        .or_else(|| command.and_then(extract_clean_exe_path))?;

    if !clean_path.exists() {
        return None;
    }

    #[cfg(target_os = "windows")]
    {
        extract_windows_icon(&clean_path)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = clean_path;
        None
    }
}

#[cfg(target_os = "windows")]
fn get_cache_dir() -> PathBuf {
    let dir = std::env::temp_dir().join("winsentials_icon_cache");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

#[cfg(target_os = "windows")]
#[allow(
    unsafe_code,
    clippy::too_many_lines,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
fn extract_windows_icon(path: &Path) -> Option<String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null_mut;
    use windows_sys::Win32::Graphics::Gdi::{
        BI_RGB, BITMAP, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, DeleteObject, GetDC,
        GetDIBits, GetObjectW, RGBQUAD, ReleaseDC,
    };
    use windows_sys::Win32::UI::Shell::{SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON, SHGetFileInfoW};
    use windows_sys::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, ICONINFO};

    let mut hasher = DefaultHasher::new();
    path.to_string_lossy().to_lowercase().hash(&mut hasher);
    let hash_str = format!("{:016x}", hasher.finish());

    let cache_dir = get_cache_dir();
    let cache_file = cache_dir.join(format!("{hash_str}.png"));

    if cache_file.exists() && cache_file.metadata().map_or(0, |m| m.len()) > 0 {
        return Some(cache_file.to_string_lossy().to_string());
    }

    let mut wide: Vec<u16> = OsStr::new(path).encode_wide().collect();
    wide.push(0);

    let mut shfi: SHFILEINFOW = unsafe { std::mem::zeroed() };
    let res = unsafe {
        SHGetFileInfoW(
            wide.as_ptr(),
            0,
            &raw mut shfi,
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        )
    };

    if res == 0 || shfi.hIcon.is_null() {
        return None;
    }

    let hicon = shfi.hIcon;

    let mut icon_info: ICONINFO = unsafe { std::mem::zeroed() };
    if unsafe { GetIconInfo(hicon, &raw mut icon_info) } == 0 {
        unsafe {
            DestroyIcon(hicon);
        }
        return None;
    }

    let mut bmp: BITMAP = unsafe { std::mem::zeroed() };
    if unsafe {
        GetObjectW(
            icon_info.hbmColor,
            std::mem::size_of::<BITMAP>() as i32,
            (&raw mut bmp).cast(),
        )
    } == 0
    {
        unsafe {
            if !icon_info.hbmColor.is_null() {
                DeleteObject(icon_info.hbmColor);
            }
            if !icon_info.hbmMask.is_null() {
                DeleteObject(icon_info.hbmMask);
            }
            DestroyIcon(hicon);
        }
        return None;
    }

    let width = bmp.bmWidth;
    let height = bmp.bmHeight;
    if width <= 0 || height <= 0 {
        unsafe {
            if !icon_info.hbmColor.is_null() {
                DeleteObject(icon_info.hbmColor);
            }
            if !icon_info.hbmMask.is_null() {
                DeleteObject(icon_info.hbmMask);
            }
            DestroyIcon(hicon);
        }
        return None;
    }

    let pixel_count = (width * height) as usize;
    let mut bgra_buf: Vec<u8> = vec![0u8; pixel_count * 4];

    let mut bi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: width,
            biHeight: -height, // Top-down
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB,
            biSizeImage: 0,
            biXPelsPerMeter: 0,
            biYPelsPerMeter: 0,
            biClrUsed: 0,
            biClrImportant: 0,
        },
        bmiColors: [RGBQUAD {
            rgbBlue: 0,
            rgbGreen: 0,
            rgbRed: 0,
            rgbReserved: 0,
        }],
    };

    let hdc = unsafe { GetDC(null_mut()) };
    let dib_res = unsafe {
        GetDIBits(
            hdc,
            icon_info.hbmColor,
            0,
            height as u32,
            bgra_buf.as_mut_ptr().cast(),
            &raw mut bi,
            DIB_RGB_COLORS,
        )
    };
    unsafe {
        ReleaseDC(null_mut(), hdc);
        if !icon_info.hbmColor.is_null() {
            DeleteObject(icon_info.hbmColor);
        }
        if !icon_info.hbmMask.is_null() {
            DeleteObject(icon_info.hbmMask);
        }
        DestroyIcon(hicon);
    }

    if dib_res == 0 {
        return None;
    }

    // Convert BGRA to RGBA and ensure alpha channel isn't completely zero
    let mut has_non_zero_alpha = false;
    for chunk in bgra_buf.chunks_exact(4) {
        if chunk[3] > 0 {
            has_non_zero_alpha = true;
            break;
        }
    }

    let mut rgba_buf = Vec::with_capacity(pixel_count * 4);
    for chunk in bgra_buf.chunks_exact(4) {
        let b = chunk[0];
        let g = chunk[1];
        let r = chunk[2];
        let mut a = chunk[3];
        if !has_non_zero_alpha && (r > 0 || g > 0 || b > 0) {
            a = 255;
        }
        rgba_buf.push(r);
        rgba_buf.push(g);
        rgba_buf.push(b);
        rgba_buf.push(a);
    }

    if image::save_buffer(
        &cache_file,
        &rgba_buf,
        width as u32,
        height as u32,
        image::ExtendedColorType::Rgba8,
    )
    .is_ok()
    {
        Some(cache_file.to_string_lossy().to_string())
    } else {
        None
    }
}
