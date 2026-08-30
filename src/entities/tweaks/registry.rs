use crate::entities::tweaks::context_menu::{
    is_classic_context_menu_applied, is_create_symlink_applied, is_menu_show_delay_disabled,
    set_classic_context_menu, set_create_symlink, set_menu_show_delay_disabled,
};
use crate::entities::tweaks::explorer::{
    is_hide_gallery_applied, is_hide_home_applied, is_hide_linux_applied, is_hide_network_applied,
    is_open_to_this_pc_applied, is_wsl_installed, set_hide_gallery, set_hide_home, set_hide_linux,
    set_hide_network, set_open_to_this_pc,
};
use crate::entities::tweaks::input::{
    is_disable_mouse_acceleration_applied, set_disable_mouse_acceleration,
};
use crate::entities::tweaks::interface_tweak::{
    is_disable_jpeg_compression_applied, is_remove_shortcut_arrows_applied,
    is_remove_shortcut_suffix_applied, set_disable_jpeg_compression, set_remove_shortcut_arrows,
    set_remove_shortcut_suffix,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum TweakCategory {
    ContextMenu,
    Explorer,
    Interface,
    Input,
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
        is_applied: is_create_symlink_applied,
        set_applied: set_create_symlink,
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
        is_applied: is_disable_mouse_acceleration_applied,
        set_applied: set_disable_mouse_acceleration,
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
