use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::{self, OpenOptions};
use std::path::{Path, PathBuf};

#[cfg(target_os = "windows")]
use std::os::windows::fs::{MetadataExt, OpenOptionsExt};

use glob::{MatchOptions, Pattern};

use super::devices::{remove_unused_device, scan_unused_devices};
use super::rules::{
    Exclusion, Rule, parse_catalog, parse_exclusions, resolve_roots, rules_detected,
};
use super::types::{
    CleanupCategory, CleanupError, CleanupPath, CleanupReport, CleanupSnapshot, CleanupTarget,
};

pub fn scan_category_targets(category: CleanupCategory) -> Vec<CleanupTarget> {
    if category == CleanupCategory::Devices {
        return scan_unused_devices();
    }

    let mut grouped: HashMap<String, Vec<Rule>> = HashMap::new();
    for rule in parse_catalog() {
        if rule.category == category {
            grouped.entry(rule.name.clone()).or_default().push(rule);
        }
    }

    let allowed_roots = allowed_roots();
    let exclusions = parse_exclusions();
    let mut detection_cache = HashMap::new();
    let mut targets = grouped
        .into_iter()
        .filter_map(|(name, rules)| {
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
    targets
}

pub fn scan_cleanup_targets() -> CleanupSnapshot {
    let mut targets = Vec::new();
    for category in CleanupCategory::ALL {
        if category != CleanupCategory::Devices {
            targets.extend(scan_category_targets(category));
        }
    }
    CleanupSnapshot { targets }
}

pub fn clean_selected(snapshot: &CleanupSnapshot, selected: &HashSet<String>) -> CleanupReport {
    let mut report = CleanupReport::default();
    for target in snapshot
        .targets
        .iter()
        .filter(|target| selected.contains(&target.id))
    {
        clean_target(target, &mut report);
    }
    report
}

pub fn clean_category_selected(
    snapshot: &CleanupSnapshot,
    selected: &HashSet<String>,
    category: CleanupCategory,
) -> CleanupReport {
    let mut report = CleanupReport::default();
    for target in snapshot
        .targets
        .iter()
        .filter(|target| target.category == category && selected.contains(&target.id))
    {
        clean_target(target, &mut report);
    }
    report
}

fn clean_target(target: &CleanupTarget, report: &mut CleanupReport) {
    if let Some(instance_id) = &target.device_instance_id {
        match remove_unused_device(instance_id) {
            Ok(()) => report.removed_paths += 1,
            Err(error) => {
                eprintln!("cleanup: {error}");
                report.failures += 1;
            }
        }
        return;
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

pub(crate) fn resolve_rule(
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

pub(crate) fn allowed_roots() -> Vec<PathBuf> {
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

pub(crate) fn is_safe_cleanup_path(path: &Path, roots: &[PathBuf]) -> bool {
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
