use super::*;

#[test]
fn system_tweaks_only_flag_meaningful_side_effects() {
    let system: Vec<_> = ALL_TWEAKS
        .iter()
        .filter(|tweak| tweak.category == TweakCategory::System)
        .collect();

    assert_eq!(system.len(), 8);
    assert_eq!(system[0].side_effect.unwrap().level, SideEffectLevel::Low);
    assert_eq!(
        system[1].side_effect.unwrap().level,
        SideEffectLevel::Medium
    );
    assert!(system[2].side_effect.is_none());
    assert!(system[3].side_effect.is_none());
    assert_eq!(
        system[4].side_effect.unwrap().level,
        SideEffectLevel::Medium
    );
    assert_eq!(system[5].side_effect.unwrap().level, SideEffectLevel::Low);
    assert!(system[6].side_effect.is_none());
    assert_eq!(system[7].side_effect.unwrap().level, SideEffectLevel::Low);
}

#[test]
fn classic_context_menu_flags_third_party_patch_conflicts() {
    let tweak = ALL_TWEAKS
        .iter()
        .find(|tweak| tweak.id == "classic_context_menu")
        .unwrap();
    let side_effect = tweak.side_effect.unwrap();

    assert_eq!(side_effect.level, SideEffectLevel::Low);
    assert_eq!(
        side_effect.description_key,
        "tweaks.classic_context_menu_side_effect"
    );
}

#[test]
fn take_ownership_flags_system_acl_risk() {
    let tweak = ALL_TWEAKS
        .iter()
        .find(|tweak| tweak.id == "take_ownership")
        .unwrap();
    let side_effect = tweak.side_effect.unwrap();

    assert_eq!(side_effect.level, SideEffectLevel::Medium);
    assert_eq!(
        side_effect.description_key,
        "tweaks.take_ownership_side_effect"
    );
}

#[test]
fn network_tweaks_are_registered() {
    let network: Vec<_> = ALL_TWEAKS
        .iter()
        .filter(|tweak| tweak.category == TweakCategory::Network)
        .collect();

    assert_eq!(network.len(), 5);
    assert_eq!(network[0].id, "bbr2");
    assert_eq!(network[1].id, "rss");
    assert_eq!(network[2].id, "fast_send_copy");
    assert_eq!(network[3].id, "disable_ndu");
    assert_eq!(network[4].id, "disable_network_power_saving");
}

#[test]
fn disable_network_power_saving_tweak_is_valid() {
    let tweak = ALL_TWEAKS
        .iter()
        .find(|tweak| tweak.id == "disable_network_power_saving")
        .unwrap();
    assert_eq!(tweak.category, TweakCategory::Network);
    assert_eq!(tweak.restart, RestartRequirement::Reboot);
    assert_eq!(tweak.side_effect.unwrap().level, SideEffectLevel::Low);
    assert_eq!(
        tweak.side_effect.unwrap().description_key,
        "tweaks.disable_network_power_saving_side_effect"
    );
}

#[test]
fn input_tweaks_are_registered() {
    let input: Vec<_> = ALL_TWEAKS
        .iter()
        .filter(|tweak| tweak.category == TweakCategory::Input)
        .collect();

    assert_eq!(input.len(), 3);
    assert_eq!(input[0].id, "disable_mouse_acceleration");
    assert_eq!(input[1].id, "csrss_priority");
    assert_eq!(input[2].id, "raw_mouse_throttle");
}

#[test]
fn raw_mouse_throttle_tweak_is_valid() {
    let tweak = ALL_TWEAKS
        .iter()
        .find(|tweak| tweak.id == "raw_mouse_throttle")
        .unwrap();
    assert_eq!(tweak.category, TweakCategory::Input);
    assert_eq!(tweak.restart, RestartRequirement::Logoff);
    assert_eq!(tweak.min_build, Some(22000));
    assert!(tweak.side_effect.is_none());
}

#[test]
fn disable_usb_power_saving_tweak_is_valid() {
    let tweak = ALL_TWEAKS
        .iter()
        .find(|tweak| tweak.id == "disable_usb_power_saving")
        .unwrap();
    assert_eq!(tweak.category, TweakCategory::System);
    assert_eq!(tweak.restart, RestartRequirement::Reboot);
    assert_eq!(tweak.side_effect.unwrap().level, SideEffectLevel::Low);
    assert_eq!(
        tweak.side_effect.unwrap().description_key,
        "tweaks.disable_usb_power_saving_side_effect"
    );
}
