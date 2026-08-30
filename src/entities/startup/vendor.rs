use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};

/// Expands environment variables in a string (e.g. `%SystemRoot%`, `%ProgramFiles%`, `%APPDATA%`).
pub fn expand_env_vars(input: &str) -> String {
    let mut result = input.to_string();
    if !result.contains('%') {
        return result;
    }

    let env_vars = [
        "SystemRoot",
        "windir",
        "ProgramFiles",
        "ProgramFiles(x86)",
        "ProgramData",
        "APPDATA",
        "LOCALAPPDATA",
        "USERPROFILE",
        "SystemDrive",
    ];

    for var in env_vars {
        if let Ok(val) = std::env::var(var) {
            let pattern = format!("%{var}%");
            result = result.replace(&pattern, &val);
            let pattern_lower = format!("%{}%", var.to_ascii_lowercase());
            result = result.replace(&pattern_lower, &val);
        }
    }

    result
}

/// Extracts a clean executable path from a command string with arguments or quotes.
pub fn extract_clean_exe_path(cmd: &str) -> Option<PathBuf> {
    let trimmed = cmd.trim();
    if trimmed.is_empty() {
        return None;
    }

    // Strip kernel/NT prefixes like \??\ or \\?\
    let clean_str = trimmed
        .strip_prefix(r"\??\")
        .or_else(|| trimmed.strip_prefix(r"\\?\"))
        .unwrap_or(trimmed);

    let expanded = expand_env_vars(clean_str);
    let trimmed_exp = expanded.trim();

    // 1. Quoted path: "C:\Program Files\App\app.exe" --arg
    if let Some(stripped) = trimmed_exp.strip_prefix('"') {
        if let Some(end_quote) = stripped.find('"') {
            let path_part = &stripped[..end_quote];
            let p = PathBuf::from(path_part);
            return Some(p);
        }
    }

    // 2. Look for common executable extensions followed by space or end of string
    let lower = trimmed_exp.to_ascii_lowercase();
    for ext in &[".exe", ".bat", ".cmd", ".vbs", ".ps1", ".lnk"] {
        if let Some(idx) = lower.find(ext) {
            let end_idx = idx + ext.len();
            let path_part = &trimmed_exp[..end_idx];
            let p = PathBuf::from(path_part);
            return Some(p);
        }
    }

    // 3. Fallback: split at first space
    if let Some(space_idx) = trimmed_exp.find(' ') {
        let p = PathBuf::from(&trimmed_exp[..space_idx]);
        Some(p)
    } else {
        Some(PathBuf::from(trimmed_exp))
    }
}

#[cfg(target_os = "windows")]
type GetSizeFn = unsafe extern "system" fn(*const u16, *mut u32) -> u32;
#[cfg(target_os = "windows")]
type GetInfoFn = unsafe extern "system" fn(*const u16, u32, u32, *mut u8) -> i32;
#[cfg(target_os = "windows")]
type QueryValFn = unsafe extern "system" fn(*const u8, *const u16, *mut *mut u8, *mut u32) -> i32;

/// Retrieves the publisher or company name from the PE version info of a file.
#[cfg(target_os = "windows")]
#[allow(unsafe_code)]
pub fn get_file_publisher(path: &Path) -> Option<String> {
    use windows_sys::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};

    if !path.exists() {
        return None;
    }

    let wide_path: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        let dll_name: Vec<u16> = OsStr::new("version.dll")
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let version_dll = LoadLibraryW(dll_name.as_ptr());
        if version_dll.is_null() {
            return None;
        }

        let get_size_ptr = GetProcAddress(version_dll, c"GetFileVersionInfoSizeW".as_ptr().cast());
        let get_info_ptr = GetProcAddress(version_dll, c"GetFileVersionInfoW".as_ptr().cast());
        let query_val_ptr = GetProcAddress(version_dll, c"VerQueryValueW".as_ptr().cast());

        if get_size_ptr.is_none() || get_info_ptr.is_none() || query_val_ptr.is_none() {
            return None;
        }

        let get_size: GetSizeFn = std::mem::transmute(get_size_ptr);
        let get_info: GetInfoFn = std::mem::transmute(get_info_ptr);
        let query_val: QueryValFn = std::mem::transmute(query_val_ptr);

        let mut handle = 0u32;
        let size = get_size(wide_path.as_ptr(), &raw mut handle);
        if size == 0 {
            return None;
        }

        let mut buffer = vec![0u8; size as usize];
        if get_info(wide_path.as_ptr(), 0, size, buffer.as_mut_ptr()) == 0 {
            return None;
        }

        // Try Translation table first
        let mut trans_ptr = std::ptr::null_mut();
        let mut trans_len = 0u32;
        let trans_block_name: Vec<u16> = OsStr::new(r"\VarFileInfo\Translation")
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let mut lang_codepages = Vec::new();
        if query_val(
            buffer.as_ptr(),
            trans_block_name.as_ptr(),
            &raw mut trans_ptr,
            &raw mut trans_len,
        ) != 0
            && trans_len >= 4
            && !trans_ptr.is_null()
        {
            let pair_count = trans_len as usize / 4;
            #[allow(clippy::cast_ptr_alignment)]
            let pairs = std::slice::from_raw_parts(trans_ptr.cast::<u16>(), pair_count * 2);
            for i in 0..pair_count {
                let lang = pairs[i * 2];
                let cp = pairs[i * 2 + 1];
                lang_codepages.push(format!("{lang:04x}{cp:04x}"));
            }
        }

        // Common fallback codepages
        lang_codepages.push("040904b0".to_string()); // US English Unicode
        lang_codepages.push("040904e4".to_string()); // US English Windows
        lang_codepages.push("000004b0".to_string()); // Neutral Unicode
        lang_codepages.push("041904b0".to_string()); // Russian Unicode
        lang_codepages.push("041904e4".to_string()); // Russian Windows

        for sub_key in &["CompanyName", "FileDescription", "ProductName"] {
            for lc in &lang_codepages {
                let query_str = format!(r"\StringFileInfo\{lc}\{sub_key}");
                let query_wide: Vec<u16> = OsStr::new(&query_str)
                    .encode_wide()
                    .chain(std::iter::once(0))
                    .collect();

                let mut val_ptr = std::ptr::null_mut();
                let mut val_len = 0u32;
                if query_val(
                    buffer.as_ptr(),
                    query_wide.as_ptr(),
                    &raw mut val_ptr,
                    &raw mut val_len,
                ) != 0
                    && !val_ptr.is_null()
                    && val_len > 0
                {
                    #[allow(clippy::cast_ptr_alignment)]
                    let slice = std::slice::from_raw_parts(val_ptr.cast::<u16>(), val_len as usize);
                    let end = slice.iter().position(|&c| c == 0).unwrap_or(slice.len());
                    let s = String::from_utf16_lossy(&slice[..end]).trim().to_string();
                    if !s.is_empty() && !s.eq_ignore_ascii_case("unknown") {
                        return Some(s);
                    }
                }
            }
        }

        None
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_file_publisher(_path: &Path) -> Option<String> {
    None
}
