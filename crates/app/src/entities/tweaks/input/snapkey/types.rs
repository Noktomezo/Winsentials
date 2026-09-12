use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SnapKeyPreset {
    #[default]
    Off,
    Wasd,
    ArrowKeys,
    Esdf,
    Azerty,
}

impl SnapKeyPreset {
    pub const ALL: [Self; 5] = [
        Self::Off,
        Self::Wasd,
        Self::ArrowKeys,
        Self::Esdf,
        Self::Azerty,
    ];

    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Wasd => "wasd",
            Self::ArrowKeys => "arrow_keys",
            Self::Esdf => "esdf",
            Self::Azerty => "azerty",
        }
    }

    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "off" => Some(Self::Off),
            "wasd" => Some(Self::Wasd),
            "arrow_keys" => Some(Self::ArrowKeys),
            "esdf" => Some(Self::Esdf),
            "azerty" => Some(Self::Azerty),
            _ => None,
        }
    }
}

#[must_use]
pub fn snapkey_preset_label(preset: SnapKeyPreset) -> String {
    match preset {
        SnapKeyPreset::Off => rust_i18n::t!("tweaks.snapkey_off").to_string(),
        SnapKeyPreset::Wasd => rust_i18n::t!("tweaks.snapkey_wasd").to_string(),
        SnapKeyPreset::ArrowKeys => rust_i18n::t!("tweaks.snapkey_arrow_keys").to_string(),
        SnapKeyPreset::Esdf => rust_i18n::t!("tweaks.snapkey_esdf").to_string(),
        SnapKeyPreset::Azerty => rust_i18n::t!("tweaks.snapkey_azerty").to_string(),
    }
}

#[must_use]
pub const fn snapkey_preset_icon(preset: SnapKeyPreset) -> &'static str {
    match preset {
        SnapKeyPreset::Off => "icons/ban.svg",
        SnapKeyPreset::Wasd => "icons/gamepad-2.svg",
        SnapKeyPreset::ArrowKeys => "icons/gamepad-directional.svg",
        SnapKeyPreset::Esdf | SnapKeyPreset::Azerty => "icons/keyboard.svg",
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct KeyState {
    pub registered: bool,
    pub key_down: bool,
    pub group: u8,
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct GroupState {
    pub previous_key: u16,
    pub active_key: u16,
}
