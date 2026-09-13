use super::service_helper::{is_service_disabled, set_service_disabled};

const REG_DATA_COLLECTION_POLICIES: &str = r"SOFTWARE\Policies\Microsoft\Windows\DataCollection";
const REG_DATA_COLLECTION_CURRENT: &str =
    r"SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\DataCollection";
const REG_PRIVACY: &str = r"Software\Microsoft\Windows\CurrentVersion\Privacy";

const REG_APP_COMPAT: &str = r"SOFTWARE\Policies\Microsoft\Windows\AppCompat";

const REG_SQM_POLICIES: &str = r"SOFTWARE\Policies\Microsoft\SQMClient\Windows";
const REG_SQM_LM: &str = r"SOFTWARE\Microsoft\SQMClient\Windows";
const REG_SQM_CU: &str = r"Software\Microsoft\SQMClient";
const REG_CEIP_POLICIES: &str =
    r"SOFTWARE\Policies\Microsoft\Windows\Customer Experience Improvement Program";

const REG_WER_POLICIES: &str = r"SOFTWARE\Policies\Microsoft\Windows\Windows Error Reporting";
const REG_WER_CU: &str = r"Software\Microsoft\Windows\Windows Error Reporting";

// ----------------------------------------------------------------------------
// 1. Telemetry and Diagnostics
// ----------------------------------------------------------------------------

#[must_use]
pub fn is_telemetry_and_diagnostics_disabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        let hklm_policy = windows_registry::LOCAL_MACHINE
            .open(REG_DATA_COLLECTION_POLICIES)
            .ok()
            .and_then(|key| key.get_u32("AllowTelemetry").ok())
            == Some(0);

        let diagtrack_off = is_service_disabled("DiagTrack");
        hklm_policy || diagtrack_off
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_telemetry_and_diagnostics_disabled(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        // 1. Service DiagTrack (Automatic by default) & dmwappushservice (Manual/Demand by default)
        set_service_disabled("DiagTrack", applied, 2)?;
        set_service_disabled("dmwappushservice", applied, 3)?;

        // 2. Registry Policies
        let pol_key = windows_registry::LOCAL_MACHINE
            .create(REG_DATA_COLLECTION_POLICIES)
            .map_err(|e| format!("Failed to open DataCollection policies: {e}"))?;
        let cur_key = windows_registry::LOCAL_MACHINE
            .create(REG_DATA_COLLECTION_CURRENT)
            .map_err(|e| format!("Failed to open DataCollection current version key: {e}"))?;
        let priv_key = windows_registry::CURRENT_USER
            .create(REG_PRIVACY)
            .map_err(|e| format!("Failed to open Privacy user key: {e}"))?;

        if applied {
            let _ = pol_key.set_u32("AllowTelemetry", 0);
            let _ = cur_key.set_u32("AllowTelemetry", 0);
            let _ = priv_key.set_u32("TailoredExperiencesWithDiagnosticDataEnabled", 0);
        } else {
            let _ = pol_key.remove_value("AllowTelemetry");
            let _ = cur_key.remove_value("AllowTelemetry");
            let _ = priv_key.remove_value("TailoredExperiencesWithDiagnosticDataEnabled");
        }
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = applied;
        Ok(())
    }
}

// ----------------------------------------------------------------------------
// 2. Application Compatibility & Inventory Telemetry
// ----------------------------------------------------------------------------

#[must_use]
pub fn is_app_compat_telemetry_disabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        windows_registry::LOCAL_MACHINE
            .open(REG_APP_COMPAT)
            .ok()
            .and_then(|k| k.get_u32("DisableInventory").ok())
            == Some(1)
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_app_compat_telemetry_disabled(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let key = windows_registry::LOCAL_MACHINE
            .create(REG_APP_COMPAT)
            .map_err(|e| format!("Failed to open AppCompat policy key: {e}"))?;

        if applied {
            let _ = key.set_u32("AITEnable", 0);
            let _ = key.set_u32("DisableInventory", 1);
            let _ = key.set_u32("DisablePCA", 1);
            let _ = key.set_u32("DisableUAR", 1);
        } else {
            let _ = key.remove_value("AITEnable");
            let _ = key.remove_value("DisableInventory");
            let _ = key.remove_value("DisablePCA");
            let _ = key.remove_value("DisableUAR");
        }
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = applied;
        Ok(())
    }
}

// ----------------------------------------------------------------------------
// 3. Customer Experience Improvement Program (CEIP / SQM)
// ----------------------------------------------------------------------------

#[must_use]
pub fn is_ceip_sqm_disabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        windows_registry::LOCAL_MACHINE
            .open(REG_SQM_POLICIES)
            .ok()
            .and_then(|k| k.get_u32("CEIPEnable").ok())
            == Some(0)
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_ceip_sqm_disabled(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let pol_key = windows_registry::LOCAL_MACHINE
            .create(REG_SQM_POLICIES)
            .map_err(|e| format!("Failed to open SQM policy: {e}"))?;
        let lm_key = windows_registry::LOCAL_MACHINE
            .create(REG_SQM_LM)
            .map_err(|e| format!("Failed to open SQM machine key: {e}"))?;
        let cu_key = windows_registry::CURRENT_USER
            .create(REG_SQM_CU)
            .map_err(|e| format!("Failed to open SQM user key: {e}"))?;
        let ceip_pol = windows_registry::LOCAL_MACHINE
            .create(REG_CEIP_POLICIES)
            .map_err(|e| format!("Failed to open CEIP policy: {e}"))?;

        if applied {
            let _ = pol_key.set_u32("CEIPEnable", 0);
            let _ = lm_key.set_u32("CEIPEnable", 0);
            let _ = cu_key.set_u32("CEIPEnable", 0);
            let _ = ceip_pol.set_u32("SubmitData", 0);
        } else {
            let _ = pol_key.remove_value("CEIPEnable");
            let _ = lm_key.remove_value("CEIPEnable");
            let _ = cu_key.remove_value("CEIPEnable");
            let _ = ceip_pol.remove_value("SubmitData");
        }
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = applied;
        Ok(())
    }
}

// ----------------------------------------------------------------------------
// 4. Windows Error Reporting (WER)
// ----------------------------------------------------------------------------

#[must_use]
pub fn is_windows_error_reporting_disabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        let policy_off = windows_registry::LOCAL_MACHINE
            .open(REG_WER_POLICIES)
            .ok()
            .and_then(|k| k.get_u32("Disabled").ok())
            == Some(1);
        let service_off = is_service_disabled("WerSvc");
        policy_off || service_off
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_windows_error_reporting_disabled(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        set_service_disabled("WerSvc", applied, 3)?;

        let pol_key = windows_registry::LOCAL_MACHINE
            .create(REG_WER_POLICIES)
            .map_err(|e| format!("Failed to open WER policy: {e}"))?;
        let cu_key = windows_registry::CURRENT_USER
            .create(REG_WER_CU)
            .map_err(|e| format!("Failed to open WER user key: {e}"))?;

        if applied {
            let _ = pol_key.set_u32("Disabled", 1);
            let _ = pol_key.set_u32("DoReport", 0);
            let _ = cu_key.set_u32("Disabled", 1);
        } else {
            let _ = pol_key.remove_value("Disabled");
            let _ = pol_key.remove_value("DoReport");
            let _ = cu_key.remove_value("Disabled");
        }
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = applied;
        Ok(())
    }
}
