const REG_NDU_KEY: &str = r"SYSTEM\CurrentControlSet\Services\Ndu";
const NDU_START_VAL: &str = "Start";
const NDU_DISABLED: u32 = 4;
const NDU_AUTO: u32 = 2;

#[must_use]
pub fn is_disable_ndu_applied() -> bool {
    #[cfg(target_os = "windows")]
    {
        windows_registry::LOCAL_MACHINE
            .open(REG_NDU_KEY)
            .ok()
            .and_then(|key| key.get_u32(NDU_START_VAL).ok())
            .is_some_and(|val| val == NDU_DISABLED)
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_disable_ndu(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let key = windows_registry::LOCAL_MACHINE
            .create(REG_NDU_KEY)
            .map_err(|error| format!("Failed to open Ndu service key: {error}"))?;
        let val = if applied { NDU_DISABLED } else { NDU_AUTO };
        key.set_u32(NDU_START_VAL, val)
            .map_err(|error| format!("Failed to set Ndu Start value: {error}"))?;
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
    fn disable_ndu_check_runs_without_panic() {
        let _ = is_disable_ndu_applied();
    }
}
