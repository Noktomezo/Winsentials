pub mod shortcut_arrows;
pub mod shortcut_suffix;
pub mod wallpaper_quality;

#[allow(unused_imports)]
pub use shortcut_arrows::{is_remove_shortcut_arrows_applied, set_remove_shortcut_arrows};
#[allow(unused_imports)]
pub use shortcut_suffix::{is_remove_shortcut_suffix_applied, set_remove_shortcut_suffix};
#[allow(unused_imports)]
pub use wallpaper_quality::{is_disable_jpeg_compression_applied, set_disable_jpeg_compression};
