pub mod csrss_priority;
pub mod ctf;
pub mod keyboard_repeat;
pub mod mouse_acceleration;
pub mod snapkey;

#[allow(unused_imports)]
pub use csrss_priority::{is_csrss_priority_applied, set_csrss_priority};
pub use ctf::{CtfOptimizationPreset, current_ctf_preset, set_ctf_preset};
pub use keyboard_repeat::{
    KeyboardRepeatPreset, current_keyboard_repeat_preset, set_keyboard_repeat_preset,
};
#[allow(unused_imports)]
pub use mouse_acceleration::{
    is_disable_mouse_acceleration_applied, set_disable_mouse_acceleration,
};
pub use snapkey::{
    SnapKeyPreset, current_snapkey_preset, set_snapkey_preset, shutdown_snapkey,
    snapkey_preset_icon, snapkey_preset_label,
};
