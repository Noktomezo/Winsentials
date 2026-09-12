pub mod hook;
pub mod state;
pub mod types;

pub use hook::{
    current_snapkey_preset, get_state, send_key_win32, set_snapkey_preset, shutdown_snapkey,
};
pub use state::SnapKeyState;
pub use types::{GroupState, KeyState, SnapKeyPreset, snapkey_preset_icon, snapkey_preset_label};

#[cfg(test)]
mod tests;
