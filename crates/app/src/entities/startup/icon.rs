use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use crate::entities::startup::vendor::extract_clean_exe_path;

#[cfg(target_os = "windows")]
const MAX_ICON_DIMENSION: i32 = 1_024;

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
        let is_lnk = clean_path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("lnk"))
            || clean_path
                .to_string_lossy()
                .to_lowercase()
                .ends_with(".lnk.disabled");
        let actual_path = if is_lnk {
            resolve_shortcut(&clean_path).unwrap_or_else(|| clean_path.clone())
        } else {
            clean_path.clone()
        };

        // 1. Direct icon extraction from target executable (if PE has real icons)
        if actual_path.exists() {
            if let Some(icon) = extract_direct_icon(&actual_path) {
                return Some(icon);
            }
            if actual_path != clean_path && clean_path.exists() {
                if let Some(icon) = extract_direct_icon(&clean_path) {
                    return Some(icon);
                }
            }
            // 2. Sibling GUI executable discovery in the same directory (for CLI helpers / services)
            if let Some(sibling_icon) = find_sibling_icon(&actual_path) {
                return Some(sibling_icon);
            }
        }

        // 3. Fallback for PowerShell scripts or PowerShell host commands
        let is_ps = clean_path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("ps1"))
            || clean_path.file_name().is_some_and(|n| {
                let s = n.to_string_lossy().to_lowercase();
                s == "powershell.exe" || s == "pwsh.exe"
            })
            || command.is_some_and(|c| c.to_ascii_lowercase().contains("powershell"));

        if is_ps {
            let ps_path =
                PathBuf::from(r"C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe");
            if ps_path.exists() {
                if let Some(icon) = extract_direct_icon(&ps_path) {
                    return Some(icon);
                }
            }
        }

        // 4. Fallback for Batch / CMD scripts
        let is_cmd = clean_path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("bat") || e.eq_ignore_ascii_case("cmd"))
            || clean_path.file_name().is_some_and(|n| {
                let s = n.to_string_lossy().to_lowercase();
                s == "cmd.exe"
            });

        if is_cmd {
            let cmd_path = PathBuf::from(r"C:\Windows\System32\cmd.exe");
            if cmd_path.exists() {
                if let Some(icon) = extract_direct_icon(&cmd_path) {
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
    let dir = std::env::temp_dir().join("winsentials_icon_cache_v4");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

#[cfg(target_os = "windows")]
fn find_sibling_icon(exe_path: &Path) -> Option<PathBuf> {
    let parent = exe_path.parent()?;
    let current_name = exe_path.file_name()?.to_string_lossy().to_lowercase();
    let current_stem = exe_path.file_stem()?.to_string_lossy().to_lowercase();
    let folder_name = parent.file_name()?.to_string_lossy().to_lowercase();

    // Never search siblings in common/shared directories where unrelated executables live!
    let lower_parent = parent.to_string_lossy().to_lowercase();
    if lower_parent.ends_with(r"\startup")
        || lower_parent.ends_with(r"\system32")
        || lower_parent.ends_with(r"\syswow64")
        || lower_parent.ends_with(r"\windows")
        || lower_parent.ends_with(r"\temp")
        || lower_parent.ends_with(r"\tmp")
        || lower_parent.ends_with(r"\downloads")
        || lower_parent.ends_with(r"\desktop")
        || lower_parent.ends_with(r"\program files")
        || lower_parent.ends_with(r"\program files (x86)")
    {
        return None;
    }

    let Ok(entries) = std::fs::read_dir(parent) else {
        return None;
    };

    let mut ranked_candidates = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if ext.eq_ignore_ascii_case("exe") {
                    let name = path.file_name().unwrap().to_string_lossy().to_lowercase();
                    if name != current_name {
                        let stem = path.file_stem().unwrap().to_string_lossy().to_lowercase();
                        let score = if stem == folder_name {
                            0
                        } else if current_stem.starts_with(&stem) || stem.starts_with(&current_stem)
                        {
                            1
                        } else if stem.contains("radeon")
                            || stem.contains("amd")
                            || stem.contains("control")
                            || stem.contains("main")
                            || stem.contains("launcher")
                            || stem.contains("gui")
                        {
                            2
                        } else {
                            continue; // DO NOT include unrelated executables!
                        };
                        ranked_candidates.push((score, path));
                    }
                }
            }
        }
    }

    ranked_candidates.sort_by_key(|(score, _)| *score);

    for (_, cand) in ranked_candidates {
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

    // 1. Extract authentic embedded PE icon
    let mut large_icon: HICON = null_mut();
    let icon_count =
        unsafe { ExtractIconExW(wide.as_ptr(), 0, &raw mut large_icon, null_mut(), 1) };

    if icon_count > 0 && !large_icon.is_null() {
        if let Some(saved) = hicon_to_png(large_icon, &cache_file) {
            return Some(saved);
        }
    }

    // 2. Query Windows Shell API (SHGetFileInfoW) for .lnk shortcuts, manifests, and file associations
    let mut sfi: SHFILEINFOW = unsafe { std::mem::zeroed() };
    let res = unsafe {
        SHGetFileInfoW(
            wide.as_ptr(),
            0,
            &raw mut sfi,
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        )
    };
    if res != 0 && !sfi.hIcon.is_null() {
        if let Some(saved) = hicon_to_png(sfi.hIcon, &cache_file) {
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

    let cleanup = || {
        // SAFETY: these handles were returned by `GetIconInfo`; each is released once
        // after all GDI reads finish, and null bitmap handles are explicitly skipped.
        unsafe {
            if !icon_info.hbmColor.is_null() {
                DeleteObject(icon_info.hbmColor);
            }
            if !icon_info.hbmMask.is_null() {
                DeleteObject(icon_info.hbmMask);
            }
            DestroyIcon(hicon);
        }
    };

    let mut bmp: BITMAP = unsafe { std::mem::zeroed() };
    if unsafe {
        GetObjectW(
            icon_info.hbmColor,
            std::mem::size_of::<BITMAP>() as i32,
            (&raw mut bmp).cast(),
        )
    } == 0
    {
        cleanup();
        return None;
    }

    let width = bmp.bmWidth;
    let height = bmp.bmHeight;
    if width <= 0 || height <= 0 || width > MAX_ICON_DIMENSION || height > MAX_ICON_DIMENSION {
        cleanup();
        return None;
    }

    let Some(pixel_count) = (width as usize).checked_mul(height as usize) else {
        cleanup();
        return None;
    };
    let Some(buffer_len) = pixel_count.checked_mul(4) else {
        cleanup();
        return None;
    };
    let mut bgra_buf: Vec<u8> = vec![0u8; buffer_len];

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
    // SAFETY: the bitmap dimensions are validated above and `bgra_buf` contains exactly
    // `width * height * 4` writable bytes required by the 32-bit top-down DIB request.
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
    // SAFETY: `hdc` was acquired by `GetDC` above and has not been released yet.
    unsafe { ReleaseDC(null_mut(), hdc) };
    cleanup();

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

    let mut rgba_buf = Vec::with_capacity(buffer_len);
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

/// Resolves a Windows Shell Link (`.lnk`) file to its target executable path.
#[cfg(target_os = "windows")]
#[must_use]
pub fn resolve_shortcut(lnk_path: &Path) -> Option<PathBuf> {
    let bytes = std::fs::read(lnk_path).ok()?;
    if bytes.len() < 76 || bytes[0..4] != [0x4C, 0x00, 0x00, 0x00] {
        return None;
    }

    let flags = u32::from_le_bytes([bytes[0x14], bytes[0x15], bytes[0x16], bytes[0x17]]);
    let has_id_list = (flags & 0x01) != 0;
    let has_link_info = (flags & 0x02) != 0;

    let mut offset = 76usize;

    if has_id_list {
        if bytes.len() < offset + 2 {
            return None;
        }
        let id_list_size = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]) as usize;
        offset += 2 + id_list_size;
    }

    if has_link_info && bytes.len() >= offset + 28 {
        let info_start = offset;
        let header_size = u32::from_le_bytes([
            bytes[info_start + 4],
            bytes[info_start + 5],
            bytes[info_start + 6],
            bytes[info_start + 7],
        ]) as usize;

        // Try Unicode local base path if header_size >= 0x24 (Windows Vista+)
        if header_size >= 0x24 && bytes.len() >= info_start + 32 {
            let u_offset = u32::from_le_bytes([
                bytes[info_start + 28],
                bytes[info_start + 29],
                bytes[info_start + 30],
                bytes[info_start + 31],
            ]) as usize;
            if u_offset > 0 && info_start + u_offset < bytes.len() {
                let slice = &bytes[info_start + u_offset..];
                let u16_chars: Vec<u16> = slice
                    .chunks_exact(2)
                    .map(|c| u16::from_le_bytes([c[0], c[1]]))
                    .take_while(|&c| c != 0)
                    .collect();
                let path_str = String::from_utf16_lossy(&u16_chars);
                let pb = PathBuf::from(path_str);
                if pb.exists() {
                    return Some(pb);
                }
            }
        }

        // Try ASCII local base path
        let a_offset = u32::from_le_bytes([
            bytes[info_start + 16],
            bytes[info_start + 17],
            bytes[info_start + 18],
            bytes[info_start + 19],
        ]) as usize;
        if a_offset > 0 && info_start + a_offset < bytes.len() {
            let slice = &bytes[info_start + a_offset..];
            let ascii_str: String = slice
                .iter()
                .copied()
                .take_while(|&b| b != 0)
                .map(|b| b as char)
                .collect();
            let pb = PathBuf::from(ascii_str);
            if pb.exists() {
                return Some(pb);
            }
        }
    }

    // Fallback heuristic: scan raw bytes for any existing executable or working dir path
    find_path_in_bytes(&bytes, lnk_path)
}

/// Fallback scanner for non-standard or advertised/MSI shortcuts.
#[cfg(target_os = "windows")]
fn find_path_in_bytes(bytes: &[u8], lnk_path: &Path) -> Option<PathBuf> {
    let file_name = lnk_path.file_name().and_then(|s| s.to_str()).unwrap_or("");
    let stem = file_name
        .trim_end_matches(".disabled")
        .trim_end_matches(".lnk");

    let check_candidate = |cand: &str| -> Option<PathBuf> {
        let trimmed = cand.trim().trim_matches('"');
        if trimmed.is_empty() {
            return None;
        }

        // Direct file check
        let pb = PathBuf::from(trimmed);
        if pb.is_file() && pb.exists() {
            return Some(pb);
        }

        // Working directory + stem.exe check
        if pb.is_dir() && pb.exists() && !stem.is_empty() {
            let exe_cand = pb.join(format!("{stem}.exe"));
            if exe_cand.is_file() && exe_cand.exists() {
                return Some(exe_cand);
            }
        }

        // Relative path from shortcut's folder
        if trimmed.starts_with(r"..") {
            if let Some(parent) = lnk_path.parent() {
                let resolved = parent.join(trimmed);
                if resolved.is_file() && resolved.exists() {
                    return Some(resolved);
                }
            }
        }

        None
    };

    // 1. Scan ASCII strings
    let mut i = 0;
    while i + 4 < bytes.len() {
        if (bytes[i].is_ascii_alphabetic()
            && bytes[i + 1] == b':'
            && (bytes[i + 2] == b'\\' || bytes[i + 2] == b'/'))
            || (bytes[i] == b'.' && bytes[i + 1] == b'.' && bytes[i + 2] == b'\\')
        {
            let end = bytes[i..]
                .iter()
                .position(|&b| b == 0 || b == b'"' || b < 32)
                .unwrap_or(bytes.len() - i);
            let candidate: String = bytes[i..i + end].iter().map(|&b| b as char).collect();
            if let Some(found) = check_candidate(&candidate) {
                return Some(found);
            }
            i += end + 1;
        } else {
            i += 1;
        }
    }

    // 2. Scan UTF-16 strings (both even and odd byte offsets)
    for start_offset in [0, 1] {
        let mut idx = start_offset;
        while idx + 8 < bytes.len() {
            let c0 = u16::from_le_bytes([bytes[idx], bytes[idx + 1]]);
            let c1 = u16::from_le_bytes([bytes[idx + 2], bytes[idx + 3]]);
            let c2 = u16::from_le_bytes([bytes[idx + 4], bytes[idx + 5]]);

            let is_drive = u8::try_from(c0).is_ok_and(|b| b.is_ascii_alphabetic())
                && c1 == u16::from(b':')
                && (c2 == u16::from(b'\\') || c2 == u16::from(b'/'));
            let is_dotdot = c0 == u16::from(b'.')
                && c1 == u16::from(b'.')
                && (c2 == u16::from(b'\\') || c2 == u16::from(b'/'));

            if is_drive || is_dotdot {
                let u16s: Vec<u16> = bytes[idx..]
                    .chunks_exact(2)
                    .map(|c| u16::from_le_bytes([c[0], c[1]]))
                    .take_while(|&c| c != 0 && c != u16::from(b'"') && c >= 32)
                    .collect();
                let candidate = String::from_utf16_lossy(&u16s);
                if let Some(found) = check_candidate(&candidate) {
                    return Some(found);
                }
                idx += 2;
            } else {
                idx += 2;
            }
        }
    }

    None
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

    #[test]
    #[cfg(target_os = "windows")]
    fn test_extract_powershell_script_icon() {
        let icon = resolve_entry_icon(
            Some(r"C:\ProgramData\Winhance\OpenWebSearch\OpenWebSearchRepair.ps1"),
            Some(
                r#"powershell.exe -ExecutionPolicy Bypass -NoProfile -Command "iex([IO.File]::ReadAllText('C:\ProgramData\Winhance\OpenWebSearch\OpenWebSearchRepair.ps1'))""#,
            ),
        );
        assert!(icon.is_some());
        let path = icon.unwrap();
        assert!(Path::new(&path).exists());
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_extract_amnezia_service_icon() {
        let amnezia_path = r"C:\Program Files\AmneziaVPN\AmneziaVPN-service.exe";
        if Path::new(amnezia_path).exists() {
            let icon = resolve_entry_icon(Some(amnezia_path), None);
            assert!(icon.is_some());
        }
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_extract_cncmd_sibling_icon() {
        let cncmd_path = r"C:\Program Files\AMD\CNext\CNext\cncmd.exe";
        if Path::new(cncmd_path).exists() {
            let icon = resolve_entry_icon(Some(cncmd_path), None);
            assert!(icon.is_some());
        }
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_resolve_shortcut_sharex() {
        let appdata = std::env::var("APPDATA").unwrap();
        let lnk = PathBuf::from(appdata)
            .join(r"Microsoft\Windows\Start Menu\Programs\Startup\ShareX.lnk");
        if lnk.exists() {
            let target = resolve_shortcut(&lnk);
            assert!(target.is_some());
            let t = target.unwrap();
            assert!(t.to_string_lossy().to_lowercase().contains("sharex.exe"));
        }
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_resolve_shortcut_waves() {
        let progdata = std::env::var("PROGRAMDATA").unwrap();
        let lnk = PathBuf::from(progdata)
            .join(r"Microsoft\Windows\Start Menu\Programs\Startup\WavesLocalServer.lnk");
        if lnk.exists() {
            let target = resolve_shortcut(&lnk);
            assert!(target.is_some());
            let t = target.unwrap();
            assert!(
                t.to_string_lossy()
                    .to_lowercase()
                    .contains("waveslocalserver.exe")
            );
        }
    }
}
