use super::*;
use crate::entities::startup::{StartupEntry, StartupScope, StartupSource, StartupStatus};

#[test]
fn fallback_app_icon_does_not_duplicate_source_badge() {
    for source in [
        StartupSource::Registry,
        StartupSource::StartupFolder,
        StartupSource::Service,
        StartupSource::ScheduledTask,
    ] {
        assert_ne!(fallback_app_icon(), source.icon());
    }
}

#[test]
fn filter_state_is_any_active() {
    let empty = types::StartupFilterState::default();
    assert!(!empty.is_any_active());

    let with_scope = types::StartupFilterState {
        scope: Some(StartupScope::CurrentUser),
        ..Default::default()
    };
    assert!(with_scope.is_any_active());

    let with_source = types::StartupFilterState {
        source: Some(StartupSource::Registry),
        ..Default::default()
    };
    assert!(with_source.is_any_active());

    let with_status = types::StartupFilterState {
        status: Some(StartupStatus::Enabled),
        ..Default::default()
    };
    assert!(with_status.is_any_active());
}

fn make_dummy_entry(
    id: &str,
    name: &str,
    scope: StartupScope,
    source: StartupSource,
    status: StartupStatus,
) -> StartupEntry {
    StartupEntry {
        id: id.to_string(),
        name: name.to_string(),
        display_name: name.to_string(),
        publisher: None,
        source,
        scope,
        status,
        command: Some("C:\\app.exe".to_string()),
        target_path: None,
        icon_path: None,
        location_label: "HKCU".to_string(),
        raw_id: id.to_string(),
    }
}

#[test]
fn test_filter_entries_multi_criteria() {
    let entries = vec![
        make_dummy_entry(
            "1",
            "Discord",
            StartupScope::CurrentUser,
            StartupSource::Registry,
            StartupStatus::Enabled,
        ),
        make_dummy_entry(
            "2",
            "Steam",
            StartupScope::CurrentUser,
            StartupSource::StartupFolder,
            StartupStatus::Disabled,
        ),
        make_dummy_entry(
            "3",
            "Nvidia",
            StartupScope::AllUsers,
            StartupSource::Service,
            StartupStatus::Enabled,
        ),
        make_dummy_entry(
            "4",
            "OneDrive",
            StartupScope::AllUsers,
            StartupSource::ScheduledTask,
            StartupStatus::Disabled,
        ),
    ];

    // Filter by Scope CurrentUser
    let filter_scope = types::StartupFilterState {
        scope: Some(StartupScope::CurrentUser),
        ..Default::default()
    };
    let res: Vec<_> = entries
        .iter()
        .filter(|e| filter_scope.scope.is_none_or(|s| e.scope == s))
        .collect();
    assert_eq!(res.len(), 2);

    // Filter by Scope CurrentUser + Status Enabled
    let res_multi: Vec<_> = entries
        .iter()
        .filter(|e| {
            filter_scope.scope.is_none_or(|s| e.scope == s) && e.status == StartupStatus::Enabled
        })
        .collect();
    assert_eq!(res_multi.len(), 1);
    assert_eq!(res_multi[0].name, "Discord");

    // Filter by Source Service
    let filter_source = types::StartupFilterState {
        source: Some(StartupSource::Service),
        ..Default::default()
    };
    let res_source: Vec<_> = entries
        .iter()
        .filter(|e| filter_source.source.is_none_or(|s| e.source == s))
        .collect();
    assert_eq!(res_source.len(), 1);
    assert_eq!(res_source[0].name, "Nvidia");
}
