use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

use glob::{MatchOptions, Pattern, glob_with};

use super::types::CleanupCategory;

pub(crate) const CATALOG: &str = include_str!("catalog.tsv");
pub(crate) const EXCLUSIONS: &str = include_str!("exclusions.tsv");

#[derive(Clone)]
pub(crate) struct Rule {
    pub(crate) category: CleanupCategory,
    pub(crate) name: String,
    pub(crate) root: String,
    pub(crate) mask: String,
    pub(crate) recurse: bool,
    pub(crate) remove_self: bool,
    pub(crate) detect: Option<String>,
}

pub(crate) struct Exclusion {
    pub(crate) prefix: String,
    pub(crate) pattern: Option<Pattern>,
    pub(crate) literal: Option<String>,
}

pub(crate) fn parse_catalog() -> Vec<Rule> {
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

pub(crate) fn parse_exclusions() -> HashMap<String, Vec<Exclusion>> {
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

pub(crate) fn is_broad_rule(rule: &Rule) -> bool {
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

pub(crate) fn rules_detected(rules: &[Rule], cache: &mut HashMap<String, bool>) -> bool {
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

pub(crate) fn detect_matches(detect: &str) -> bool {
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

pub(crate) fn resolve_roots(raw_root: &str) -> impl Iterator<Item = PathBuf> {
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

pub(crate) fn expand_environment(value: &str) -> String {
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

pub(crate) fn normalize_pattern(value: &str) -> String {
    value.replace('\\', "/").trim_end_matches('/').to_owned()
}