const REG_MOUSE: &str = r"Control Panel\Mouse";
const VAL_RAW_MOUSE_THROTTLE: &str = "RawMouseThrottleDuration";
const THROTTLE_50HZ_MS: u32 = 20;

#[must_use]
pub fn is_raw_mouse_throttle_applied() -> bool {
    #[cfg(target_os = "windows")]
    {
        windows_registry::CURRENT_USER
            .open(REG_MOUSE)
            .ok()
            .and_then(|key| key.get_u32(VAL_RAW_MOUSE_THROTTLE).ok())
            == Some(THROTTLE_50HZ_MS)
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_raw_mouse_throttle(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let key = windows_registry::CURRENT_USER
            .create(REG_MOUSE)
            .map_err(|e| format!("Failed to open {REG_MOUSE}: {e}"))?;

        if applied {
            key.set_u32(VAL_RAW_MOUSE_THROTTLE, THROTTLE_50HZ_MS)
                .map_err(|e| format!("Failed to set {VAL_RAW_MOUSE_THROTTLE}: {e}"))?;
        } else {
            let _ = key.remove_value(VAL_RAW_MOUSE_THROTTLE);
        }
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = applied;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mouse_throttle_check_runs_without_panic() {
        let _ = is_raw_mouse_throttle_applied();
    }

    #[test]
    fn mouse_throttle_toggle_roundtrip() {
        let original = is_raw_mouse_throttle_applied();
        assert!(set_raw_mouse_throttle(!original).is_ok());
        assert_eq!(is_raw_mouse_throttle_applied(), !original);
        assert!(set_raw_mouse_throttle(original).is_ok());
        assert_eq!(is_raw_mouse_throttle_applied(), original);
    }
}
