mod classic_context_menu;
mod disable_shortcut_arrows;
mod disable_transparency;
mod hide_explorer_gallery;
mod hide_explorer_home;

use crate::tweaks::Tweak;

pub fn get_tweaks() -> Vec<Box<dyn Tweak>> {
  vec![
    Box::new(classic_context_menu::ClassicContextMenuTweak::new()),
    Box::new(disable_shortcut_arrows::DisableShortcutArrowsTweak::new()),
    Box::new(disable_transparency::DisableTransparencyTweak::new()),
    Box::new(hide_explorer_home::HideExplorerHomeTweak::new()),
    Box::new(hide_explorer_gallery::HideExplorerGalleryTweak::new()),
  ]
}
