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
fn test_search_snapkey_found_by_name_and_aliases() {
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

    let by_null_bind = search_tweaks("null bind", 22631);
    assert!(!by_null_bind.is_empty());
    assert!(by_null_bind.iter().any(|r| r.tweak_id == "snapkey"));

    let by_strafe_ru = search_tweaks("стрейф", 22631);
    assert!(!by_strafe_ru.is_empty());
    assert!(by_strafe_ru.iter().any(|r| r.tweak_id == "snapkey"));
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

    let by_delay = search_tweaks("repeat delay", 22631);
    assert!(!by_delay.is_empty());
    assert_eq!(by_delay[0].tweak_id, "keyboard_repeat");
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

    let uac = search_tweaks("uac", 22631);
    assert!(!uac.is_empty());
    assert_eq!(uac[0].tweak_id, "disable_uac");

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

    let mouse = search_tweaks("мышь", 22631);
    assert!(!mouse.is_empty());
    assert!(
        mouse
            .iter()
            .any(|r| r.tweak_id == "disable_mouse_acceleration"
                || r.tweak_id == "raw_mouse_throttle")
    );
}
