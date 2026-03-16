pub mod classic_context_menu;
pub mod fast_taskbar_thumbnails;
pub mod faster_cursor_blink_rate;
pub mod hide_gallery_navigation_pane;
pub mod hide_home_navigation_pane;
pub mod hide_network_navigation_pane;
pub mod remove_shortcut_arrows;
pub mod remove_shortcut_suffix;

use crate::tweaks::Tweak;

pub fn tweaks() -> Vec<Box<dyn Tweak>> {
    vec![
        Box::new(classic_context_menu::ClassicContextMenuTweak::new()),
        Box::new(fast_taskbar_thumbnails::FastTaskbarThumbnailsTweak::new()),
        Box::new(faster_cursor_blink_rate::FasterCursorBlinkRateTweak::new()),
        Box::new(remove_shortcut_arrows::RemoveShortcutArrowsTweak::new()),
        Box::new(remove_shortcut_suffix::RemoveShortcutSuffixTweak::new()),
        Box::new(hide_home_navigation_pane::HideHomeNavigationPaneTweak::new()),
        Box::new(hide_gallery_navigation_pane::HideGalleryNavigationPaneTweak::new()),
        Box::new(hide_network_navigation_pane::HideNetworkNavigationPaneTweak::new()),
    ]
}
