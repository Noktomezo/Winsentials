use crate::entities::tweaks::context_menu::{
    is_classic_context_menu_applied, is_copy_image_applied, is_create_symlink_applied,
    is_menu_show_delay_disabled, is_take_ownership_applied, set_classic_context_menu,
    set_copy_image, set_create_symlink, set_menu_show_delay_disabled, set_take_ownership,
};
use crate::entities::tweaks::explorer::{
    is_hide_gallery_applied, is_hide_home_applied, is_hide_linux_applied, is_hide_network_applied,
    is_open_to_this_pc_applied, is_wsl_installed, set_hide_gallery, set_hide_home, set_hide_linux,
    set_hide_network, set_open_to_this_pc,
};
use crate::entities::tweaks::input::{
    is_csrss_priority_applied, is_disable_mouse_acceleration_applied, set_csrss_priority,
    set_disable_mouse_acceleration,
};
use crate::entities::tweaks::interface_tweak::{
    is_disable_jpeg_compression_applied, is_remove_shortcut_arrows_applied,
    is_remove_shortcut_suffix_applied, set_disable_jpeg_compression, set_remove_shortcut_arrows,
    set_remove_shortcut_suffix,
};
use crate::entities::tweaks::network::{
    is_bbr2_applied, is_disable_ndu_applied, is_fast_send_copy_applied, is_rss_applied, set_bbr2,
    set_disable_ndu, set_fast_send_copy, set_rss,
};
use crate::entities::tweaks::security::{
    are_security_center_notifications_disabled, is_download_warning_disabled,
    is_quick_access_history_disabled, is_removable_autoplay_disabled, is_startup_delay_disabled,
    is_uac_disabled, set_download_warning_disabled, set_quick_access_history_disabled,
    set_removable_autoplay_disabled, set_security_center_notifications_disabled,
    set_startup_delay_disabled, set_uac_disabled,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum TweakCategory {
    ContextMenu,
    Explorer,
    Interface,
    Input,
    System,
    Network,
    Privacy,
    Performance,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum RestartRequirement {
    None,
    Explorer,
    Logoff,
    Reboot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SideEffectLevel {
    Low,
    Medium,
    High,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SideEffect {
    pub level: SideEffectLevel,
    pub description_key: &'static str,
}

#[allow(dead_code)]
pub struct TweakDefinition {
    pub id: &'static str,
    pub category: TweakCategory,
    pub icon: &'static str,
    pub title_key: &'static str,
    pub desc_key: &'static str,
    pub min_build: Option<u32>,
    pub max_build: Option<u32>,
    pub custom_support: Option<fn() -> bool>,
    pub restart: RestartRequirement,
    pub side_effect: Option<SideEffect>,
    pub is_applied: fn() -> bool,
    pub set_applied: fn(bool) -> Result<(), String>,
}

impl TweakDefinition {
    #[must_use]
    pub fn is_supported(&self, current_build: u32) -> bool {
        if let Some(min) = self.min_build {
            if current_build < min {
                return false;
            }
        }
        if let Some(max) = self.max_build {
            if current_build > max {
                return false;
            }
        }
        if let Some(custom) = self.custom_support {
            if !custom() {
                return false;
            }
        }
        true
    }
}

pub const ALL_TWEAKS: &[TweakDefinition] = &[
    TweakDefinition {
        id: "classic_context_menu",
        category: TweakCategory::ContextMenu,
        icon: "icons/square-menu.svg",
        title_key: "tweaks.classic_context_menu_title",
        desc_key: "tweaks.classic_context_menu_desc",
        min_build: Some(22000), // Windows 11 only
        max_build: None,
        custom_support: None,
        restart: RestartRequirement::Explorer,
        side_effect: Some(SideEffect {
            level: SideEffectLevel::Low,
            description_key: "tweaks.classic_context_menu_side_effect",
        }),
        is_applied: is_classic_context_menu_applied,
        set_applied: set_classic_context_menu,
    },
    TweakDefinition {
        id: "disable_menu_show_delay",
        category: TweakCategory::ContextMenu,
        icon: "icons/zap.svg",
        title_key: "tweaks.disable_menu_delay_title",
        desc_key: "tweaks.disable_menu_delay_desc",
        min_build: None,
        max_build: None,
        custom_support: None,
        restart: RestartRequirement::None,
        side_effect: None,
        is_applied: is_menu_show_delay_disabled,
        set_applied: set_menu_show_delay_disabled,
    },
    TweakDefinition {
        id: "create_symlink",
        category: TweakCategory::ContextMenu,
        icon: "icons/link.svg",
        title_key: "tweaks.create_symlink_title",
        desc_key: "tweaks.create_symlink_desc",
        min_build: None,
        max_build: None,
        custom_support: None,
        restart: RestartRequirement::None,
        side_effect: None,
        is_applied: is_create_symlink_applied,
        set_applied: set_create_symlink,
    },
    TweakDefinition {
        id: "take_ownership",
        category: TweakCategory::ContextMenu,
        icon: "icons/shield-check.svg",
        title_key: "tweaks.take_ownership_title",
        desc_key: "tweaks.take_ownership_desc",
        min_build: None,
        max_build: None,
        custom_support: None,
        restart: RestartRequirement::None,
        side_effect: Some(SideEffect {
            level: SideEffectLevel::Medium,
            description_key: "tweaks.take_ownership_side_effect",
        }),
        is_applied: is_take_ownership_applied,
        set_applied: set_take_ownership,
    },
    TweakDefinition {
        id: "copy_image",
        category: TweakCategory::ContextMenu,
        icon: "icons/copy-image.svg",
        title_key: "tweaks.copy_image_title",
        desc_key: "tweaks.copy_image_desc",
        min_build: None,
        max_build: None,
        custom_support: None,
        restart: RestartRequirement::None,
        side_effect: None,
        is_applied: is_copy_image_applied,
        set_applied: set_copy_image,
    },
    TweakDefinition {
        id: "open_to_this_pc",
        category: TweakCategory::Explorer,
        icon: "icons/monitor.svg",
        title_key: "tweaks.open_to_this_pc_title",
        desc_key: "tweaks.open_to_this_pc_desc",
        min_build: None,
        max_build: None,
        custom_support: None,
        restart: RestartRequirement::None,
        side_effect: None,
        is_applied: is_open_to_this_pc_applied,
        set_applied: set_open_to_this_pc,
    },
    TweakDefinition {
        id: "hide_network_nav_pane",
        category: TweakCategory::Explorer,
        icon: "icons/network.svg",
        title_key: "tweaks.hide_network_title",
        desc_key: "tweaks.hide_network_desc",
        min_build: None,
        max_build: None,
        custom_support: None,
        restart: RestartRequirement::None,
        side_effect: None,
        is_applied: is_hide_network_applied,
        set_applied: set_hide_network,
    },
    TweakDefinition {
        id: "hide_home_nav_pane",
        category: TweakCategory::Explorer,
        icon: "icons/house.svg",
        title_key: "tweaks.hide_home_title",
        desc_key: "tweaks.hide_home_desc",
        min_build: Some(22000), // Windows 11 only
        max_build: None,
        custom_support: None,
        restart: RestartRequirement::None,
        side_effect: None,
        is_applied: is_hide_home_applied,
        set_applied: set_hide_home,
    },
    TweakDefinition {
        id: "hide_gallery_nav_pane",
        category: TweakCategory::Explorer,
        icon: "icons/images.svg",
        title_key: "tweaks.hide_gallery_title",
        desc_key: "tweaks.hide_gallery_desc",
        min_build: Some(22000), // Windows 11 only
        max_build: None,
        custom_support: None,
        restart: RestartRequirement::None,
        side_effect: None,
        is_applied: is_hide_gallery_applied,
        set_applied: set_hide_gallery,
    },
    TweakDefinition {
        id: "hide_linux_nav_pane",
        category: TweakCategory::Explorer,
        icon: "icons/terminal.svg",
        title_key: "tweaks.hide_linux_title",
        desc_key: "tweaks.hide_linux_desc",
        min_build: None,
        max_build: None,
        custom_support: Some(is_wsl_installed),
        restart: RestartRequirement::None,
        side_effect: None,
        is_applied: is_hide_linux_applied,
        set_applied: set_hide_linux,
    },
    TweakDefinition {
        id: "remove_shortcut_arrows",
        category: TweakCategory::Interface,
        icon: "icons/arrow-up-right.svg",
        title_key: "tweaks.remove_shortcut_arrows_title",
        desc_key: "tweaks.remove_shortcut_arrows_desc",
        min_build: None,
        max_build: None,
        custom_support: None,
        restart: RestartRequirement::None,
        side_effect: None,
        is_applied: is_remove_shortcut_arrows_applied,
        set_applied: set_remove_shortcut_arrows,
    },
    TweakDefinition {
        id: "remove_shortcut_suffix",
        category: TweakCategory::Interface,
        icon: "icons/file-symlink.svg",
        title_key: "tweaks.remove_shortcut_suffix_title",
        desc_key: "tweaks.remove_shortcut_suffix_desc",
        min_build: None,
        max_build: None,
        custom_support: None,
        restart: RestartRequirement::None,
        side_effect: None,
        is_applied: is_remove_shortcut_suffix_applied,
        set_applied: set_remove_shortcut_suffix,
    },
    TweakDefinition {
        id: "disable_jpeg_compression",
        category: TweakCategory::Interface,
        icon: "icons/image.svg",
        title_key: "tweaks.disable_jpeg_compression_title",
        desc_key: "tweaks.disable_jpeg_compression_desc",
        min_build: None,
        max_build: None,
        custom_support: None,
        restart: RestartRequirement::None,
        side_effect: None,
        is_applied: is_disable_jpeg_compression_applied,
        set_applied: set_disable_jpeg_compression,
    },
    TweakDefinition {
        id: "disable_mouse_acceleration",
        category: TweakCategory::Input,
        icon: "icons/mouse-pointer.svg",
        title_key: "tweaks.disable_mouse_acceleration_title",
        desc_key: "tweaks.disable_mouse_acceleration_desc",
        min_build: None,
        max_build: None,
        custom_support: None,
        restart: RestartRequirement::None,
        side_effect: None,
        is_applied: is_disable_mouse_acceleration_applied,
        set_applied: set_disable_mouse_acceleration,
    },
    TweakDefinition {
        id: "csrss_priority",
        category: TweakCategory::Input,
        icon: "icons/chevrons-up.svg",
        title_key: "tweaks.csrss_priority_title",
        desc_key: "tweaks.csrss_priority_desc",
        min_build: None,
        max_build: None,
        custom_support: None,
        restart: RestartRequirement::Reboot,
        side_effect: None,
        is_applied: is_csrss_priority_applied,
        set_applied: set_csrss_priority,
    },
    TweakDefinition {
        id: "disable_uac",
        category: TweakCategory::System,
        icon: "icons/shield-off.svg",
        title_key: "tweaks.disable_uac_title",
        desc_key: "tweaks.disable_uac_desc",
        min_build: None,
        max_build: None,
        custom_support: None,
        restart: RestartRequirement::Reboot,
        side_effect: Some(SideEffect {
            level: SideEffectLevel::Low,
            description_key: "tweaks.disable_uac_side_effect",
        }),
        is_applied: is_uac_disabled,
        set_applied: set_uac_disabled,
    },
    TweakDefinition {
        id: "disable_download_warning",
        category: TweakCategory::System,
        icon: "icons/file-down.svg",
        title_key: "tweaks.disable_download_warning_title",
        desc_key: "tweaks.disable_download_warning_desc",
        min_build: None,
        max_build: None,
        custom_support: None,
        restart: RestartRequirement::None,
        side_effect: Some(SideEffect {
            level: SideEffectLevel::Medium,
            description_key: "tweaks.disable_download_warning_side_effect",
        }),
        is_applied: is_download_warning_disabled,
        set_applied: set_download_warning_disabled,
    },
    TweakDefinition {
        id: "disable_removable_autoplay",
        category: TweakCategory::System,
        icon: "icons/usb.svg",
        title_key: "tweaks.disable_removable_autoplay_title",
        desc_key: "tweaks.disable_removable_autoplay_desc",
        min_build: None,
        max_build: None,
        custom_support: None,
        restart: RestartRequirement::Explorer,
        side_effect: None,
        is_applied: is_removable_autoplay_disabled,
        set_applied: set_removable_autoplay_disabled,
    },
    TweakDefinition {
        id: "disable_quick_access_history",
        category: TweakCategory::System,
        icon: "icons/clock-arrow-down.svg",
        title_key: "tweaks.disable_quick_access_history_title",
        desc_key: "tweaks.disable_quick_access_history_desc",
        min_build: None,
        max_build: None,
        custom_support: None,
        restart: RestartRequirement::Explorer,
        side_effect: None,
        is_applied: is_quick_access_history_disabled,
        set_applied: set_quick_access_history_disabled,
    },
    TweakDefinition {
        id: "disable_security_center_notifications",
        category: TweakCategory::System,
        icon: "icons/bell-off.svg",
        title_key: "tweaks.disable_security_center_notifications_title",
        desc_key: "tweaks.disable_security_center_notifications_desc",
        min_build: Some(18362),
        max_build: None,
        custom_support: None,
        restart: RestartRequirement::Logoff,
        side_effect: Some(SideEffect {
            level: SideEffectLevel::Medium,
            description_key: "tweaks.disable_security_center_notifications_side_effect",
        }),
        is_applied: are_security_center_notifications_disabled,
        set_applied: set_security_center_notifications_disabled,
    },
    TweakDefinition {
        id: "disable_startup_delay",
        category: TweakCategory::System,
        icon: "icons/rocket.svg",
        title_key: "tweaks.disable_startup_delay_title",
        desc_key: "tweaks.disable_startup_delay_desc",
        min_build: None,
        max_build: None,
        custom_support: None,
        restart: RestartRequirement::Logoff,
        side_effect: Some(SideEffect {
            level: SideEffectLevel::Low,
            description_key: "tweaks.disable_startup_delay_side_effect",
        }),
        is_applied: is_startup_delay_disabled,
        set_applied: set_startup_delay_disabled,
    },
    TweakDefinition {
        id: "bbr2",
        category: TweakCategory::Network,
        icon: "icons/gauge.svg",
        title_key: "tweaks.bbr2_title",
        desc_key: "tweaks.bbr2_desc",
        min_build: Some(22621),
        max_build: None,
        custom_support: None,
        restart: RestartRequirement::None,
        side_effect: None,
        is_applied: is_bbr2_applied,
        set_applied: set_bbr2,
    },
    TweakDefinition {
        id: "rss",
        category: TweakCategory::Network,
        icon: "icons/cpu.svg",
        title_key: "tweaks.rss_title",
        desc_key: "tweaks.rss_desc",
        min_build: None,
        max_build: None,
        custom_support: None,
        restart: RestartRequirement::None,
        side_effect: None,
        is_applied: is_rss_applied,
        set_applied: set_rss,
    },
    TweakDefinition {
        id: "fast_send_copy",
        category: TweakCategory::Network,
        icon: "icons/zap.svg",
        title_key: "tweaks.fast_send_copy_title",
        desc_key: "tweaks.fast_send_copy_desc",
        min_build: None,
        max_build: None,
        custom_support: None,
        restart: RestartRequirement::Reboot,
        side_effect: None,
        is_applied: is_fast_send_copy_applied,
        set_applied: set_fast_send_copy,
    },
    TweakDefinition {
        id: "disable_ndu",
        category: TweakCategory::Network,
        icon: "icons/chart-network.svg",
        title_key: "tweaks.disable_ndu_title",
        desc_key: "tweaks.disable_ndu_desc",
        min_build: None,
        max_build: None,
        custom_support: None,
        restart: RestartRequirement::Reboot,
        side_effect: Some(SideEffect {
            level: SideEffectLevel::Low,
            description_key: "tweaks.disable_ndu_side_effect",
        }),
        is_applied: is_disable_ndu_applied,
        set_applied: set_disable_ndu,
    },
];

#[must_use]
pub fn get_all_tweaks() -> &'static [TweakDefinition] {
    ALL_TWEAKS
}

#[must_use]
pub fn count_applied_tweaks(build: u32) -> (usize, usize) {
    let mut applied = 0;
    let mut total_supported = 0;

    for tweak in ALL_TWEAKS {
        if tweak.is_supported(build) {
            total_supported += 1;
            if (tweak.is_applied)() {
                applied += 1;
            }
        }
    }

    (applied, total_supported)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_tweaks_only_flag_meaningful_side_effects() {
        let system: Vec<_> = ALL_TWEAKS
            .iter()
            .filter(|tweak| tweak.category == TweakCategory::System)
            .collect();

        assert_eq!(system.len(), 6);
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

        assert_eq!(network.len(), 4);
        assert_eq!(network[0].id, "bbr2");
        assert_eq!(network[1].id, "rss");
        assert_eq!(network[2].id, "fast_send_copy");
        assert_eq!(network[3].id, "disable_ndu");
    }

    #[test]
    fn input_tweaks_are_registered() {
        let input: Vec<_> = ALL_TWEAKS
            .iter()
            .filter(|tweak| tweak.category == TweakCategory::Input)
            .collect();

        assert_eq!(input.len(), 2);
        assert_eq!(input[0].id, "disable_mouse_acceleration");
        assert_eq!(input[1].id, "csrss_priority");
    }
}
