use std::collections::HashMap;
use std::ffi::{OsStr, c_void};
use std::mem::size_of;
use std::path::{Path, PathBuf};
use std::ptr::null_mut;
use std::sync::{Mutex, OnceLock};

use base64::Engine;
use png::{BitDepth, ColorType, Encoder};
use regex::Regex;
use windows::Win32::Foundation::{MAX_PATH, RPC_E_CHANGED_MODE};
use windows::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleDC, CreateDIBSection, DIB_RGB_COLORS,
    DeleteDC, DeleteObject, GetDIBits, HGDIOBJ, SelectObject,
};
use windows::Win32::Storage::FileSystem::{
    FILE_ATTRIBUTE_NORMAL, FILE_FLAGS_AND_ATTRIBUTES, GetFileVersionInfoSizeW, GetFileVersionInfoW,
    VerQueryValueW,
};
use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
    IPersistFile, STGM_READ,
};
use windows::Win32::UI::Shell::{
    IShellLinkW, SHFILEINFOW, SHGFI_ICON, SHGFI_SMALLICON, SHGFI_USEFILEATTRIBUTES, SHGetFileInfoW,
    SLGP_RAWPATH, ShellLink,
};
use windows::Win32::UI::WindowsAndMessaging::{DI_NORMAL, DestroyIcon, DrawIconEx};
use windows::core::Interface;
use windows::core::PCWSTR;

#[derive(Debug, Clone, Default)]
pub struct ParsedCommand {
    pub raw: String,
    pub executable_path: Option<PathBuf>,
    pub arguments: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct StartupPresentation {
    pub display_name: String,
    pub publisher: Option<String>,
    pub icon_data_url: Option<String>,
    pub target_path: Option<String>,
    pub arguments: Option<String>,
    pub working_directory: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct ShortcutTarget {
    target_path: Option<PathBuf>,
    arguments: Option<String>,
    working_directory: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct FilePresentation {
    display_name: Option<String>,
    publisher: Option<String>,
    icon_data_url: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct VersionStrings {
    file_description: Option<String>,
    product_name: Option<String>,
    company_name: Option<String>,
}

pub fn parse_command_line(input: &str) -> ParsedCommand {
    let raw = input.trim().to_string();
    if raw.is_empty() {
        return ParsedCommand::default();
    }

    if let Some(parsed) = parse_quoted_command(&raw) {
        return parsed;
    }

    if let Some(parsed) = parse_unquoted_command(&raw) {
        return parsed;
    }

    ParsedCommand {
        raw,
        executable_path: None,
        arguments: None,
    }
}

pub fn registry_presentation(raw_name: &str, command: Option<&str>) -> StartupPresentation {
    let parsed = command.map(parse_command_line).unwrap_or_default();
    let executable = parsed.executable_path.as_deref();
    let metadata = executable.map(file_presentation_for_path);

    StartupPresentation {
        display_name: metadata
            .as_ref()
            .and_then(|value| value.display_name.clone())
            .unwrap_or_else(|| normalize_registry_name(raw_name)),
        publisher: metadata.as_ref().and_then(|value| value.publisher.clone()),
        icon_data_url: metadata
            .as_ref()
            .and_then(|value| value.icon_data_url.clone()),
        target_path: executable.map(path_to_string),
        arguments: parsed.arguments,
        working_directory: None,
    }
}

pub fn registry_presentation_light(raw_name: &str, command: Option<&str>) -> StartupPresentation {
    let parsed = command.map(parse_command_line).unwrap_or_default();
    let executable = parsed.executable_path.as_deref();

    StartupPresentation {
        display_name: file_stem(executable).unwrap_or_else(|| normalize_registry_name(raw_name)),
        publisher: None,
        icon_data_url: None,
        target_path: executable.map(path_to_string),
        arguments: parsed.arguments,
        working_directory: None,
    }
}

pub fn startup_folder_presentation(path: &Path) -> StartupPresentation {
    let shortcut = resolve_shortcut_cached(path);
    let resolved_target = shortcut
        .as_ref()
        .and_then(|value| value.target_path.as_deref());
    let metadata = resolved_target
        .or(Some(path))
        .map(file_presentation_for_path);
    let working_directory = shortcut
        .as_ref()
        .and_then(|value| value.working_directory.clone())
        .or_else(|| path.parent().map(path_to_string));
    let arguments = shortcut.as_ref().and_then(|value| value.arguments.clone());

    StartupPresentation {
        display_name: metadata
            .as_ref()
            .and_then(|value| value.display_name.clone())
            .unwrap_or_else(|| fallback_path_name(path)),
        publisher: metadata.as_ref().and_then(|value| value.publisher.clone()),
        icon_data_url: metadata
            .as_ref()
            .and_then(|value| value.icon_data_url.clone()),
        target_path: resolved_target
            .map(path_to_string)
            .or_else(|| Some(path_to_string(path))),
        arguments,
        working_directory,
    }
}

pub fn startup_folder_presentation_light(path: &Path) -> StartupPresentation {
    let shortcut = resolve_shortcut_cached(path);

    StartupPresentation {
        display_name: fallback_path_name(path),
        publisher: None,
        icon_data_url: None,
        target_path: shortcut
            .as_ref()
            .and_then(|value| value.target_path.as_ref())
            .map(|value| path_to_string(value))
            .or_else(|| Some(path_to_string(path))),
        arguments: shortcut.as_ref().and_then(|value| value.arguments.clone()),
        working_directory: shortcut
            .as_ref()
            .and_then(|value| value.working_directory.clone())
            .or_else(|| path.parent().map(path_to_string)),
    }
}

pub fn scheduled_task_presentation(
    raw_name: &str,
    command: Option<&str>,
    arguments: Option<&str>,
    working_directory: Option<&str>,
) -> StartupPresentation {
    let executable = command
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty());
    let metadata = executable.as_deref().map(file_presentation_for_path);

    StartupPresentation {
        display_name: metadata
            .as_ref()
            .and_then(|value| value.display_name.clone())
            .unwrap_or_else(|| normalize_task_name(raw_name)),
        publisher: metadata.as_ref().and_then(|value| value.publisher.clone()),
        icon_data_url: metadata
            .as_ref()
            .and_then(|value| value.icon_data_url.clone()),
        target_path: executable.as_deref().map(path_to_string),
        arguments: arguments
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string),
        working_directory: working_directory
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string),
    }
}

pub fn scheduled_task_presentation_light(
    raw_name: &str,
    command: Option<&str>,
    arguments: Option<&str>,
    working_directory: Option<&str>,
) -> StartupPresentation {
    let executable = command
        .map(PathBuf::from)
        .filter(|path| !path.as_os_str().is_empty());

    StartupPresentation {
        display_name: file_stem(executable.as_deref())
            .unwrap_or_else(|| normalize_task_name(raw_name)),
        publisher: None,
        icon_data_url: None,
        target_path: executable.as_deref().map(path_to_string),
        arguments: arguments
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string),
        working_directory: working_directory
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string),
    }
}

fn parse_quoted_command(raw: &str) -> Option<ParsedCommand> {
    let trimmed = raw.trim_start();
    if !trimmed.starts_with('"') {
        return None;
    }

    let end_quote = trimmed[1..].find('"')? + 1;
    let executable = trimmed[1..end_quote].trim();
    if executable.is_empty() {
        return None;
    }

    let arguments = trimmed[(end_quote + 1)..].trim().to_string();

    Some(ParsedCommand {
        raw: raw.to_string(),
        executable_path: Some(PathBuf::from(executable)),
        arguments: (!arguments.is_empty()).then_some(arguments),
    })
}

fn parse_unquoted_command(raw: &str) -> Option<ParsedCommand> {
    let lower = raw.to_ascii_lowercase();
    for extension in command_extensions() {
        let mut search_start = 0;
        while let Some(relative_index) = lower[search_start..].find(extension) {
            let index = search_start + relative_index;
            let end = index + extension.len();
            let after = raw[end..].chars().next();
            if after.is_some_and(|value| !value.is_whitespace()) {
                search_start = end;
                continue;
            }

            let executable = raw[..end].trim();
            if executable.is_empty() {
                search_start = end;
                continue;
            }

            let arguments = raw[end..].trim();
            return Some(ParsedCommand {
                raw: raw.to_string(),
                executable_path: Some(PathBuf::from(executable)),
                arguments: (!arguments.is_empty()).then_some(arguments.to_string()),
            });
        }
    }

    None
}

fn command_extensions() -> &'static [&'static str] {
    &[
        ".exe", ".com", ".bat", ".cmd", ".ps1", ".vbs", ".js", ".msc", ".scr",
    ]
}

fn normalize_registry_name(raw_name: &str) -> String {
    raw_name
        .split(['.', '_', '-'])
        .rfind(|part| !part.trim().is_empty())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| raw_name.trim().to_string())
}

fn normalize_task_name(raw_name: &str) -> String {
    let mut value = raw_name.trim().to_string();

    if let Some(index) = value.find("S-1-5-") {
        value.truncate(index);
    }

    value = trailing_guid_regex().replace(&value, "").into_owned();
    value = trailing_hex_regex().replace(&value, "").into_owned();
    value = whitespace_regex()
        .replace_all(value.trim(), " ")
        .into_owned();

    if value.is_empty() {
        raw_name.trim().to_string()
    } else {
        value
    }
}

fn trailing_guid_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| {
        Regex::new(r"(?i)(?:Core)?\{[0-9a-f-]{36}\}\s*$").expect("valid trailing GUID regex")
    })
}

fn trailing_hex_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"\s+[A-F0-9]{8,}\s*$").expect("valid trailing hex regex"))
}

fn whitespace_regex() -> &'static Regex {
    static REGEX: OnceLock<Regex> = OnceLock::new();
    REGEX.get_or_init(|| Regex::new(r"\s+").expect("valid whitespace regex"))
}

fn fallback_path_name(path: &Path) -> String {
    path.file_stem()
        .and_then(|value| value.to_str())
        .or_else(|| path.file_name().and_then(|value| value.to_str()))
        .unwrap_or("Startup entry")
        .to_string()
}

fn file_presentation_for_path(path: &Path) -> FilePresentation {
    let cache_key = cache_key(path);
    let cache = file_presentation_cache();

    if let Ok(guard) = cache.lock()
        && let Some(cached) = guard.get(&cache_key)
    {
        return cached.clone();
    }

    let computed = compute_file_presentation(path);

    if let Ok(mut guard) = cache.lock() {
        guard.insert(cache_key, computed.clone());
    }

    computed
}

fn compute_file_presentation(path: &Path) -> FilePresentation {
    let version_strings = if path.exists() {
        read_version_strings(path)
    } else {
        None
    };

    FilePresentation {
        display_name: version_strings
            .as_ref()
            .and_then(|value| {
                value
                    .file_description
                    .as_deref()
                    .filter(|text| is_reasonable_display_name(text))
                    .map(str::to_string)
            })
            .or_else(|| {
                version_strings.as_ref().and_then(|value| {
                    value
                        .product_name
                        .as_deref()
                        .filter(|text| is_reasonable_display_name(text))
                        .map(str::to_string)
                })
            })
            .or_else(|| file_stem(Some(path))),
        publisher: version_strings.as_ref().and_then(|value| {
            value
                .company_name
                .as_deref()
                .filter(|text| is_reasonable_publisher(text))
                .map(str::to_string)
        }),
        icon_data_url: resolve_icon_data_url(Some(path)),
    }
}

fn read_version_strings(path: &Path) -> Option<VersionStrings> {
    let wide_path = wide_from_os(path.as_os_str());
    let mut handle = 0u32;
    let size = unsafe { GetFileVersionInfoSizeW(PCWSTR(wide_path.as_ptr()), Some(&mut handle)) };
    if size == 0 {
        return None;
    }

    let mut data = vec![0u8; size as usize];
    unsafe {
        GetFileVersionInfoW(
            PCWSTR(wide_path.as_ptr()),
            Some(handle),
            size,
            data.as_mut_ptr() as *mut c_void,
        )
        .ok()?;
    }

    let translation = query_translation(&data).unwrap_or((0x0409, 0x04B0));

    Some(VersionStrings {
        file_description: query_version_string(
            &data,
            &format!(
                "\\StringFileInfo\\{:04x}{:04x}\\FileDescription",
                translation.0, translation.1
            ),
        ),
        product_name: query_version_string(
            &data,
            &format!(
                "\\StringFileInfo\\{:04x}{:04x}\\ProductName",
                translation.0, translation.1
            ),
        ),
        company_name: query_version_string(
            &data,
            &format!(
                "\\StringFileInfo\\{:04x}{:04x}\\CompanyName",
                translation.0, translation.1
            ),
        ),
    })
}

fn query_translation(data: &[u8]) -> Option<(u16, u16)> {
    #[repr(C)]
    struct Translation {
        language: u16,
        code_page: u16,
    }

    let mut buffer = null_mut();
    let mut length = 0u32;
    let key = wide_from_str("\\VarFileInfo\\Translation");
    let success = unsafe {
        VerQueryValueW(
            data.as_ptr() as *const c_void,
            PCWSTR(key.as_ptr()),
            &mut buffer,
            &mut length,
        )
    };

    if !success.as_bool() || buffer.is_null() || length < size_of::<Translation>() as u32 {
        return None;
    }

    let translation = unsafe { &*(buffer as *const Translation) };
    Some((translation.language, translation.code_page))
}

fn query_version_string(data: &[u8], key: &str) -> Option<String> {
    let mut buffer = null_mut();
    let mut length = 0u32;
    let wide_key = wide_from_str(key);
    let success = unsafe {
        VerQueryValueW(
            data.as_ptr() as *const c_void,
            PCWSTR(wide_key.as_ptr()),
            &mut buffer,
            &mut length,
        )
    };

    if !success.as_bool() || buffer.is_null() || length == 0 {
        return None;
    }

    let max_units = length as usize;
    if max_units == 0 {
        return None;
    }

    let slice = unsafe { std::slice::from_raw_parts(buffer as *const u16, max_units) };
    let nul_index = slice.iter().position(|ch| *ch == 0).unwrap_or(slice.len());
    let value = String::from_utf16_lossy(&slice[..nul_index])
        .trim()
        .to_string();

    (!value.is_empty()).then_some(value)
}

fn is_reasonable_display_name(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > 80 {
        return false;
    }

    if trimmed.chars().any(|ch| ch.is_control()) {
        return false;
    }

    let lower = trimmed.to_ascii_lowercase();
    let suspicious_tokens = [
        "fileversion",
        "productversion",
        "originalfilename",
        "stringfileinfo",
        "varfileinfo",
    ];

    if suspicious_tokens.iter().any(|token| lower.contains(token)) {
        return false;
    }

    if trimmed.contains('\u{fffd}') {
        return false;
    }

    true
}

fn is_reasonable_publisher(value: &str) -> bool {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > 100 {
        return false;
    }

    if trimmed.chars().any(|ch| ch.is_control()) {
        return false;
    }

    let lower = trimmed.to_ascii_lowercase();
    let suspicious_tokens = [
        "fileversion",
        "productversion",
        "originalfilename",
        "stringfileinfo",
        "varfileinfo",
    ];

    if suspicious_tokens.iter().any(|token| lower.contains(token)) {
        return false;
    }

    if trimmed.contains('\u{fffd}') {
        return false;
    }

    true
}

fn resolve_icon_data_url(path: Option<&Path>) -> Option<String> {
    let path = path?;
    let wide_path = wide_from_os(path.as_os_str());
    let mut info = SHFILEINFOW::default();
    let flags = if path.exists() {
        SHGFI_ICON | SHGFI_SMALLICON
    } else {
        SHGFI_ICON | SHGFI_SMALLICON | SHGFI_USEFILEATTRIBUTES
    };

    let result = unsafe {
        SHGetFileInfoW(
            PCWSTR(wide_path.as_ptr()),
            FILE_FLAGS_AND_ATTRIBUTES(FILE_ATTRIBUTE_NORMAL.0),
            Some(&mut info),
            size_of::<SHFILEINFOW>() as u32,
            flags,
        )
    };

    if result == 0 || info.hIcon.0.is_null() {
        return None;
    }

    let icon = info.hIcon;
    let encoded = icon_to_png_data_url(icon);
    let _ = unsafe { DestroyIcon(icon) };
    encoded
}

fn resolve_shortcut_cached(path: &Path) -> Option<ShortcutTarget> {
    let cache_key = cache_key(path);
    let cache = shortcut_target_cache();

    if let Ok(guard) = cache.lock()
        && let Some(cached) = guard.get(&cache_key)
    {
        return cached.clone();
    }

    let computed = resolve_shortcut(path).ok().flatten();

    if let Ok(mut guard) = cache.lock() {
        guard.insert(cache_key, computed.clone());
    }

    computed
}

fn icon_to_png_data_url(icon: windows::Win32::UI::WindowsAndMessaging::HICON) -> Option<String> {
    const ICON_SIZE: i32 = 32;

    let mut bitmap_info = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: ICON_SIZE,
            biHeight: -ICON_SIZE,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        },
        ..Default::default()
    };

    let device_context = unsafe { CreateCompatibleDC(None) };
    if device_context.0.is_null() {
        return None;
    }

    let mut bits = null_mut();
    let bitmap =
        unsafe { CreateDIBSection(None, &bitmap_info, DIB_RGB_COLORS, &mut bits, None, 0).ok()? };

    let previous = unsafe { SelectObject(device_context, HGDIOBJ(bitmap.0)) };
    if previous.0.is_null() {
        let _ = unsafe { DeleteObject(HGDIOBJ(bitmap.0)) };
        let _ = unsafe { DeleteDC(device_context) };
        return None;
    }

    if unsafe {
        DrawIconEx(
            device_context,
            0,
            0,
            icon,
            ICON_SIZE,
            ICON_SIZE,
            0,
            None,
            DI_NORMAL,
        )
    }
    .is_err()
    {
        let _ = unsafe { SelectObject(device_context, previous) };
        let _ = unsafe { DeleteObject(HGDIOBJ(bitmap.0)) };
        let _ = unsafe { DeleteDC(device_context) };
        return None;
    }

    let mut pixels = vec![0u8; (ICON_SIZE * ICON_SIZE * 4) as usize];
    let scanlines = unsafe {
        GetDIBits(
            device_context,
            bitmap,
            0,
            ICON_SIZE as u32,
            Some(pixels.as_mut_ptr() as *mut c_void),
            &mut bitmap_info,
            DIB_RGB_COLORS,
        )
    };

    let _ = unsafe { SelectObject(device_context, previous) };
    let _ = unsafe { DeleteObject(HGDIOBJ(bitmap.0)) };
    let _ = unsafe { DeleteDC(device_context) };

    if scanlines == 0 {
        return None;
    }

    for chunk in pixels.chunks_exact_mut(4) {
        chunk.swap(0, 2);
    }

    let mut png_data = Vec::new();
    let mut encoder = Encoder::new(&mut png_data, ICON_SIZE as u32, ICON_SIZE as u32);
    encoder.set_color(ColorType::Rgba);
    encoder.set_depth(BitDepth::Eight);

    {
        let mut writer = encoder.write_header().ok()?;
        writer.write_image_data(&pixels).ok()?;
    }

    Some(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(png_data)
    ))
}

fn resolve_shortcut(path: &Path) -> windows::core::Result<Option<ShortcutTarget>> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    if !extension.eq_ignore_ascii_case("lnk") {
        return Ok(None);
    }

    let com = ComGuard::new()?;
    let shell_link: IShellLinkW =
        unsafe { CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)? };
    let persist_file: IPersistFile = shell_link.cast()?;
    let wide_path = wide_from_os(path.as_os_str());

    unsafe {
        persist_file.Load(PCWSTR(wide_path.as_ptr()), STGM_READ)?;
    }

    let target_path = read_shell_link_path(&shell_link)?;
    let arguments = read_shell_link_arguments(&shell_link)?;
    let working_directory = read_shell_link_working_directory(&shell_link)?;

    drop(com);

    Ok(Some(ShortcutTarget {
        target_path: target_path.filter(|value| !value.as_os_str().is_empty()),
        arguments,
        working_directory,
    }))
}

fn read_shell_link_path(shell_link: &IShellLinkW) -> windows::core::Result<Option<PathBuf>> {
    let mut buffer = [0u16; MAX_PATH as usize];
    unsafe {
        shell_link.GetPath(&mut buffer, std::ptr::null_mut(), SLGP_RAWPATH.0 as u32)?;
    }

    Ok(wide_buffer_to_string(&buffer).map(PathBuf::from))
}

fn read_shell_link_arguments(shell_link: &IShellLinkW) -> windows::core::Result<Option<String>> {
    let mut buffer = [0u16; 1024];
    unsafe {
        shell_link.GetArguments(&mut buffer)?;
    }

    Ok(wide_buffer_to_string(&buffer))
}

fn read_shell_link_working_directory(
    shell_link: &IShellLinkW,
) -> windows::core::Result<Option<String>> {
    let mut buffer = [0u16; MAX_PATH as usize];
    unsafe {
        shell_link.GetWorkingDirectory(&mut buffer)?;
    }

    Ok(wide_buffer_to_string(&buffer))
}

fn file_stem(path: Option<&Path>) -> Option<String> {
    let path = path?;
    path.file_stem()
        .and_then(|value| value.to_str())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn cache_key(path: &Path) -> PathBuf {
    PathBuf::from(path.to_string_lossy().to_ascii_lowercase())
}

fn file_presentation_cache() -> &'static Mutex<HashMap<PathBuf, FilePresentation>> {
    static CACHE: OnceLock<Mutex<HashMap<PathBuf, FilePresentation>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn shortcut_target_cache() -> &'static Mutex<HashMap<PathBuf, Option<ShortcutTarget>>> {
    static CACHE: OnceLock<Mutex<HashMap<PathBuf, Option<ShortcutTarget>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn wide_from_os(value: &OsStr) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;

    value.encode_wide().chain(Some(0)).collect()
}

fn wide_from_str(value: &str) -> Vec<u16> {
    wide_from_os(OsStr::new(value))
}

fn wide_buffer_to_string(buffer: &[u16]) -> Option<String> {
    let end = buffer
        .iter()
        .position(|value| *value == 0)
        .unwrap_or(buffer.len());
    let value = String::from_utf16_lossy(&buffer[..end]).trim().to_string();
    (!value.is_empty()).then_some(value)
}

struct ComGuard {
    initialized: bool,
}

impl ComGuard {
    fn new() -> windows::core::Result<Self> {
        let status = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        if status.is_ok() {
            return Ok(Self { initialized: true });
        }

        if status == RPC_E_CHANGED_MODE {
            return Ok(Self { initialized: false });
        }

        status.ok()?;
        unreachable!("successful COM initialization should have returned earlier")
    }
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        if self.initialized {
            unsafe {
                CoUninitialize();
            }
        }
    }
}
