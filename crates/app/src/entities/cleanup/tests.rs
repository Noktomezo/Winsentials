use std::collections::HashSet;
use std::env;
use std::fs;

use super::rules::{CATALOG, Exclusion, Rule, is_broad_rule, parse_catalog};
use super::scanner::{parse_unused_devices, resolve_rule};
use super::types::{CleanupCategory, CleanupSnapshot, CleanupState, CleanupTarget};

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