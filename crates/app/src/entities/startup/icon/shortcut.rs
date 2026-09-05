use std::path::{Path, PathBuf};
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
