use super::*;

#[test]
fn test_snapkey_presets_enum() {
    assert_eq!(SnapKeyPreset::ALL.len(), 5);
    for preset in SnapKeyPreset::ALL {
        assert_eq!(SnapKeyPreset::from_id(preset.id()), Some(preset));
    }
    assert_eq!(SnapKeyPreset::from_id("invalid"), None);
}

#[test]
fn test_snapkey_wasd_socd_counter_strafe() {
    let mut state = SnapKeyState::new();
    let mut sent = Vec::new();
    state.set_preset(SnapKeyPreset::Wasd, |vk, down| sent.push((vk, down)));
    sent.clear();

    // 1. Press A (65)
    state.handle_key_down(65, |vk, down| sent.push((vk, down)));
    assert_eq!(sent, vec![(65, true)]);
    sent.clear();

    // 2. While holding A, press D (68) -> Opposing cardinal direction
    // SnappyTappy behavior: D down is sent, A is immediately released!
    state.handle_key_down(68, |vk, down| sent.push((vk, down)));
    assert_eq!(sent, vec![(68, true), (65, false)]);
    sent.clear();

    // 3. Release D while still holding A
    // SnappyTappy behavior: D up is sent, A is immediately re-pressed!
    state.handle_key_up(68, |vk, down| sent.push((vk, down)));
    assert_eq!(sent, vec![(68, false), (65, true)]);
    sent.clear();

    // 4. Release A
    state.handle_key_up(65, |vk, down| sent.push((vk, down)));
    assert_eq!(sent, vec![(65, false)]);
}

#[test]
fn test_snapkey_press_both_release_first() {
    let mut state = SnapKeyState::new();
    let mut sent = Vec::new();
    state.set_preset(SnapKeyPreset::Wasd, |vk, down| sent.push((vk, down)));
    sent.clear();

    // 1. Press A (65)
    state.handle_key_down(65, |vk, down| sent.push((vk, down)));
    assert_eq!(sent, vec![(65, true)]);
    sent.clear();

    // 2. Press D (68) -> D down sent, A released
    state.handle_key_down(68, |vk, down| sent.push((vk, down)));
    assert_eq!(sent, vec![(68, true), (65, false)]);
    sent.clear();

    // 3. Release A while D is still held
    state.handle_key_up(65, |vk, down| sent.push((vk, down)));
    assert_eq!(sent, vec![(65, false)]);
    sent.clear();

    // 4. Release D
    state.handle_key_up(68, |vk, down| sent.push((vk, down)));
    assert_eq!(sent, vec![(68, false)]);
}

#[test]
fn test_snapkey_preset_switching_releases_held_keys() {
    let mut state = SnapKeyState::new();
    let mut sent = Vec::new();
    state.set_preset(SnapKeyPreset::Wasd, |vk, down| sent.push((vk, down)));
    sent.clear();

    // Press A
    state.handle_key_down(65, |vk, down| sent.push((vk, down)));
    assert_eq!(sent, vec![(65, true)]);
    sent.clear();

    // Switch to ArrowKeys while A is held
    state.set_preset(SnapKeyPreset::ArrowKeys, |vk, down| sent.push((vk, down)));
    // A must be released during preset transition
    assert_eq!(sent, vec![(65, false)]);
    assert!(!state.is_registered(65));
    assert!(state.is_registered(37)); // Left arrow is now registered
}

#[test]
fn test_snapkey_independent_groups_movement_and_strafe() {
    let mut state = SnapKeyState::new();
    let mut sent = Vec::new();
    state.set_preset(SnapKeyPreset::Wasd, |vk, down| sent.push((vk, down)));
    sent.clear();

    // Press W (87) [Group 1 - Forward]
    state.handle_key_down(87, |vk, down| sent.push((vk, down)));
    assert_eq!(sent, vec![(87, true)]);
    sent.clear();

    // Press A (65) [Group 0 - Strafe] - Should NOT affect W!
    state.handle_key_down(65, |vk, down| sent.push((vk, down)));
    assert_eq!(sent, vec![(65, true)]);
    sent.clear();

    // Press S (83) [Group 1 - Backward] - Should interrupt W, but NOT A!
    state.handle_key_down(83, |vk, down| sent.push((vk, down)));
    assert_eq!(sent, vec![(83, true), (87, false)]);
    sent.clear();

    // Clean release
    state.handle_key_up(83, |vk, down| sent.push((vk, down)));
    state.handle_key_up(87, |vk, down| sent.push((vk, down)));
    state.handle_key_up(65, |vk, down| sent.push((vk, down)));
}

#[test]
fn test_snapkey_typematic_repeat_active_key() {
    let mut state = SnapKeyState::new();
    let mut sent = Vec::new();
    state.set_preset(SnapKeyPreset::Wasd, |vk, down| sent.push((vk, down)));
    sent.clear();

    // Initial press of 'A' (65)
    state.handle_key_down(65, |vk, down| sent.push((vk, down)));
    assert_eq!(sent, vec![(65, true)]);
    sent.clear();

    // Typematic repeat 1
    state.handle_key_down(65, |vk, down| sent.push((vk, down)));
    assert_eq!(sent, vec![(65, true)]);
    sent.clear();

    // Typematic repeat 2
    state.handle_key_down(65, |vk, down| sent.push((vk, down)));
    assert_eq!(sent, vec![(65, true)]);
    sent.clear();

    // Release 'A'
    state.handle_key_up(65, |vk, down| sent.push((vk, down)));
    assert_eq!(sent, vec![(65, false)]);
}

#[test]
fn test_snapkey_typematic_repeat_suppressed_key_ignored() {
    let mut state = SnapKeyState::new();
    let mut sent = Vec::new();
    state.set_preset(SnapKeyPreset::Wasd, |vk, down| sent.push((vk, down)));
    sent.clear();

    // 1. Hold 'A' (65)
    state.handle_key_down(65, |vk, down| sent.push((vk, down)));
    assert_eq!(sent, vec![(65, true)]);
    sent.clear();

    // 2. Press opposing 'D' (68) -> D active, A suppressed
    state.handle_key_down(68, |vk, down| sent.push((vk, down)));
    assert_eq!(sent, vec![(68, true), (65, false)]);
    sent.clear();

    // 3. Repeat message arrives for suppressed 'A' -> must NOT emit repeat!
    state.handle_key_down(65, |vk, down| sent.push((vk, down)));
    assert!(sent.is_empty());

    // 4. Repeat message arrives for active 'D' -> must emit repeat!
    state.handle_key_down(68, |vk, down| sent.push((vk, down)));
    assert_eq!(sent, vec![(68, true)]);
}

#[test]
fn test_snapkey_sync_ghost_keys_releases_unheld_active_key() {
    let mut state = SnapKeyState::new();
    let mut sent = Vec::new();
    state.set_preset(SnapKeyPreset::Wasd, |vk, down| sent.push((vk, down)));
    sent.clear();

    // Press 'A'
    state.handle_key_down(65, |vk, down| sent.push((vk, down)));
    assert_eq!(sent, vec![(65, true)]);
    sent.clear();

    // Simulate focus switch or missed keyup: physical key 'A' is now released.
    // An incoming keystroke (e.g. Backspace = 8) triggers sync_ghost_keys.
    state.sync_ghost_keys(8, |_vk| false, |vk, down| sent.push((vk, down)));

    // 'A' must be automatically released in Windows and state cleared!
    assert_eq!(sent, vec![(65, false)]);
    assert_eq!(state.groups[0].active_key, 0);
    assert!(!state.keys[65].key_down);
}

#[test]
fn test_snapkey_sync_ghost_keys_preserves_physically_held_keys() {
    let mut state = SnapKeyState::new();
    let mut sent = Vec::new();
    state.set_preset(SnapKeyPreset::Wasd, |vk, down| sent.push((vk, down)));
    sent.clear();

    // Press 'A'
    state.handle_key_down(65, |vk, down| sent.push((vk, down)));
    sent.clear();

    // Physical key 'A' is still held down
    state.sync_ghost_keys(8, |vk| vk == 65, |vk, down| sent.push((vk, down)));

    // Nothing should be released
    assert!(sent.is_empty());
    assert_eq!(state.groups[0].active_key, 65);
    assert!(state.keys[65].key_down);
}

#[test]
fn test_snapkey_sync_ghost_keys_promotes_previous_if_active_ghost() {
    let mut state = SnapKeyState::new();
    let mut sent = Vec::new();
    state.set_preset(SnapKeyPreset::Wasd, |vk, down| sent.push((vk, down)));
    sent.clear();

    // Press 'A' then 'D'
    state.handle_key_down(65, |vk, down| sent.push((vk, down)));
    state.handle_key_down(68, |vk, down| sent.push((vk, down)));
    sent.clear();

    // 'D' was physically released outside hook, but 'A' is still held down
    state.sync_ghost_keys(8, |vk| vk == 65, |vk, down| sent.push((vk, down)));

    // 'D' released, and 'A' re-pressed
    assert_eq!(sent, vec![(68, false), (65, true)]);
    assert_eq!(state.groups[0].active_key, 65);
    assert_eq!(state.groups[0].previous_key, 0);
}
