use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use rayon::prelude::*;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

use super::{
    cleanup_status_from_error, expand_env_path, expand_wildcard_path, target_size_bytes,
    wildcard_match,
};
use crate::cleanup::types::{CleanupCategoryReport, CleanupEntry, CleanupEntryStatus};
use crate::error::AppError;

const WINAPP2: &str = include_str!("../../assets/Winapp2.ini");
const WINAPPX: &str = include_str!("../../assets/Winappx.ini");

const EXCLUDED_BROWSER_ENTRY_TERMS: &[&str] = &[
    "autofill",
    "backup",
    "bookmark",
    "cookies",
    "credential",
    "download history",
    "form",
    "history",
    "login",
    "password",
    "pinned tabs",
    "places",
    "saved",
    "session",
    "site preferences",
    "storage",
    "sync",
];

const EXCLUDED_WINDOWS_ENTRY_TERMS: &[&str] = &[
    "windows activity history",
    "windows credential manager saved credentials",
    "windows network certificate cache",
    "windows notepad",
    "windows recent access",
    "windows recent documents",
    "windows recent wallpaper locations",
    "windows shell",
    "windows start menu",
    "windows taskbar",
];

pub const WINAPP_CATEGORIES: &[&str] = &[
    "windows",
    "browsers",
    "applications",
    "development",
    "gaming",
    "media",
    "appx",
];

#[derive(Clone)]
struct IniEntry {
    name: String,
    values: HashMap<String, Vec<String>>,
}

#[derive(Clone)]
struct FileRule {
    root: String,
    patterns: Vec<String>,
    recurse: bool,
    remove_self: bool,
}

struct FileRuleMatch {
    path: PathBuf,
    remove_self: bool,
}

#[derive(Clone)]
struct WinappTarget {
    id: String,
    name: String,
    rules: Vec<FileRule>,
}

pub fn is_winapp_category(category_id: &str) -> bool {
    WINAPP_CATEGORIES.contains(&category_id)
}

pub fn scan_category(category_id: &str) -> Result<CleanupCategoryReport, AppError> {
    let entries = if category_id == "appx" {
        scan_appx_entries(false, &[])?
    } else {
        winapp_targets_for_category(category_id)
            .into_par_iter()
            .map(|target| scan_or_clean_winapp_target(&target, false))
            .collect()
    };

    Ok(CleanupCategoryReport {
        id: category_id.to_string(),
        entries,
    })
}

pub fn clean_category(
    category_id: &str,
    exclude_entry_ids: &[String],
) -> Result<CleanupCategoryReport, AppError> {
    let entries = if category_id == "appx" {
        scan_appx_entries(true, exclude_entry_ids)?
    } else {
        let exclude_set: HashSet<String> = exclude_entry_ids.iter().cloned().collect();
        winapp_targets_for_category(category_id)
            .into_par_iter()
            .map(|target| {
                let should_clean = !exclude_set.contains(&target.id);
                scan_or_clean_winapp_target(&target, should_clean)
            })
            .collect()
    };

    Ok(CleanupCategoryReport {
        id: category_id.to_string(),
        entries,
    })
}

fn winapp_targets_for_category(category_id: &str) -> Vec<WinappTarget> {
    parse_ini(WINAPP2)
        .into_par_iter()
        .filter(|entry| {
            let broad_category = broad_category(entry);
            broad_category == category_id && !is_excluded_winapp_entry(entry, broad_category)
        })
        .filter(is_detected)
        .filter_map(|entry| {
            let rules = file_rules(&entry);
            (!rules.is_empty()).then(|| WinappTarget {
                id: slug(&entry.name),
                name: entry.name,
                rules,
            })
        })
        .collect()
}

fn is_excluded_winapp_entry(entry: &IniEntry, broad_category: &str) -> bool {
    let name = entry.name.to_ascii_lowercase();

    match broad_category {
        "browsers" => contains_any(&name, EXCLUDED_BROWSER_ENTRY_TERMS),
        "windows" => contains_any(&name, EXCLUDED_WINDOWS_ENTRY_TERMS),
        _ => false,
    }
}

fn scan_or_clean_winapp_target(target: &WinappTarget, clean: bool) -> CleanupEntry {
    let matches = matched_paths(&target.rules);
    let mut first_error = None;

    if clean {
        for matched in &matches {
            if let Err(error) = remove_match(matched)
                && first_error.is_none()
            {
                first_error = Some(error);
            }
        }
    }

    let remaining_matches = matched_paths(&target.rules);
    let mut size_bytes = 0;

    for matched in &remaining_matches {
        match target_size_bytes(&matched.path) {
            Ok(size) => size_bytes += size,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                if first_error.is_none() {
                    first_error = Some(error);
                }
            }
        }
    }

    let status = if let Some(error) = &first_error {
        cleanup_status_from_error(error)
    } else if remaining_matches.is_empty() {
        CleanupEntryStatus::Clean
    } else {
        CleanupEntryStatus::Pending
    };

    CleanupEntry {
        id: target.id.clone(),
        name: target.name.clone(),
        path: format_match_summary(remaining_matches.len()),
        status,
        size_bytes,
        error: first_error.map(|error| error.to_string()),
        icon_data_url: None,
    }
}

fn matched_paths(rules: &[FileRule]) -> Vec<FileRuleMatch> {
    let mut seen = HashSet::new();
    let mut matches = Vec::new();

    for rule in rules {
        let Some(root) = expand_winapp_path(&rule.root) else {
            continue;
        };

        for root in expand_wildcard_path(PathBuf::from(root)) {
            collect_rule_matches(rule, &root, &mut seen, &mut matches);
        }
    }

    matches
}

fn collect_rule_matches(
    rule: &FileRule,
    root: &Path,
    seen: &mut HashSet<String>,
    matches: &mut Vec<FileRuleMatch>,
) {
    let metadata = fs::symlink_metadata(root);
    if metadata.as_ref().is_ok_and(|metadata| metadata.is_file()) {
        push_match(root, rule.remove_self, seen, matches);
        return;
    }

    let Ok(metadata) = metadata else {
        return;
    };
    if !metadata.is_dir() {
        return;
    }

    if rule.remove_self && pattern_list_matches(&rule.patterns, "*") {
        push_match(root, true, seen, matches);
        return;
    }

    walk_rule_dir(root, rule.recurse, &rule.patterns, seen, matches);
}

fn walk_rule_dir(
    root: &Path,
    recurse: bool,
    patterns: &[String],
    seen: &mut HashSet<String>,
    matches: &mut Vec<FileRuleMatch>,
) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if pattern_list_matches(patterns, &name) {
            push_match(&path, false, seen, matches);
        }

        let Ok(metadata) = fs::symlink_metadata(&path) else {
            continue;
        };
        if recurse && metadata.is_dir() && !metadata.file_type().is_symlink() {
            walk_rule_dir(&path, true, patterns, seen, matches);
        }
    }
}

fn push_match(
    path: &Path,
    remove_self: bool,
    seen: &mut HashSet<String>,
    matches: &mut Vec<FileRuleMatch>,
) {
    let key = path.to_string_lossy().to_ascii_lowercase();
    if seen.insert(key) {
        matches.push(FileRuleMatch {
            path: path.to_path_buf(),
            remove_self,
        });
    }
}

fn pattern_list_matches(patterns: &[String], value: &str) -> bool {
    patterns
        .iter()
        .any(|pattern| wildcard_match(pattern, value))
}

fn remove_match(matched: &FileRuleMatch) -> io::Result<()> {
    let metadata = fs::symlink_metadata(&matched.path)?;
    if metadata.file_type().is_symlink() {
        Ok(())
    } else if metadata.is_dir() {
        if matched.remove_self {
            fs::remove_dir_all(&matched.path)
        } else {
            remove_dir_contents(&matched.path)
        }
    } else {
        fs::remove_file(&matched.path)
    }
}

fn remove_dir_contents(path: &Path) -> io::Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() {
            continue;
        } else if metadata.is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }

    Ok(())
}

fn file_rules(entry: &IniEntry) -> Vec<FileRule> {
    entry
        .values
        .iter()
        .filter(|(key, _)| key.starts_with("FileKey"))
        .flat_map(|(_, values)| values)
        .filter_map(|value| parse_file_rule(value))
        .collect()
}

fn parse_file_rule(value: &str) -> Option<FileRule> {
    let mut parts = value.split('|').map(str::trim);
    let root = parts.next()?.to_string();
    let mut patterns = Vec::new();
    let mut recurse = false;
    let mut remove_self = false;

    for part in parts {
        if part.eq_ignore_ascii_case("RECURSE") {
            recurse = true;
        } else if part.eq_ignore_ascii_case("REMOVESELF") {
            remove_self = true;
        } else if !part.is_empty() {
            patterns.extend(
                part.split(';')
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(str::to_string),
            );
        }
    }

    if patterns.is_empty() {
        patterns.push("*".to_string());
    }

    Some(FileRule {
        root,
        patterns,
        recurse,
        remove_self,
    })
}

fn is_detected(entry: &IniEntry) -> bool {
    let has_detection = entry.values.keys().any(|key| {
        key.starts_with("Detect") || key.starts_with("DetectFile") || key == "SpecialDetect"
    });

    if !has_detection {
        return false;
    }

    entry.values.iter().any(|(key, values)| {
        values.iter().any(|value| {
            if key.starts_with("DetectFile") {
                detect_file(value)
            } else if key.starts_with("Detect") {
                detect_registry(value)
            } else if key == "SpecialDetect" {
                special_detect(value)
            } else {
                false
            }
        })
    })
}

fn detect_file(value: &str) -> bool {
    expand_winapp_path(value)
        .map(PathBuf::from)
        .map(expand_wildcard_path)
        .is_some_and(|paths| paths.into_iter().any(|path| path.exists()))
}

fn detect_registry(value: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        let Some((hive, subkey)) = value.split_once('\\') else {
            return false;
        };
        let subkey = subkey.split('|').next().unwrap_or(subkey);
        let root = match hive.to_ascii_uppercase().as_str() {
            "HKCU" | "HKEY_CURRENT_USER" => {
                winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER)
            }
            "HKLM" | "HKEY_LOCAL_MACHINE" => {
                winreg::RegKey::predef(winreg::enums::HKEY_LOCAL_MACHINE)
            }
            _ => return false,
        };
        root.open_subkey(subkey).is_ok()
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = value;
        false
    }
}

fn special_detect(value: &str) -> bool {
    let path = match value.trim() {
        "DET_CHROME" => "%LocalAppData%\\Google\\Chrome\\User Data",
        "DET_FIREFOX" => "%AppData%\\Mozilla\\Firefox",
        "DET_EDGE" => "%LocalAppData%\\Microsoft\\Edge\\User Data",
        "DET_OPERA" => "%AppData%\\Opera Software\\Opera Stable",
        "DET_THUNDERBIRD" => "%AppData%\\Thunderbird",
        "DET_WINSTORE" => "%LocalAppData%\\Packages",
        _ => return false,
    };
    detect_file(path)
}

fn expand_winapp_path(path: &str) -> Option<String> {
    let normalized = path
        .replace("%LocalAppData%", "{LOCALAPPDATA}")
        .replace("%AppData%", "{APPDATA}")
        .replace("%ProgramData%", "{PROGRAMDATA}")
        .replace("%CommonAppData%", "{PROGRAMDATA}")
        .replace("%UserProfile%", "{USERPROFILE}")
        .replace("%WinDir%", "{WINDIR}")
        .replace("%SystemRoot%", "{WINDIR}")
        .replace("%Temp%", "{TEMP}")
        .replace("%Tmp%", "{TMP}")
        .replace("%ProgramFiles(x86)%", "{PROGRAMFILES_X86}")
        .replace("%ProgramFilesX86%", "{PROGRAMFILES_X86}")
        .replace("%ProgramFiles%", "{PROGRAMFILES}");

    expand_env_path(&normalized)
}

fn broad_category(entry: &IniEntry) -> &'static str {
    let name = entry.name.to_ascii_lowercase();
    let lang = entry.first("LangSecRef").unwrap_or_default();

    if lang == "3029"
        || contains_any(
            &name,
            &[
                "chrome", "firefox", "edge", "opera", "brave", "vivaldi", "browser", "chromium",
                "safari",
            ],
        )
    {
        "browsers"
    } else if contains_any(
        &name,
        &[
            "steam",
            "epic games",
            "battle.net",
            "ubisoft",
            "ea app",
            "riot",
            "game",
            "minecraft",
            "roblox",
        ],
    ) {
        "gaming"
    } else if contains_any(
        &name,
        &[
            "visual studio",
            "vscode",
            "jetbrains",
            "android studio",
            "git ",
            "github",
            "node",
            "npm",
            "python",
            "rust",
            "docker",
            "developer",
        ],
    ) {
        "development"
    } else if contains_any(
        &name,
        &[
            "adobe",
            "photoshop",
            "media",
            "vlc",
            "spotify",
            "discord",
            "telegram",
            "slack",
            "teams",
            "zoom",
        ],
    ) {
        "media"
    } else if contains_any(
        &name,
        &[
            "windows",
            "microsoft",
            "explorer",
            "defender",
            "wer",
            "prefetch",
            "thumbnail",
            "font cache",
            "directx",
            "delivery optimization",
        ],
    ) {
        "windows"
    } else {
        "applications"
    }
}

fn contains_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}

impl IniEntry {
    fn first(&self, key: &str) -> Option<String> {
        self.values
            .get(key)
            .and_then(|values| values.first())
            .cloned()
    }
}

fn parse_ini(content: &str) -> Vec<IniEntry> {
    let mut entries = Vec::new();
    let mut current: Option<IniEntry> = None;

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with(';') {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            if let Some(entry) = current.take() {
                entries.push(entry);
            }
            current = Some(IniEntry {
                name: line
                    .trim_matches(['[', ']'])
                    .trim_end_matches(" *")
                    .to_string(),
                values: HashMap::new(),
            });
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if let Some(entry) = current.as_mut() {
            entry
                .values
                .entry(key.trim().to_string())
                .or_default()
                .push(value.trim().to_string());
        }
    }

    if let Some(entry) = current {
        entries.push(entry);
    }

    entries
}

fn scan_appx_entries(
    clean: bool,
    exclude_entry_ids: &[String],
) -> Result<Vec<CleanupEntry>, AppError> {
    let packages = installed_appx_packages()?;
    let entries = parse_ini(WINAPPX)
        .into_iter()
        .filter_map(|entry| {
            let package_name = entry.first("PackageName")?;
            let full_name = packages.get(&package_name.to_ascii_lowercase())?.clone();
            let entry_id = format!("appx_{}", slug(&package_name));
            let should_clean = clean && !exclude_entry_ids.contains(&entry_id);

            let (status, error) = if should_clean {
                match remove_appx_package(&full_name) {
                    Ok(()) => (CleanupEntryStatus::Removed, None),
                    Err(error) => (CleanupEntryStatus::Failed, Some(error.to_string())),
                }
            } else {
                (CleanupEntryStatus::Pending, None)
            };
            Some(CleanupEntry {
                id: entry_id,
                name: entry.name,
                path: package_name,
                status,
                size_bytes: 0,
                error,
                icon_data_url: None,
            })
        })
        .collect();

    Ok(entries)
}

fn installed_appx_packages() -> Result<HashMap<String, String>, AppError> {
    #[cfg(target_os = "windows")]
    {
        let expression = duct::cmd(
            "powershell",
            &[
                "-NoProfile",
                "-NonInteractive",
                "-Command",
                "Get-AppxPackage | ForEach-Object { $_.Name + '|' + $_.PackageFullName }",
            ],
        )
        .stdout_capture()
        .stderr_capture()
        .unchecked();

        let expression = expression.before_spawn(|command| {
            command.creation_flags(CREATE_NO_WINDOW);
            Ok(())
        });

        let output = expression.run().map_err(|error| {
            AppError::message(format!("failed to discover AppX packages: {error}"))
        })?;

        if !output.status.success() {
            return Err(AppError::CommandFailed {
                command: "Get-AppxPackage".to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
            });
        }

        Ok(String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|line| line.split_once('|'))
            .map(|(name, full_name)| (name.to_ascii_lowercase(), full_name.to_string()))
            .collect())
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(HashMap::new())
    }
}

fn remove_appx_package(package_full_name: &str) -> Result<(), AppError> {
    let escaped = package_full_name.replace('\'', "''");
    let expression = duct::cmd(
        "powershell",
        &[
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            &format!("Remove-AppxPackage -Package '{}'", escaped),
        ],
    )
    .stdout_capture()
    .stderr_capture()
    .unchecked();

    #[cfg(target_os = "windows")]
    let expression = expression.before_spawn(|command| {
        command.creation_flags(CREATE_NO_WINDOW);
        Ok(())
    });

    let output = expression.run()?;

    if output.status.success() {
        Ok(())
    } else {
        Err(AppError::CommandFailed {
            command: "Remove-AppxPackage".to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_string(),
        })
    }
}

fn format_match_summary(count: usize) -> String {
    match count {
        0 => "No matching cleanup targets found".to_string(),
        1 => "1 matched cleanup target".to_string(),
        count => format!("{count} matched cleanup targets"),
    }
}

fn slug(value: &str) -> String {
    let mut slug = String::new();
    let mut last_sep = false;

    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_sep = false;
        } else if !last_sep {
            slug.push('_');
            last_sep = true;
        }
    }

    slug.trim_matches('_').to_string()
}
