use super::*;

#[test]
fn test_search_tweaks_empty_query() {
    let results = search_tweaks("", 22631);
    assert!(results.is_empty());

    let results = search_tweaks("   ", 22631);
    assert!(results.is_empty());
}

#[test]
fn test_search_tweaks_finds_results() {
    let results = search_tweaks("context", 22631);
    assert!(!results.is_empty());
    assert!(results.iter().any(|r| r.route == AppRoute::ContextMenu));
    assert!(results.len() <= MAX_SEARCH_RESULTS);
}

#[test]
fn test_search_tweaks_limited_to_five_items() {
    let results = search_tweaks("e", 22631);
    assert_eq!(results.len(), MAX_SEARCH_RESULTS);
}

#[test]
fn test_search_tweaks_unsupported_build_filtered() {
    let results_old = search_tweaks("classic", 19041);
    let results_new = search_tweaks("classic", 22631);
    assert!(results_new.len() >= results_old.len());
}

#[test]
fn test_category_to_route_coverage() {
    assert_eq!(
        category_to_route(TweakCategory::ContextMenu),
        AppRoute::ContextMenu
    );
    assert_eq!(
        category_to_route(TweakCategory::Explorer),
        AppRoute::Explorer
    );
    assert_eq!(
        category_to_route(TweakCategory::Interface),
        AppRoute::Interface
    );
    assert_eq!(category_to_route(TweakCategory::Input), AppRoute::Input);
    assert_eq!(category_to_route(TweakCategory::System), AppRoute::System);
    assert_eq!(
        category_to_route(TweakCategory::Network),
        AppRoute::NetworkTweaks
    );
    assert_eq!(category_to_route(TweakCategory::Privacy), AppRoute::Privacy);
}

#[test]
fn test_search_snapkey_found_by_name_and_socd() {
    let by_name = search_tweaks("snapkey", 22631);
    assert!(!by_name.is_empty());
    assert_eq!(by_name[0].tweak_id, "snapkey");
    assert_eq!(by_name[0].route, AppRoute::Input);

    let by_socd = search_tweaks("socd", 22631);
    assert!(!by_socd.is_empty());
    assert_eq!(by_socd[0].tweak_id, "snapkey");

    let by_snap_tap = search_tweaks("snap tap", 22631);
    assert!(!by_snap_tap.is_empty());
    assert!(by_snap_tap.iter().any(|r| r.tweak_id == "snapkey"));
}

#[test]
fn test_search_ctf_optimization_found() {
    let by_ctf = search_tweaks("ctf", 22631);
    assert!(!by_ctf.is_empty());
    assert_eq!(by_ctf[0].tweak_id, "ctf_optimization");
    assert_eq!(by_ctf[0].route, AppRoute::Input);

    let by_ctfmon = search_tweaks("ctfmon", 22631);
    assert!(!by_ctfmon.is_empty());
    assert_eq!(by_ctfmon[0].tweak_id, "ctf_optimization");
}

#[test]
fn test_search_keyboard_repeat_found() {
    let by_repeat = search_tweaks("repeat", 22631);
    assert!(!by_repeat.is_empty());
    assert!(by_repeat.iter().any(|r| r.tweak_id == "keyboard_repeat"));

    let by_keyboard = search_tweaks("keyboard", 22631);
    assert!(!by_keyboard.is_empty());
    assert!(by_keyboard.iter().any(|r| r.tweak_id == "keyboard_repeat"));
}

#[test]
fn test_levenshtein_distance_typos_match() {
    // 1 typo edit: snappkey -> snapkey
    let r1 = search_tweaks("snappkey", 22631);
    assert!(!r1.is_empty());
    assert_eq!(r1[0].tweak_id, "snapkey");

    // 2 typo edits: scod -> socd -> snapkey
    let r2 = search_tweaks("scod", 22631);
    assert!(!r2.is_empty());
    assert_eq!(r2[0].tweak_id, "snapkey");

    // 2 typo edits: keybaord -> keyboard -> keyboard_repeat
    let r3 = search_tweaks("keybaord", 22631);
    assert!(!r3.is_empty());
    assert!(r3.iter().any(|r| r.tweak_id == "keyboard_repeat"));
}

#[test]
fn test_english_queries_in_russian_locale() {
    rust_i18n::set_locale("ru");

    let mouse_accel = search_tweaks("mouse acceleration", 22631);
    assert!(!mouse_accel.is_empty());
    assert_eq!(mouse_accel[0].tweak_id, "disable_mouse_acceleration");

    let fast_send = search_tweaks("fast send", 22631);
    assert!(!fast_send.is_empty());
    assert_eq!(fast_send[0].tweak_id, "fast_send_copy");

    let telemetry = search_tweaks("telemetry", 22631);
    assert!(!telemetry.is_empty());
    assert!(
        telemetry
            .iter()
            .any(|r| r.tweak_id == "telemetry_and_diagnostics")
    );
}

#[test]
fn test_russian_queries_in_english_locale() {
    rust_i18n::set_locale("en");

    let ctx = search_tweaks("контекстное меню", 22631);
    assert!(!ctx.is_empty());
    assert_eq!(ctx[0].tweak_id, "classic_context_menu");
}
