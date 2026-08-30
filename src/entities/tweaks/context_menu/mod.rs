pub mod classic_menu;
pub mod create_symlink;
pub mod menu_delay;
pub mod open_with_editor;

#[allow(unused_imports)]
pub use classic_menu::{is_classic_context_menu_applied, set_classic_context_menu};
#[allow(unused_imports)]
pub use create_symlink::{is_create_symlink_applied, set_create_symlink};
#[allow(unused_imports)]
pub use menu_delay::{is_menu_show_delay_disabled, set_menu_show_delay_disabled};
#[allow(unused_imports)]
pub use open_with_editor::{detect_notepad, is_open_with_notepad_applied, set_open_with_notepad};
