use super::service_helper::{is_service_disabled, set_service_disabled};
use crate::shared::shell_notify::notify_shell_change;

const REG_SEARCH_CU: &str = r"Software\Microsoft\Windows\CurrentVersion\Search";
const REG_SEARCH_POLICIES: &str = r"SOFTWARE\Policies\Microsoft\Windows\Windows Search";

const REG_SETTING_SYNC_POLICIES: &str = r"SOFTWARE\Policies\Microsoft\Windows\SettingSync";

const REG_LOCATION_POLICIES: &str = r"SOFTWARE\Policies\Microsoft\Windows\LocationAndSensors";

const REG_ENV_LM: &str = r"SYSTEM\CurrentControlSet\Control\Session Manager\Environment";

const REG_SIUF_CU: &str = r"Software\Microsoft\Siuf\Rules";
const REG_DATA_COLLECTION_POLICIES: &str = r"SOFTWARE\Policies\Microsoft\Windows\DataCollection";

const REG_INTL_PROFILE_CU: &str = r"Control Panel\International\User Profile";

const REG_WIFI_POLICY_HOTSPOT: &str =
    r"SOFTWARE\Microsoft\PolicyManager\default\WiFi\AllowWiFiHotSpotReporting";
const REG_WIFI_POLICY_SENSE: &str =
    r"SOFTWARE\Microsoft\PolicyManager\default\WiFi\AllowAutoConnectToWiFiSenseHotspots";
const REG_WIFI_WCM: &str = r"SOFTWARE\Microsoft\WcmSvc\wifinetworkmanager\config";

// ----------------------------------------------------------------------------
// 9. Web Search & Bing in Start Menu
// ----------------------------------------------------------------------------

#[must_use]
pub fn is_search_bing_integration_disabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        windows_registry::LOCAL_MACHINE
            .open(REG_SEARCH_POLICIES)
            .ok()
            .and_then(|k| k.get_u32("DisableWebSearch").ok())
            == Some(1)
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_search_bing_integration_disabled(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let cu_search = windows_registry::CURRENT_USER
            .create(REG_SEARCH_CU)
            .map_err(|e| format!("Failed to open Search key: {e}"))?;
        let pol_search = windows_registry::LOCAL_MACHINE
            .create(REG_SEARCH_POLICIES)
            .map_err(|e| format!("Failed to open Search policy: {e}"))?;

        if applied {
            let _ = cu_search.set_u32("BingSearchEnabled", 0);
            let _ = cu_search.set_u32("CortanaConsent", 0);
            let _ = cu_search.set_u32("AllowSearchToUseLocation", 0);
            let _ = pol_search.set_u32("DisableWebSearch", 1);
            let _ = pol_search.set_u32("ConnectedSearchUseWeb", 0);
        } else {
            let _ = cu_search.remove_value("BingSearchEnabled");
            let _ = cu_search.remove_value("CortanaConsent");
            let _ = cu_search.remove_value("AllowSearchToUseLocation");
            let _ = pol_search.remove_value("DisableWebSearch");
            let _ = pol_search.remove_value("ConnectedSearchUseWeb");
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
// 10. Windows Settings Cloud Sync
// ----------------------------------------------------------------------------

#[must_use]
pub fn is_cloud_sync_disabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        windows_registry::LOCAL_MACHINE
            .open(REG_SETTING_SYNC_POLICIES)
            .ok()
            .and_then(|k| k.get_u32("DisableSettingSync").ok())
            == Some(2)
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_cloud_sync_disabled(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let key = windows_registry::LOCAL_MACHINE
            .create(REG_SETTING_SYNC_POLICIES)
            .map_err(|e| format!("Failed to open SettingSync policy: {e}"))?;

        if applied {
            let _ = key.set_u32("DisableSettingSync", 2);
            let _ = key.set_u32("DisableSettingSyncUserOverride", 1);
            let _ = key.set_u32("DisableSyncOnPaidNetwork", 1);
            let _ = key.set_u32("DisableApplicationSettingSync", 2);
            let _ = key.set_u32("DisableCredentialsSettingSync", 2);
            let _ = key.set_u32("DisablePersonalizationSettingSync", 2);
            let _ = key.set_u32("DisableWebBrowserSettingSync", 2);
            let _ = key.set_u32("DisableWindowsSettingSync", 2);
        } else {
            let _ = key.remove_value("DisableSettingSync");
            let _ = key.remove_value("DisableSettingSyncUserOverride");
            let _ = key.remove_value("DisableSyncOnPaidNetwork");
            let _ = key.remove_value("DisableApplicationSettingSync");
            let _ = key.remove_value("DisableCredentialsSettingSync");
            let _ = key.remove_value("DisablePersonalizationSettingSync");
            let _ = key.remove_value("DisableWebBrowserSettingSync");
            let _ = key.remove_value("DisableWindowsSettingSync");
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
// 11. Location & Sensors Tracking
// ----------------------------------------------------------------------------

#[must_use]
pub fn is_location_and_sensors_disabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        let pol_off = windows_registry::LOCAL_MACHINE
            .open(REG_LOCATION_POLICIES)
            .ok()
            .and_then(|k| k.get_u32("DisableLocation").ok())
            == Some(1);
        let svc_off = is_service_disabled("lfsvc");
        pol_off || svc_off
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_location_and_sensors_disabled(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        set_service_disabled("lfsvc", applied, 3)?;

        let key = windows_registry::LOCAL_MACHINE
            .create(REG_LOCATION_POLICIES)
            .map_err(|e| format!("Failed to open LocationAndSensors policy: {e}"))?;

        if applied {
            let _ = key.set_u32("DisableLocation", 1);
            let _ = key.set_u32("DisableLocationScripting", 1);
            let _ = key.set_u32("DisableSensors", 1);
            let _ = key.set_u32("DisableWindowsLocationProvider", 1);
        } else {
            let _ = key.remove_value("DisableLocation");
            let _ = key.remove_value("DisableLocationScripting");
            let _ = key.remove_value("DisableSensors");
            let _ = key.remove_value("DisableWindowsLocationProvider");
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
// 12. Developer Telemetry (.NET CLI & PowerShell 7)
// ----------------------------------------------------------------------------

#[must_use]
pub fn is_developer_telemetry_disabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        windows_registry::LOCAL_MACHINE
            .open(REG_ENV_LM)
            .ok()
            .and_then(|k| k.get_string("DOTNET_CLI_TELEMETRY_OPTOUT").ok())
            .as_deref()
            == Some("1")
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_developer_telemetry_disabled(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let key = windows_registry::LOCAL_MACHINE
            .create(REG_ENV_LM)
            .map_err(|e| format!("Failed to open system Environment: {e}"))?;

        if applied {
            let _ = key.set_string("DOTNET_CLI_TELEMETRY_OPTOUT", "1");
            let _ = key.set_string("POWERSHELL_TELEMETRY_OPTOUT", "1");
        } else {
            let _ = key.remove_value("DOTNET_CLI_TELEMETRY_OPTOUT");
            let _ = key.remove_value("POWERSHELL_TELEMETRY_OPTOUT");
        }
        notify_shell_change();
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = applied;
        Ok(())
    }
}

// ----------------------------------------------------------------------------
// 13. Windows Feedback & Diagnostic Prompts
// ----------------------------------------------------------------------------

#[must_use]
pub fn is_feedback_notifications_disabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        windows_registry::CURRENT_USER
            .open(REG_SIUF_CU)
            .ok()
            .and_then(|k| k.get_u32("NumberOfSIUFInPeriod").ok())
            == Some(0)
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_feedback_notifications_disabled(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let siuf_key = windows_registry::CURRENT_USER
            .create(REG_SIUF_CU)
            .map_err(|e| format!("Failed to open SIUF Rules: {e}"))?;
        let pol_key = windows_registry::LOCAL_MACHINE
            .create(REG_DATA_COLLECTION_POLICIES)
            .map_err(|e| format!("Failed to open DataCollection policy: {e}"))?;

        if applied {
            let _ = siuf_key.set_u32("NumberOfSIUFInPeriod", 0);
            let _ = siuf_key.remove_value("PeriodInNanoSeconds");
            let _ = pol_key.set_u32("DoNotShowFeedbackNotifications", 1);
        } else {
            let _ = siuf_key.remove_value("NumberOfSIUFInPeriod");
            let _ = pol_key.remove_value("DoNotShowFeedbackNotifications");
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
// 14. Websites Access to Language List
// ----------------------------------------------------------------------------

#[must_use]
pub fn is_website_language_access_disabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        windows_registry::CURRENT_USER
            .open(REG_INTL_PROFILE_CU)
            .ok()
            .and_then(|k| k.get_u32("HttpAcceptLanguageOptOut").ok())
            == Some(1)
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_website_language_access_disabled(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let key = windows_registry::CURRENT_USER
            .create(REG_INTL_PROFILE_CU)
            .map_err(|e| format!("Failed to open International User Profile: {e}"))?;

        if applied {
            key.set_u32("HttpAcceptLanguageOptOut", 1)
                .map_err(|e| format!("Failed to set language opt-out: {e}"))
        } else {
            let _ = key.remove_value("HttpAcceptLanguageOptOut");
            Ok(())
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = applied;
        Ok(())
    }
}

// ----------------------------------------------------------------------------
// 15. Wi-Fi Sense
// ----------------------------------------------------------------------------

#[must_use]
pub fn is_wifi_sense_disabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        windows_registry::LOCAL_MACHINE
            .open(REG_WIFI_WCM)
            .ok()
            .and_then(|k| k.get_u32("AutoConnectAllowedOEM").ok())
            == Some(0)
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_wifi_sense_disabled(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let pol_hotspot = windows_registry::LOCAL_MACHINE.create(REG_WIFI_POLICY_HOTSPOT);
        let pol_sense = windows_registry::LOCAL_MACHINE.create(REG_WIFI_POLICY_SENSE);
        let wcm_key = windows_registry::LOCAL_MACHINE.create(REG_WIFI_WCM);

        if applied {
            if let Ok(k) = pol_hotspot {
                let _ = k.set_u32("value", 0);
            }
            if let Ok(k) = pol_sense {
                let _ = k.set_u32("value", 0);
            }
            if let Ok(k) = wcm_key {
                let _ = k.set_u32("AutoConnectAllowedOEM", 0);
            }
        } else {
            if let Ok(k) = pol_hotspot {
                let _ = k.remove_value("value");
            }
            if let Ok(k) = pol_sense {
                let _ = k.remove_value("value");
            }
            if let Ok(k) = wcm_key {
                let _ = k.remove_value("AutoConnectAllowedOEM");
            }
        }
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = applied;
        Ok(())
    }
}
