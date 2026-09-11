const REG_GRAPHICS_DRIVERS: &str = r"SYSTEM\CurrentControlSet\Control\GraphicsDrivers";
const REG_DWM: &str = r"SOFTWARE\Microsoft\Windows\Dwm";

const TARGET_TDR_DELAY_SECS: u32 = 10;
const TARGET_TDR_DDI_DELAY_SECS: u32 = 10;
const MPO_OVERLAY_TEST_MODE_DISABLED: u32 = 5;

#[must_use]
pub fn is_gpu_tdr_delay_applied() -> bool {
    #[cfg(target_os = "windows")]
    {
        let Ok(key) = windows_registry::LOCAL_MACHINE.open(REG_GRAPHICS_DRIVERS) else {
            return false;
        };

        let tdr_delay = key.get_u32("TdrDelay").ok();
        let tdr_ddi_delay = key.get_u32("TdrDdiDelay").ok();

        tdr_delay == Some(TARGET_TDR_DELAY_SECS) && tdr_ddi_delay == Some(TARGET_TDR_DDI_DELAY_SECS)
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_gpu_tdr_delay(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let key = windows_registry::LOCAL_MACHINE
            .create(REG_GRAPHICS_DRIVERS)
            .map_err(|error| format!("Failed to open GraphicsDrivers registry key: {error}"))?;

        if applied {
            key.set_u32("TdrDelay", TARGET_TDR_DELAY_SECS)
                .map_err(|error| format!("Failed to set TdrDelay: {error}"))?;
            key.set_u32("TdrDdiDelay", TARGET_TDR_DDI_DELAY_SECS)
                .map_err(|error| format!("Failed to set TdrDdiDelay: {error}"))?;
        } else {
            let _ = key.remove_value("TdrDelay");
            let _ = key.remove_value("TdrDdiDelay");
        }

        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = applied;
        Ok(())
    }
}

#[must_use]
pub fn is_disable_mpo_applied() -> bool {
    #[cfg(target_os = "windows")]
    {
        let Ok(key) = windows_registry::LOCAL_MACHINE.open(REG_DWM) else {
            return false;
        };

        key.get_u32("OverlayTestMode").ok() == Some(MPO_OVERLAY_TEST_MODE_DISABLED)
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_disable_mpo(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let key = windows_registry::LOCAL_MACHINE
            .create(REG_DWM)
            .map_err(|error| format!("Failed to open DWM registry key: {error}"))?;

        if applied {
            key.set_u32("OverlayTestMode", MPO_OVERLAY_TEST_MODE_DISABLED)
                .map_err(|error| format!("Failed to set OverlayTestMode: {error}"))?;
        } else {
            let _ = key.remove_value("OverlayTestMode");
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
    fn gpu_tdr_delay_check_runs_without_panic() {
        let _ = is_gpu_tdr_delay_applied();
    }

    #[test]
    fn disable_mpo_check_runs_without_panic() {
        let _ = is_disable_mpo_applied();
    }
}
