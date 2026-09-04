use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use glob::{MatchOptions, Pattern, glob_with};
use thiserror::Error;

const CATALOG: &str = include_str!("catalog.tsv");
const EXCLUSIONS: &str = include_str!("exclusions.tsv");
#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CleanupCategory {
    Windows,
    Browsers,
    Applications,
    Development,
    Games,
    Media,
    Devices,
}

impl CleanupCategory {
    pub const ALL: [Self; 7] = [
        Self::Windows,
        Self::Browsers,
        Self::Applications,
        Self::Development,
        Self::Games,
        Self::Media,
        Self::Devices,
    ];

    pub const fn id(self) -> &'static str {
        match self {
            Self::Windows => "windows",
            Self::Browsers => "browsers",
            Self::Applications => "applications",
            Self::Development => "development",
            Self::Games => "games",
            Self::Media => "media",
            Self::Devices => "devices",
        }
    }

    pub const fn icon(self) -> &'static str {
        match self {
            Self::Windows => "icons/monitor-cog.svg",
            Self::Browsers => "icons/globe.svg",
            Self::Applications => "icons/app-window.svg",
            Self::Development => "icons/code-xml.svg",
            Self::Games => "icons/gamepad-2.svg",
            Self::Media => "icons/video.svg",
            Self::Devices => "icons/usb.svg",
        }
    }
}

#[derive(Clone, Debug)]
pub struct CleanupPath {
    pub path: PathBuf,
    pub bytes: u64,
}

#[derive(Clone, Debug)]
pub struct CleanupTarget {
    pub id: String,
    pub name: String,
    pub category: CleanupCategory,
    pub paths: Vec<CleanupPath>,
    prune_roots: Vec<PathBuf>,
    pub device_instance_id: Option<String>,
    pub bytes: u64,
}

#[derive(Clone, Debug, Default)]
pub struct CleanupSnapshot {
    pub targets: Vec<CleanupTarget>,
}

#[derive(Clone, Debug, Default)]
pub struct CleanupReport {
    pub removed_bytes: u64,
    pub removed_paths: usize,
    pub failures: usize,
}

#[derive(Debug, Error)]
pub enum CleanupError {
    #[error("cleanup path is outside the allowed roots: {0}")]
    UnsafePath(PathBuf),
    #[error("could not remove {path}")]
    Remove {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("could not remove unused device {0}")]
    DeviceRemoval(String),
}

#[derive(Clone, Debug, Default)]
pub struct CleanupState {
    pub snapshot: CleanupSnapshot,
    pub selected: HashSet<String>,
    pub expanded: Option<CleanupCategory>,
    pub scanning: bool,
    pub cleaning: bool,
    pub scanned_once: bool,
}

impl CleanupState {
    pub fn apply_snapshot(&mut self, snapshot: CleanupSnapshot) {
        let available = snapshot
            .targets
            .iter()
            .map(|target| target.id.as_str())
            .collect::<HashSet<_>>();
        self.selected.retain(|id| available.contains(id.as_str()));
        self.snapshot = snapshot;
        self.scanning = false;
        self.scanned_once = true;
    }

    pub fn toggle_target(&mut self, id: &str) {
        if !self.selected.remove(id) {
            self.selected.insert(id.to_owned());
        }
    }

    pub fn toggle_category(&mut self, category: CleanupCategory) {
        let ids = self
            .snapshot
            .targets
            .iter()
            .filter(|target| target.category == category)
            .map(|target| target.id.clone())
            .collect::<Vec<_>>();
        let select = ids.iter().any(|id| !self.selected.contains(id));
        for id in ids {
            if select {
                self.selected.insert(id);
            } else {
                self.selected.remove(&id);
            }
        }
    }

    pub fn toggle_all(&mut self) {
        if self.selected.len() == self.snapshot.targets.len() {
            self.selected.clear();
        } else {
            self.selected = self
                .snapshot
                .targets
                .iter()
                .map(|target| target.id.clone())
                .collect();
        }
    }

    pub fn selected_totals(&self) -> (usize, u64) {
        self.snapshot
            .targets
            .iter()
            .filter(|target| self.selected.contains(&target.id))
            .fold((0, 0), |(count, bytes), target| {
                (count + 1, bytes + target.bytes)
            })
    }
}

#[derive(Clone)]
struct Rule {
    category: CleanupCategory,
    name: String,
    root: String,
    mask: String,
    recurse: bool,
    remove_self: bool,
    detect: Option<String>,
}

struct Exclusion {
    prefix: String,
    pattern: Option<Pattern>,
    literal: Option<String>,
}

pub fn scan_cleanup_targets() -> CleanupSnapshot {
    let mut grouped: HashMap<(CleanupCategory, String), Vec<Rule>> = HashMap::new();
    for rule in parse_catalog() {
        grouped
            .entry((rule.category, rule.name.clone()))
            .or_default()
            .push(rule);
    }

    let allowed_roots = allowed_roots();
    let exclusions = parse_exclusions();
    let mut detection_cache = HashMap::new();
    let mut targets = grouped
        .into_iter()
        .filter_map(|((category, name), rules)| {
            if !rules_detected(&rules, &mut detection_cache) {
                return None;
            }
            let mut paths = Vec::new();
            let mut prune_roots = Vec::new();
            let mut seen = HashSet::new();
            let target_exclusions = exclusions.get(&name).map_or(&[][..], Vec::as_slice);
            for rule in &rules {
                resolve_rule(
                    rule,
                    &allowed_roots,
                    target_exclusions,
                    &mut seen,
                    &mut paths,
                    &mut prune_roots,
                );
            }
            if paths.is_empty() {
                return None;
            }
            prune_roots.sort();
            prune_roots.dedup();
            let bytes = paths.iter().map(|path| path.bytes).sum();
            Some(CleanupTarget {
                id: format!("{}:{name}", category.id()),
                name,
                category,
                paths,
                prune_roots,
                device_instance_id: None,
                bytes,
            })
        })
        .collect::<Vec<_>>();
    targets.sort_by_key(|target| target.name.to_lowercase());
    CleanupSnapshot { targets }
}

#[must_use]
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["Б", "КБ", "МБ", "ГБ"];
    let mut value = bytes;
    let mut unit = 0;
    let mut remainder = 0;
    while value >= 1024 && unit < UNITS.len() - 1 {
        remainder = value % 1024;
        value /= 1024;
        unit += 1;
    }
    if unit == 0 {
        format!("{value} {}", UNITS[unit])
    } else {
        format!(
            "{value}.{} {}",
            remainder.saturating_mul(10) / 1024,
            UNITS[unit]
        )
    }
}

pub fn clean_selected(snapshot: &CleanupSnapshot, selected: &HashSet<String>) -> CleanupReport {
    let mut report = CleanupReport::default();
    for target in snapshot
        .targets
        .iter()
        .filter(|target| selected.contains(&target.id))
    {
        if let Some(instance_id) = &target.device_instance_id {
            match remove_unused_device(instance_id) {
                Ok(()) => report.removed_paths += 1,
                Err(error) => {
                    eprintln!("cleanup: {error}");
                    report.failures += 1;
                }
            }
            continue;
        }
        for path in &target.paths {
            match remove_cleanup_path(path) {
                Ok(()) => {
                    report.removed_bytes += path.bytes;
                    report.removed_paths += 1;
                }
                Err(error) => {
                    eprintln!("cleanup: {error}");
                    report.failures += 1;
                }
            }
        }
        for root in &target.prune_roots {
            prune_empty_dirs(root);
        }
    }
    report
}

pub fn scan_unused_devices() -> Vec<CleanupTarget> {
    #[cfg(target_os = "windows")]
    {
        let script = r"$OutputEncoding = [Console]::OutputEncoding = [Text.UTF8Encoding]::new(); Get-PnpDevice | Where-Object { $_.Present -eq $false } | ForEach-Object { $name = if ($_.FriendlyName) { $_.FriendlyName } else { $_.Class }; '{0}`t{1}' -f ($name -replace '[`t`r`n]', ' '), $_.InstanceId }";
        let output = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", script])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
        let Ok(output) = output else {
            return Vec::new();
        };
        if !output.status.success() {
            return Vec::new();
        }
        parse_unused_devices(&output.stdout)
    }
    #[cfg(not(target_os = "windows"))]
    Vec::new()
}

fn parse_unused_devices(output: &[u8]) -> Vec<CleanupTarget> {
    String::from_utf8_lossy(output)
        .lines()
        .filter_map(|line| line.split_once('\t'))
        .filter(|(name, instance_id)| !name.is_empty() && !instance_id.is_empty())
        .map(|(name, instance_id)| CleanupTarget {
            id: format!("devices:{instance_id}"),
            name: name.to_owned(),
            category: CleanupCategory::Devices,
            paths: Vec::new(),
            prune_roots: Vec::new(),
            device_instance_id: Some(instance_id.to_owned()),
            bytes: 0,
        })
        .collect()
}

fn remove_unused_device(instance_id: &str) -> Result<(), CleanupError> {
    #[cfg(target_os = "windows")]
    {
        let status = Command::new("pnputil")
            .args(["/remove-device", instance_id])
            .creation_flags(CREATE_NO_WINDOW)
            .status()
            .map_err(|_| CleanupError::DeviceRemoval(instance_id.to_owned()))?;
        status
            .success()
            .then_some(())
            .ok_or_else(|| CleanupError::DeviceRemoval(instance_id.to_owned()))
    }
    #[cfg(not(target_os = "windows"))]
    Err(CleanupError::DeviceRemoval(instance_id.to_owned()))
}

fn parse_catalog() -> Vec<Rule> {
    CATALOG
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .filter_map(|line| {
            let mut columns = line.splitn(4, '\t');
            let category = match columns.next()? {
                "windows" => CleanupCategory::Windows,
                "browsers" => CleanupCategory::Browsers,
                "applications" => CleanupCategory::Applications,
                "development" => CleanupCategory::Development,
                "games" => CleanupCategory::Games,
                "media" => CleanupCategory::Media,
                _ => return None,
            };
            let name = columns.next()?.to_owned();
            let mut value = columns.next()?.split('|');
            let detect = columns.next().map(str::to_owned);
            let root = value.next()?.to_owned();
            let mask = value.next().unwrap_or("*").to_owned();
            let flags = value.collect::<Vec<_>>().join("|").to_ascii_uppercase();
            let rule = Rule {
                category,
                name,
                root,
                mask,
                recurse: flags.contains("RECURSE"),
                remove_self: flags.contains("REMOVESELF"),
                detect,
            };
            Some(rule)
        })
        .collect()
}

fn parse_exclusions() -> HashMap<String, Vec<Exclusion>> {
    let mut exclusions = HashMap::<String, Vec<Exclusion>>::new();
    for line in EXCLUSIONS
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
    {
        let Some((name, value)) = line.split_once('\t') else {
            continue;
        };
        let mut parts = value.splitn(3, '|');
        if parts
            .next()
            .is_none_or(|kind| kind.eq_ignore_ascii_case("REG"))
        {
            continue;
        }
        let Some(root) = parts.next() else {
            continue;
        };
        let pattern = parts.next().filter(|pattern| !pattern.is_empty());
        for root in resolve_roots(root) {
            let prefix = format!(
                "{}/",
                root.to_string_lossy()
                    .replace('\\', "/")
                    .trim_end_matches('/')
                    .to_ascii_lowercase()
            );
            let (pattern, literal) = pattern.map_or((None, None), |pattern| {
                if pattern.contains(['*', '?', '[']) {
                    (Pattern::new(pattern).ok(), None)
                } else {
                    (None, Some(pattern.to_ascii_lowercase()))
                }
            });
            exclusions
                .entry(name.to_owned())
                .or_default()
                .push(Exclusion {
                    prefix,
                    pattern,
                    literal,
                });
        }
    }
    exclusions
}

fn is_broad_rule(rule: &Rule) -> bool {
    if !rule.recurse && !rule.remove_self {
        return false;
    }
    let root = rule.root.replace('\\', "/");
    let relative = root
        .strip_prefix('%')
        .and_then(|value| value.split_once('%'))
        .map_or(root.as_str(), |(_, relative)| relative)
        .trim_start_matches('/');
    relative
        .find(['*', '?'])
        .is_some_and(|wildcard| relative[..wildcard].trim_matches('/').is_empty())
}

fn rules_detected(rules: &[Rule], cache: &mut HashMap<String, bool>) -> bool {
    let mut detects = rules.iter().filter_map(|rule| rule.detect.as_deref());
    let Some(first) = detects.next() else {
        return !rules.iter().any(is_broad_rule);
    };
    first
        .split(';')
        .chain(detects.flat_map(|detects| detects.split(';')))
        .any(|detect| {
            *cache
                .entry(detect.to_owned())
                .or_insert_with(|| detect_matches(detect))
        })
}

fn detect_matches(detect: &str) -> bool {
    if let Some(path) = detect.strip_prefix("file:") {
        return resolve_roots(path).next().is_some();
    }
    #[cfg(target_os = "windows")]
    if let Some(path) = detect.strip_prefix("reg:") {
        let (hive, path) = path.split_once('\\').unwrap_or((path, ""));
        let (path, value) = path
            .rsplit_once('|')
            .map_or((path, None), |(path, value)| (path, Some(value)));
        let root = match hive.to_ascii_uppercase().as_str() {
            "HKCU" | "HKEY_CURRENT_USER" => windows_registry::CURRENT_USER,
            "HKLM" | "HKEY_LOCAL_MACHINE" => windows_registry::LOCAL_MACHINE,
            "HKU" | "HKEY_USERS" => windows_registry::USERS,
            "HKCR" | "HKEY_CLASSES_ROOT" => windows_registry::CLASSES_ROOT,
            "HKCC" | "HKEY_CURRENT_CONFIG" => windows_registry::CURRENT_CONFIG,
            _ => return false,
        };
        return root
            .open(path)
            .is_ok_and(|key| value.is_none_or(|value| key.get_value(value).is_ok()));
    }
    false
}

fn resolve_rule(
    rule: &Rule,
    allowed_roots: &[PathBuf],
    exclusions: &[Exclusion],
    seen: &mut HashSet<String>,
    paths: &mut Vec<CleanupPath>,
    prune_roots: &mut Vec<PathBuf>,
) {
    let patterns = rule
        .mask
        .split(';')
        .filter_map(|mask| Pattern::new(if mask == "*.*" { "*" } else { mask }).ok())
        .collect::<Vec<_>>();
    if patterns.is_empty() {
        return;
    }

    for root in resolve_roots(&rule.root) {
        if !is_safe_cleanup_path(&root, allowed_roots) {
            continue;
        }
        let Ok(metadata) = fs::symlink_metadata(&root) else {
            continue;
        };
        if !metadata.is_dir() || is_reparse_point(&metadata) {
            continue;
        }
        scan_directory(
            root.clone(),
            &patterns,
            rule.recurse || rule.remove_self,
            allowed_roots,
            exclusions,
            seen,
            paths,
        );
        if rule.remove_self {
            prune_roots.push(root);
        }
    }
}

fn resolve_roots(raw_root: &str) -> impl Iterator<Item = PathBuf> {
    let root = normalize_pattern(&expand_environment(raw_root));
    let options = MatchOptions {
        case_sensitive: false,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };
    glob_with(&root, options)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
}

fn scan_directory(
    root: PathBuf,
    patterns: &[Pattern],
    recurse: bool,
    allowed_roots: &[PathBuf],
    exclusions: &[Exclusion],
    seen: &mut HashSet<String>,
    paths: &mut Vec<CleanupPath>,
) {
    let options = MatchOptions {
        case_sensitive: false,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };
    let mut pending = vec![root];
    while let Some(directory) = pending.pop() {
        let Ok(entries) = fs::read_dir(directory) else {
            continue;
        };
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            let Ok(metadata) = fs::symlink_metadata(&path) else {
                continue;
            };
            if is_reparse_point(&metadata) {
                continue;
            }
            if metadata.is_dir() {
                if recurse {
                    pending.push(path);
                }
                continue;
            }
            if !metadata.is_file()
                || !patterns.iter().any(|pattern| {
                    pattern.matches_with(&entry.file_name().to_string_lossy(), options)
                })
                || !is_safe_cleanup_path(&path, allowed_roots)
                || is_protected(&path)
                || is_excluded(&path, exclusions, options)
            {
                continue;
            }
            let key = path.to_string_lossy().to_ascii_lowercase();
            if !seen.insert(key) {
                continue;
            }
            let Some(bytes) = deletable_size(&path, &metadata) else {
                continue;
            };
            paths.push(CleanupPath { path, bytes });
        }
    }
}

fn is_excluded(path: &Path, exclusions: &[Exclusion], options: MatchOptions) -> bool {
    let normalized = path
        .to_string_lossy()
        .replace('\\', "/")
        .to_ascii_lowercase();
    exclusions.iter().any(|exclusion| {
        let Some(relative) = normalized.strip_prefix(&exclusion.prefix) else {
            return false;
        };
        if let Some(pattern) = &exclusion.pattern {
            return pattern.matches_with(
                Path::new(relative)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .as_ref(),
                options,
            );
        }
        exclusion
            .literal
            .as_deref()
            .is_none_or(|literal| relative == literal)
    })
}

fn expand_environment(value: &str) -> String {
    let vars = env::vars()
        .map(|(key, value)| (key.to_ascii_uppercase(), value))
        .collect::<HashMap<_, _>>();
    let mut output = String::with_capacity(value.len());
    let mut rest = value;
    while let Some(start) = rest.find('%') {
        output.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let Some(end) = after.find('%') else {
            output.push_str(&rest[start..]);
            return output;
        };
        let key = after[..end].to_ascii_uppercase();
        if let Some(replacement) = vars.get(&key) {
            output.push_str(replacement);
        } else {
            output.push('%');
            output.push_str(&after[..=end]);
        }
        rest = &after[end + 1..];
    }
    output.push_str(rest);
    output
}

fn normalize_pattern(value: &str) -> String {
    value.replace('\\', "/").trim_end_matches('/').to_owned()
}

fn deletable_size(path: &Path, metadata: &fs::Metadata) -> Option<u64> {
    #[cfg(target_os = "windows")]
    OpenOptions::new()
        .read(true)
        .access_mode(0x0001_0000)
        .share_mode(0x7)
        .open(path)
        .ok()?;
    Some(metadata.len())
}

fn is_reparse_point(metadata: &fs::Metadata) -> bool {
    #[cfg(target_os = "windows")]
    return metadata.file_attributes() & 0x400 != 0;
    #[cfg(not(target_os = "windows"))]
    return metadata.file_type().is_symlink();
}

fn is_protected(path: &Path) -> bool {
    path.to_string_lossy()
        .replace('\\', "/")
        .to_ascii_lowercase()
        .contains("/indexeddb/chrome-extension_")
}

fn allowed_roots() -> Vec<PathBuf> {
    [
        "SYSTEMDRIVE",
        "USERPROFILE",
        "LOCALAPPDATA",
        "APPDATA",
        "PROGRAMDATA",
        "PROGRAMFILES",
        "PROGRAMFILES(X86)",
        "WINDIR",
        "SYSTEMROOT",
        "TEMP",
        "TMP",
    ]
    .into_iter()
    .filter_map(env::var_os)
    .map(PathBuf::from)
    .collect()
}

fn is_safe_cleanup_path(path: &Path, roots: &[PathBuf]) -> bool {
    path.is_absolute()
        && roots
            .iter()
            .any(|root| path.starts_with(root) && path != root)
}

fn remove_cleanup_path(target: &CleanupPath) -> Result<(), CleanupError> {
    if !is_safe_cleanup_path(&target.path, &allowed_roots()) {
        return Err(CleanupError::UnsafePath(target.path.clone()));
    }
    let metadata = fs::symlink_metadata(&target.path).map_err(|source| CleanupError::Remove {
        path: target.path.clone(),
        source,
    })?;
    let result = if metadata.is_file() {
        fs::remove_file(&target.path)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "cleanup target is no longer a file",
        ))
    };
    result.map_err(|source| CleanupError::Remove {
        path: target.path.clone(),
        source,
    })
}

fn prune_empty_dirs(root: &Path) {
    let roots = allowed_roots();
    if !is_safe_cleanup_path(root, &roots) {
        return;
    }
    let mut directories = vec![root.to_path_buf()];
    let mut cursor = 0;
    while cursor < directories.len() {
        if let Ok(entries) = fs::read_dir(&directories[cursor]) {
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if fs::symlink_metadata(&path)
                    .is_ok_and(|metadata| metadata.is_dir() && !is_reparse_point(&metadata))
                {
                    directories.push(path);
                }
            }
        }
        cursor += 1;
    }
    for directory in directories.into_iter().rev() {
        let _ = fs::remove_dir(directory);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_selection_tracks_targets() {
        let rules = parse_catalog();
        assert!(rules.len() > 10_000);
        assert!(
            rules
                .iter()
                .all(|rule| rule.category != CleanupCategory::Devices)
        );
        let detected_names = rules
            .iter()
            .filter(|rule| rule.detect.is_some())
            .map(|rule| rule.name.as_str())
            .collect::<HashSet<_>>();
        assert!(
            rules
                .iter()
                .filter(|rule| is_broad_rule(rule))
                .all(|rule| detected_names.contains(rule.name.as_str()))
        );
        assert!(CATALOG.contains("Mozilla Firefox Web Storage"));
        assert!(CATALOG.contains("Windows Temporary Files"));
        assert!(!CATALOG.contains("Saved Usernames & Passwords"));
        assert!(!CATALOG.contains("Web Browsing Cookies"));

        let mut state = CleanupState::default();
        state.apply_snapshot(CleanupSnapshot {
            targets: vec![CleanupTarget {
                id: "windows:Cache".into(),
                name: "Cache".into(),
                category: CleanupCategory::Windows,
                paths: Vec::new(),
                prune_roots: Vec::new(),
                device_instance_id: None,
                bytes: 42,
            }],
        });
        state.toggle_all();
        assert_eq!(state.selected_totals(), (1, 42));
        state.toggle_category(CleanupCategory::Windows);
        assert!(state.selected.is_empty());
    }

    #[test]
    fn unused_devices_are_separate_targets() {
        let targets = parse_unused_devices("Старое устройство\tUSB\\OLD\r\n".as_bytes());
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].category, CleanupCategory::Devices);
        assert_eq!(targets[0].device_instance_id.as_deref(), Some("USB\\OLD"));
    }

    #[test]
    fn recursive_scan_keeps_zero_byte_matches() {
        let temp = env::temp_dir();
        let root = temp.join(format!("winsentials-cleanup-{}", std::process::id()));
        let nested = root.join("nested");
        fs::create_dir_all(&nested).unwrap();
        let empty = nested.join("empty.tmp");
        fs::write(&empty, []).unwrap();
        fs::write(nested.join("protected.tmp"), b"keep").unwrap();
        fs::write(nested.join("keep.txt"), b"keep").unwrap();
        let exclusions = [Exclusion {
            prefix: format!(
                "{}/",
                nested
                    .to_string_lossy()
                    .replace('\\', "/")
                    .to_ascii_lowercase()
            ),
            pattern: None,
            literal: Some("protected.tmp".into()),
        }];
        let rule = Rule {
            category: CleanupCategory::Windows,
            name: "test".into(),
            root: root.to_string_lossy().into_owned(),
            mask: "*.tmp".into(),
            recurse: true,
            remove_self: false,
            detect: None,
        };
        let mut paths = Vec::new();
        resolve_rule(
            &rule,
            std::slice::from_ref(&temp),
            &exclusions,
            &mut HashSet::new(),
            &mut paths,
            &mut Vec::new(),
        );
        assert_eq!(paths.len(), 1);
        fs::remove_dir_all(root).unwrap();
    }
}
