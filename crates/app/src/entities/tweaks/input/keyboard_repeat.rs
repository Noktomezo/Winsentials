#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyboardRepeatPreset {
    Standard,
    Balanced,
    Fast,
    Ultra,
}

impl KeyboardRepeatPreset {
    pub const ALL: [Self; 4] = [Self::Standard, Self::Balanced, Self::Fast, Self::Ultra];

    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Balanced => "balanced",
            Self::Fast => "fast",
            Self::Ultra => "ultra",
        }
    }

    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "standard" => Some(Self::Standard),
            "balanced" => Some(Self::Balanced),
            "fast" => Some(Self::Fast),
            "ultra" => Some(Self::Ultra),
            _ => None,
        }
    }

    #[must_use]
    pub const fn values(self) -> (u32, u32) {
        match self {
            Self::Standard => (2, 20),
            Self::Balanced => (1, 24),
            Self::Fast => (0, 27),
            Self::Ultra => (0, 31),
        }
    }
}

#[must_use]
pub fn current_keyboard_repeat_preset() -> KeyboardRepeatPreset {
    keyboard_repeat_values().map_or(KeyboardRepeatPreset::Standard, |(delay, speed)| {
        KeyboardRepeatPreset::ALL
            .into_iter()
            .min_by_key(|preset| {
                let (preset_delay, preset_speed) = preset.values();
                delay.abs_diff(preset_delay) * 8 + speed.abs_diff(preset_speed)
            })
            .unwrap_or(KeyboardRepeatPreset::Standard)
    })
}

#[allow(unsafe_code)]
fn keyboard_repeat_values() -> Option<(u32, u32)> {
    #[cfg(target_os = "windows")]
    unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            SPI_GETKEYBOARDDELAY, SPI_GETKEYBOARDSPEED, SystemParametersInfoW,
        };

        let mut delay = 0;
        let mut speed = 0;
        let got_delay = SystemParametersInfoW(SPI_GETKEYBOARDDELAY, 0, (&raw mut delay).cast(), 0);
        let got_speed = SystemParametersInfoW(SPI_GETKEYBOARDSPEED, 0, (&raw mut speed).cast(), 0);
        (got_delay != 0 && got_speed != 0).then_some((delay, speed))
    }
    #[cfg(not(target_os = "windows"))]
    {
        None
    }
}

#[allow(unsafe_code)]
pub fn set_keyboard_repeat_preset(preset: KeyboardRepeatPreset) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    unsafe {
        use std::ptr::null_mut;
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            SPI_SETKEYBOARDDELAY, SPI_SETKEYBOARDSPEED, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE,
            SystemParametersInfoW,
        };

        let (delay, speed) = preset.values();
        let flags = SPIF_UPDATEINIFILE | SPIF_SENDCHANGE;
        let set_delay = SystemParametersInfoW(SPI_SETKEYBOARDDELAY, delay, null_mut(), flags);
        let set_speed = SystemParametersInfoW(SPI_SETKEYBOARDSPEED, speed, null_mut(), flags);
        if set_delay == 0 || set_speed == 0 {
            return Err(format!(
                "Failed to update keyboard repeat settings: {}",
                std::io::Error::last_os_error()
            ));
        }
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = preset;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_get_progressively_faster() {
        assert_eq!(
            KeyboardRepeatPreset::ALL.map(KeyboardRepeatPreset::values),
            [(2, 20), (1, 24), (0, 27), (0, 31),]
        );
        assert!(
            KeyboardRepeatPreset::ALL
                .windows(2)
                .all(|pair| pair[0].values().0 >= pair[1].values().0
                    && pair[0].values().1 < pair[1].values().1)
        );
    }
}
