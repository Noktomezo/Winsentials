use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};

/// Expands environment variables in a string (e.g. `%SystemRoot%`, `%ProgramFiles%`, `%APPDATA%`).
#[must_use]
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
#[must_use]
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

/// Retrieves publisher (`CompanyName`) and description (`FileDescription`) from PE version info.
#[cfg(target_os = "windows")]
#[allow(unsafe_code, clippy::too_many_lines)]
#[must_use]
pub fn get_file_metadata(path: &Path) -> (Option<String>, Option<String>) {
    use windows_sys::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};

    if !path.exists() {
        return (None, None);
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
            return (None, None);
        }

        let get_size_ptr = GetProcAddress(version_dll, c"GetFileVersionInfoSizeW".as_ptr().cast());
        let get_info_ptr = GetProcAddress(version_dll, c"GetFileVersionInfoW".as_ptr().cast());
        let query_val_ptr = GetProcAddress(version_dll, c"VerQueryValueW".as_ptr().cast());

        if get_size_ptr.is_none() || get_info_ptr.is_none() || query_val_ptr.is_none() {
            return (None, None);
        }

        let get_size: GetSizeFn = std::mem::transmute(get_size_ptr);
        let get_info: GetInfoFn = std::mem::transmute(get_info_ptr);
        let query_val: QueryValFn = std::mem::transmute(query_val_ptr);

        let mut handle = 0u32;
        let size = get_size(wide_path.as_ptr(), &raw mut handle);
        if size == 0 {
            return (None, None);
        }

        let mut buffer = vec![0u8; size as usize];
        if get_info(wide_path.as_ptr(), 0, size, buffer.as_mut_ptr()) == 0 {
            return (None, None);
        }

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

        lang_codepages.push("040904b0".to_string()); // US English Unicode
        lang_codepages.push("040904e4".to_string()); // US English Windows
        lang_codepages.push("000004b0".to_string()); // Neutral Unicode
        lang_codepages.push("041904b0".to_string()); // Russian Unicode
        lang_codepages.push("041904e4".to_string()); // Russian Windows

        let read_prop = |sub_key: &str| -> Option<String> {
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
            None
        };

        let publisher = read_prop("CompanyName").or_else(|| read_prop("LegalCopyright"));
        let description = read_prop("FileDescription").or_else(|| read_prop("ProductName"));

        (publisher, description)
    }
}

#[cfg(not(target_os = "windows"))]
pub fn get_file_metadata(_path: &Path) -> (Option<String>, Option<String>) {
    (None, None)
}

#[allow(dead_code)]
#[must_use]
pub fn get_file_publisher(path: &Path) -> Option<String> {
    get_file_metadata(path).0
}

#[must_use]
pub fn get_file_description(path: &Path) -> Option<String> {
    get_file_metadata(path).1
}

/// Checks if a string looks like a technical GUID, UUID or hex hash (e.g. `308046B0AF4A39CB`, `B55B6E519228D8CADAE7F34C8F656C40`).
fn is_hex_or_hash_token(token: &str) -> bool {
    let clean = token
        .trim_matches(|c| c == '{' || c == '}' || c == '-' || c == '_' || c == '(' || c == ')');
    if clean.len() >= 8 && clean.chars().all(|c| c.is_ascii_hexdigit() || c == '-') {
        clean.chars().any(|c| c.is_ascii_digit())
    } else {
        false
    }
}

/// Splits camelCase or `PascalCase` words into space-separated words.
fn split_pascal_case_word(word: &str) -> String {
    let upper_count = word.chars().filter(|c| c.is_uppercase()).count();
    if word.len() < 12
        || upper_count < 3
        || word.chars().all(|c| c.is_uppercase() || !c.is_alphabetic())
        || word.chars().all(|c| c.is_lowercase() || !c.is_alphabetic())
    {
        return word.to_string();
    }

    let mut result = String::new();
    let chars: Vec<char> = word.chars().collect();
    for i in 0..chars.len() {
        let curr = chars[i];
        if i > 0 {
            let prev = chars[i - 1];
            let next = chars.get(i + 1).copied();

            if (prev.is_lowercase() && curr.is_uppercase())
                || (prev.is_uppercase()
                    && curr.is_uppercase()
                    && next.is_some_and(char::is_lowercase))
            {
                result.push(' ');
            }
        }
        result.push(curr);
    }
    result
}

/// Strips Windows GUIDs and SID patterns from names.
fn strip_guids_and_sids(name: &str) -> String {
    let mut result = name.to_string();

    // Remove {GUID} e.g. {85B8898F-2E0F-4F4D-93D3-8E5A737D8D74}
    while let Some(start) = result.find('{') {
        if let Some(end) = result[start..].find('}') {
            let full_end = start + end + 1;
            let inside = &result[start + 1..start + end];
            if inside.len() >= 32 && inside.chars().all(|c| c.is_ascii_hexdigit() || c == '-') {
                result.replace_range(start..full_end, "");
                continue;
            }
        }
        break;
    }

    // Remove S-1-5-21-... or -S-1-5-21-... Windows SID
    if let Some(sid_pos) = result.find("S-1-") {
        let start_idx = if sid_pos > 0
            && (result.as_bytes()[sid_pos - 1] == b'-' || result.as_bytes()[sid_pos - 1] == b'_')
        {
            sid_pos - 1
        } else {
            sid_pos
        };
        let after_s1 = &result[sid_pos + 4..];
        let end_offset = after_s1
            .find(|c: char| !c.is_ascii_digit() && c != '-')
            .unwrap_or(after_s1.len());
        result.replace_range(start_idx..sid_pos + 4 + end_offset, "");
    }

    // Trim trailing separators
    let cleaned =
        result.trim_matches(|c: char| c == '-' || c == '_' || c == '.' || c.is_whitespace());
    cleaned.to_string()
}

/// Cleans and formats a technical startup entry name into a clean, human-readable display name.
#[must_use]
pub fn clean_display_name(raw_name: &str, target_exe: Option<&Path>) -> String {
    let trimmed = raw_name.trim();

    // 1. Try PE FileDescription first if raw_name looks technical
    let is_technical = trimmed.starts_with("electron.app.")
        || trimmed.starts_with("com.")
        || trimmed.starts_with("org.")
        || trimmed.starts_with("net.")
        || trimmed.contains('{')
        || trimmed.contains("-S-1-")
        || (trimmed.contains('.') && !trimmed.contains(' '))
        || is_hex_or_hash_token(trimmed);

    if let Some(path) = target_exe {
        if let Some(desc) = get_file_description(path) {
            let desc_trimmed = desc.trim();
            if !desc_trimmed.is_empty()
                && !desc_trimmed.eq_ignore_ascii_case("unknown")
                && (is_technical || desc_trimmed.len() > trimmed.len())
            {
                return desc_trimmed.to_string();
            }
        }
    }

    // 2. Handle reverse-domain / bundle prefixes (e.g. electron.app.Notion -> Notion, com.squirrel.Discord.Discord -> Discord)
    if let Some(after) = trimmed.strip_prefix("electron.app.") {
        return after.replace('_', " ");
    }
    if let Some(after) = trimmed.strip_prefix("com.squirrel.") {
        let parts: Vec<&str> = after.split('.').collect();
        if let Some(last) = parts.last() {
            if !last.is_empty() {
                return (*last).to_string();
            }
        }
    }
    if (trimmed.starts_with("com.") || trimmed.starts_with("org.") || trimmed.starts_with("net."))
        && trimmed.matches('.').count() >= 2
    {
        if let Some(last_dot) = trimmed.rfind('.') {
            let last_part = &trimmed[last_dot + 1..];
            if !last_part.is_empty() && last_part.chars().any(char::is_alphabetic) {
                return last_part.replace('_', " ");
            }
        }
    }

    // 3. Strip GUIDs and SIDs
    let without_guids = strip_guids_and_sids(trimmed);
    let mut candidate = if without_guids.is_empty() {
        if let Some(path) = target_exe {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(trimmed)
                .to_string()
        } else {
            trimmed.to_string()
        }
    } else {
        without_guids
    };

    // 4. Dot-separated names without spaces (e.g. COD.Broker.Service -> COD Broker Service)
    if candidate.contains('.') && !candidate.contains(' ') {
        candidate = candidate.replace('.', " ");
    }

    // 5. Underscores between words (e.g. User_Feed_Synchronization -> User Feed Synchronization)
    if candidate.contains('_') && !candidate.contains("://") {
        candidate = candidate.replace('_', " ");
    }

    // 6. Clean multiple spaces, filter out hex hash tokens, and split PascalCase words
    let mut words = Vec::new();
    for w in candidate.split_whitespace() {
        if is_hex_or_hash_token(w) {
            continue;
        }
        let split_w = split_pascal_case_word(w);
        for sub_w in split_w.split_whitespace() {
            words.push(sub_w.to_string());
        }
    }

    let final_name = words.join(" ");

    if final_name.is_empty() {
        if let Some(path) = target_exe {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(trimmed)
                .to_string()
        } else {
            trimmed.to_string()
        }
    } else {
        final_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_electron_notion() {
        assert_eq!(clean_display_name("electron.app.Notion", None), "Notion");
    }

    #[test]
    fn test_clean_squirrel_discord() {
        assert_eq!(
            clean_display_name("com.squirrel.Discord.Discord", None),
            "Discord"
        );
    }

    #[test]
    fn test_clean_dot_service() {
        assert_eq!(
            clean_display_name("COD.Broker.Service", None),
            "COD Broker Service"
        );
    }

    #[test]
    fn test_clean_task_with_sid() {
        assert_eq!(
            clean_display_name(
                "OneDrive Standalone Update Task-S-1-5-21-3948574-1001",
                None
            ),
            "OneDrive Standalone Update Task"
        );
    }

    #[test]
    fn test_clean_task_with_guid() {
        assert_eq!(
            clean_display_name(
                "MicrosoftEdgeUpdateTaskMachine{8F0D756C-B2D1-4E6D-967E-D1E8F2312674}",
                None
            ),
            "Microsoft Edge Update Task Machine"
        );
    }

    #[test]
    fn test_clean_underscores() {
        assert_eq!(
            clean_display_name("User_Feed_Synchronization", None),
            "User Feed Synchronization"
        );
    }

    #[test]
    fn test_clean_firefox_hashes() {
        assert_eq!(
            clean_display_name("Firefox Background Update 308046B0AF4A39CB", None),
            "Firefox Background Update"
        );
        assert_eq!(
            clean_display_name("Firefox Default Browser Agent 308046B0AF4A39CB", None),
            "Firefox Default Browser Agent"
        );
    }

    #[test]
    fn test_clean_edge_autolaunch() {
        assert_eq!(
            clean_display_name(
                "MicrosoftEdgeAutoLaunch_B55B6E519228D8CADAE7F34C8F656C40",
                None
            ),
            "Microsoft Edge Auto Launch"
        );
    }
}
