const REG_MOUSE: &str = r"Control Panel\Mouse";

#[must_use]
pub fn is_disable_mouse_acceleration_applied() -> bool {
    #[cfg(target_os = "windows")]
    {
        if let Ok(key) = windows_registry::CURRENT_USER.open(REG_MOUSE) {
            if let Ok(speed) = key.get_string("MouseSpeed") {
                return speed == "0";
            }
        }
        false
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

#[allow(unsafe_code)]
pub fn set_disable_mouse_acceleration(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let key = windows_registry::CURRENT_USER
            .create(REG_MOUSE)
            .map_err(|e| format!("Failed to open HKCU\\Control Panel\\Mouse: {e}"))?;

        let (speed, t1, t2) = if applied {
            ("0", "0", "0")
        } else {
            ("1", "6", "10")
        };

        key.set_string("MouseSpeed", speed)
            .map_err(|e| format!("Failed to set MouseSpeed: {e}"))?;
        key.set_string("MouseThreshold1", t1)
            .map_err(|e| format!("Failed to set MouseThreshold1: {e}"))?;
        key.set_string("MouseThreshold2", t2)
            .map_err(|e| format!("Failed to set MouseThreshold2: {e}"))?;

        // Immediately update running session via Win32 SystemParametersInfoW
        unsafe {
            use windows_sys::Win32::UI::WindowsAndMessaging::{
                SPI_SETMOUSE, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE, SystemParametersInfoW,
            };
            let mut params: [i32; 3] = if applied { [0, 0, 0] } else { [6, 10, 1] };
            SystemParametersInfoW(
                SPI_SETMOUSE,
                0,
                params.as_mut_ptr().cast(),
                SPIF_UPDATEINIFILE | SPIF_SENDCHANGE,
            );
        }

        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = applied;
        Ok(())
    }
}
