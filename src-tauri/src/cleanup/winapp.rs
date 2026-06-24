use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use rayon::prelude::*;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

use super::winapp_db::{
    ExcludeRule, ExcludeRuleType, IniEntry, all_exclude_rules, winapp2_entries, winappx_entries,
};
use super::{
    DeleteOutcome, cleanup_status_from_error, delete_target_contents, expand_env_path,
    expand_wildcard_path, force_remove_path, is_busy_delete_error, target_size_bytes,
    wildcard_match,
};
use crate::cleanup::types::{CleanupCategoryReport, CleanupEntry, CleanupEntryStatus};
use crate::error::AppError;

fn is_path_within_dir(path_lower: &str, dir_lower: &str) -> bool {
    path_lower == dir_lower || path_str_starts_with_dir(path_lower, dir_lower)
}

fn path_str_starts_with_dir(path: &str, dir: &str) -> bool {
    let dir_with_sep = format!("{dir}\\");
    path.starts_with(&dir_with_sep) || path.starts_with(&format!("{dir}/"))
}

fn is_excluded(path: &Path, exclude_rules: &[ExcludeRule]) -> bool {
    let path_str = path.to_string_lossy().to_ascii_lowercase();

    for rule in exclude_rules {
        match rule.rule_type {
            ExcludeRuleType::File => {
                if let Some(expanded) = expand_winapp_path(&rule.path)
                    && path_str == expanded.to_ascii_lowercase()
                {
                    return true;
                }
            }
            ExcludeRuleType::Path => {
                if let Some(expanded) = expand_winapp_path(&rule.path) {
                    for exclude_dir in expand_wildcard_path(PathBuf::from(expanded)) {
                        let dir_lower = exclude_dir.to_string_lossy().to_ascii_lowercase();
                        if !is_path_within_dir(&path_str, &dir_lower) {
                            continue;
                        }
                        if let Some(ref pattern) = rule.file_pattern {
                            let file_name = path
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_default();
                            if !wildcard_match(pattern, &file_name) {
                                continue;
                            }
                        }
                        return true;
                    }
                }
            }
        }
    }
    false
}

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
    default_checked: bool,
    warning: Option<String>,
}

pub fn is_winapp_category(category_id: &str) -> bool {
    WINAPP_CATEGORIES.contains(&category_id)
}

pub fn scan_category(category_id: &str) -> Result<CleanupCategoryReport, AppError> {
    let entries = if category_id == "appx" {
        scan_appx_entries(false, &[])?
    } else {
        let excludes = all_exclude_rules();
        let excludes_ref: &[ExcludeRule] = &excludes;
        winapp_targets_for_category(category_id)
            .into_par_iter()
            .map(|target| scan_or_clean_winapp_target(&target, false, excludes_ref))
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
        let excludes = all_exclude_rules();
        let excludes_ref: &[ExcludeRule] = &excludes;
        winapp_targets_for_category(category_id)
            .into_par_iter()
            .map(|target| {
                let should_clean = !exclude_set.contains(&target.id);
                scan_or_clean_winapp_target(&target, should_clean, excludes_ref)
            })
            .collect()
    };

    Ok(CleanupCategoryReport {
        id: category_id.to_string(),
        entries,
    })
}

fn winapp_targets_for_category(category_id: &str) -> Vec<WinappTarget> {
    winapp2_entries()
        .par_iter()
        .filter(|entry| category_for_entry(entry) == category_id)
        .filter(|entry| is_detected(entry))
        .filter_map(|entry| {
            let rules = file_rules(entry);
            (!rules.is_empty()).then(|| WinappTarget {
                id: slug(&entry.name),
                name: entry.name.clone(),
                rules,
                default_checked: default_checked(entry),
                warning: entry.first("Warning"),
            })
        })
        .collect()
}

fn scan_or_clean_winapp_target(
    target: &WinappTarget,
    clean: bool,
    exclude_rules: &[ExcludeRule],
) -> CleanupEntry {
    let matches = matched_paths(&target.rules, exclude_rules);
    let mut first_error = None;
    let mut skipped_busy_error = None;
    let mut scheduled_on_reboot_error = None;

    if clean {
        for matched in &matches {
            match remove_match(matched) {
                Ok(DeleteOutcome::Deleted) => {}
                Ok(DeleteOutcome::SkippedBusy(error)) => {
                    skipped_busy_error.get_or_insert(error);
                }
                Ok(DeleteOutcome::ScheduledOnReboot(error)) => {
                    scheduled_on_reboot_error.get_or_insert(error);
                }
                Err(error) => {
                    if is_busy_delete_error(&error) {
                        skipped_busy_error.get_or_insert(error.to_string());
                    } else if error.kind() != io::ErrorKind::NotFound {
                        first_error.get_or_insert(error);
                    }
                }
            }
        }
    }

    let mut size_bytes = 0;
    let mut remaining_count = 0;

    for matched in &matches {
        if fs::symlink_metadata(&matched.path).is_err() {
            continue;
        }
        remaining_count += 1;
        match target_size_bytes(&matched.path) {
            Ok(size) => size_bytes += size,
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) if is_busy_delete_error(&error) => {
                skipped_busy_error.get_or_insert(error.to_string());
            }
            Err(error) => {
                if first_error.is_none() {
                    first_error = Some(error);
                }
            }
        }
    }

    let (status, error_message) = if let Some(error) = &first_error {
        (cleanup_status_from_error(error), Some(error.to_string()))
    } else if let Some(error) = scheduled_on_reboot_error {
        (
            CleanupEntryStatus::Busy,
            Some(format!("Scheduled for deletion on reboot. ({error})")),
        )
    } else if let Some(error) = skipped_busy_error {
        (
            CleanupEntryStatus::Busy,
            Some(format!("Some files are in use and were skipped. ({error})")),
        )
    } else if remaining_count == 0 {
        (CleanupEntryStatus::Clean, None)
    } else {
        (CleanupEntryStatus::Pending, None)
    };

    CleanupEntry {
        id: target.id.clone(),
        name: target.name.clone(),
        path: format_match_summary(remaining_count),
        status,
        size_bytes,
        error: error_message,
        icon_data_url: None,
        default_checked: target.default_checked,
        warning: target.warning.clone(),
    }
}

fn matched_paths(rules: &[FileRule], exclude_rules: &[ExcludeRule]) -> Vec<FileRuleMatch> {
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

    if !exclude_rules.is_empty() {
        matches.retain(|m| !is_excluded(&m.path, exclude_rules));
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

fn remove_match(matched: &FileRuleMatch) -> io::Result<DeleteOutcome> {
    if matched.remove_self {
        force_remove_path(&matched.path, true)
    } else {
        delete_target_contents(&matched.path)
    }
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

fn category_for_entry(entry: &IniEntry) -> &'static str {
    if let Some(lang) = entry.first("LangSecRef")
        && let Some(category) = category_from_lang_sec_ref(&lang)
    {
        return category;
    }
    keyword_category(&entry.name)
}

fn category_from_lang_sec_ref(code: &str) -> Option<&'static str> {
    let category = match code {
        "3006" | "3026" | "3027" | "3029" | "3032" | "3033" | "3034" | "3035" | "3039" => {
            "browsers"
        }
        "3025" => "windows",
        "3021" | "3022" | "3024" | "3030" | "3031" | "3037" | "3038" | "3043" | "3044" => {
            "applications"
        }
        "3023" | "3036" => "media",
        _ => return None,
    };
    Some(category)
}

fn keyword_category(name: &str) -> &'static str {
    let name = name.to_ascii_lowercase();

    if contains_any(
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

fn default_checked(entry: &IniEntry) -> bool {
    entry
        .first("Default")
        .map(|value| value.eq_ignore_ascii_case("true"))
        .unwrap_or(true)
}

fn contains_any(value: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| value.contains(needle))
}

fn scan_appx_entries(
    clean: bool,
    exclude_entry_ids: &[String],
) -> Result<Vec<CleanupEntry>, AppError> {
    let packages = installed_appx_packages()?;
    let entries = winappx_entries()
        .iter()
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
                name: entry.name.clone(),
                path: package_name,
                status,
                size_bytes: 0,
                error,
                icon_data_url: None,
                default_checked: true,
                warning: None,
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
