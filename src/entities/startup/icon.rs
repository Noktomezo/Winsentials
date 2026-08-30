use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use crate::entities::startup::vendor::extract_clean_exe_path;

/// Resolves or extracts a PNG icon for the given executable / target path and caches it.
#[must_use]
pub fn resolve_entry_icon(target_path: Option<&str>, command: Option<&str>) -> Option<PathBuf> {
    let clean_path = target_path
        .and_then(|p| {
            let pb = PathBuf::from(p);
            if pb.exists() {
                Some(normalize_path(&pb))
            } else {
                extract_clean_exe_path(p).map(|ep| normalize_path(&ep))
            }
        })
        .or_else(|| {
            command.and_then(|c| extract_clean_exe_path(c).map(|ep| normalize_path(&ep)))
        })?;

    #[cfg(target_os = "windows")]
    {
        // 1. Direct icon extraction from target executable
        if clean_path.exists() {
            if let Some(icon) = extract_direct_icon(&clean_path) {
                return Some(icon);
            }
            // 2. Sibling GUI executable discovery in the same directory (for CLI helpers / services)
            if let Some(sibling_icon) = find_sibling_icon(&clean_path) {
                return Some(sibling_icon);
            }
        }

        // 3. Fallback: if it's powershell running a script, resolve PowerShell icon
        let name_lower = clean_path
            .file_name()
            .map_or(String::new(), |n| n.to_string_lossy().to_lowercase());
        if name_lower == "powershell.exe" || name_lower == "pwsh.exe" {
            let ps_path =
                PathBuf::from(r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe");
            if ps_path.exists() {
                if let Some(icon) = extract_direct_icon(&ps_path) {
                    return Some(icon);
                }
            }
        }
        None
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = clean_path;
        None
    }
}

fn normalize_path(path: &Path) -> PathBuf {
    if let Ok(canonical) = path.canonicalize() {
        let s = canonical.to_string_lossy();
        if let Some(stripped) = s.strip_prefix(r"\\?\") {
            PathBuf::from(stripped)
        } else {
            canonical
        }
    } else {
        path.to_path_buf()
    }
}

#[cfg(target_os = "windows")]
fn get_cache_dir() -> PathBuf {
    let dir = std::env::temp_dir().join("winsentials_icon_cache");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

#[cfg(target_os = "windows")]
fn find_sibling_icon(exe_path: &Path) -> Option<PathBuf> {
    let parent = exe_path.parent()?;
    let current_name = exe_path.file_name()?.to_string_lossy().to_lowercase();
    let current_stem = exe_path.file_stem()?.to_string_lossy().to_lowercase();
    let folder_name = parent.file_name()?.to_string_lossy().to_lowercase();

    let Ok(entries) = std::fs::read_dir(parent) else {
        return None;
    };

    let mut candidate_exes = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext.eq_ignore_ascii_case("exe") {
                    let name = path.file_name().unwrap().to_string_lossy().to_lowercase();
                    if name != current_name {
                        candidate_exes.push(path);
                    }
                }
            }
        }
    }

    // Sort candidate executables by relevance:
    // 1. Matches folder name (e.g. AmneziaVPN.exe in AmneziaVPN folder)
    // 2. Base stem of service/cli (e.g. AmneziaVPN.exe for AmneziaVPN-service.exe)
    // 3. Known main names (e.g. RadeonSoftware.exe, Launcher.exe, App.exe)
    candidate_exes.sort_by_key(|p| {
        let stem = p.file_stem().unwrap().to_string_lossy().to_lowercase();
        if stem == folder_name {
            0
        } else if current_stem.starts_with(&stem) || stem.starts_with(&current_stem) {
            1
        } else if stem.contains("radeon")
            || stem.contains("control")
            || stem.contains("main")
            || stem.contains("launcher")
            || stem.contains("gui")
        {
            2
        } else {
            10
        }
    });

    for cand in candidate_exes {
        if let Some(icon) = extract_direct_icon(&cand) {
            return Some(icon);
        }
    }
    None
}

#[cfg(target_os = "windows")]
#[allow(unsafe_code, clippy::cast_possible_truncation)]
fn extract_direct_icon(path: &Path) -> Option<PathBuf> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null_mut;
    use windows_sys::Win32::UI::Shell::{
        ExtractIconExW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON, SHGetFileInfoW,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::HICON;

    let mut hasher = DefaultHasher::new();
    path.to_string_lossy().to_lowercase().hash(&mut hasher);
    let hash_str = format!("{:016x}", hasher.finish());

    let cache_dir = get_cache_dir();
    let cache_file = cache_dir.join(format!("{hash_str}.png"));

    if cache_file.exists() && cache_file.metadata().map_or(0, |m| m.len()) > 0 {
        return Some(cache_file);
    }

    let mut wide: Vec<u16> = OsStr::new(path).encode_wide().collect();
    wide.push(0);

    // 1. Try ExtractIconExW first (extracts authentic embedded PE icon)
    let mut large_icon: HICON = null_mut();
    let icon_count =
        unsafe { ExtractIconExW(wide.as_ptr(), 0, &raw mut large_icon, null_mut(), 1) };

    if icon_count > 0 && !large_icon.is_null() {
        if let Some(saved) = hicon_to_png(large_icon, &cache_file) {
            return Some(saved);
        }
    }

    // 2. Fallback to SHGetFileInfoW
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

    if res != 0 && !shfi.hIcon.is_null() {
        if let Some(saved) = hicon_to_png(shfi.hIcon, &cache_file) {
            return Some(saved);
        }
    }

    None
}

#[cfg(target_os = "windows")]
#[allow(
    unsafe_code,
    clippy::too_many_lines,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss
)]
fn hicon_to_png(
    hicon: windows_sys::Win32::UI::WindowsAndMessaging::HICON,
    cache_file: &Path,
) -> Option<PathBuf> {
    use std::ptr::null_mut;
    use windows_sys::Win32::Graphics::Gdi::{
        BI_RGB, BITMAP, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, DeleteObject, GetDC,
        GetDIBits, GetObjectW, RGBQUAD, ReleaseDC,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, ICONINFO};

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
        cache_file,
        &rgba_buf,
        width as u32,
        height as u32,
        image::ExtendedColorType::Rgba8,
    )
    .is_ok()
    {
        Some(cache_file.to_path_buf())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[cfg(target_os = "windows")]
    fn test_extract_explorer_icon() {
        let icon = resolve_entry_icon(Some("C:\\Windows\\explorer.exe"), None);
        assert!(icon.is_some());
        let path = icon.unwrap();
        assert!(Path::new(&path).exists());
    }
}
