pub mod classic_menu;
pub mod copy_image;
pub mod create_symlink;
pub mod menu_delay;
pub mod take_ownership;

#[allow(unused_imports)]
pub use classic_menu::{is_classic_context_menu_applied, set_classic_context_menu};
#[allow(unused_imports)]
pub use copy_image::{is_copy_image_applied, set_copy_image};
#[allow(unused_imports)]
pub use create_symlink::{is_create_symlink_applied, set_create_symlink};
#[allow(unused_imports)]
pub use menu_delay::{is_menu_show_delay_disabled, set_menu_show_delay_disabled};
#[allow(unused_imports)]
pub use take_ownership::{is_take_ownership_applied, set_take_ownership};
