const REG_UAC: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System";
const REG_ATTACHMENTS: &str = r"Software\Microsoft\Windows\CurrentVersion\Policies\Attachments";
const REG_EXPLORER_POLICIES: &str = r"Software\Microsoft\Windows\CurrentVersion\Policies\Explorer";
const REG_EXPLORER: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer";
const REG_EXPLORER_SERIALIZE: &str =
    r"Software\Microsoft\Windows\CurrentVersion\Explorer\Serialize";
const REG_SECURITY_NOTIFICATIONS: &str =
    r"SOFTWARE\Policies\Microsoft\Windows Defender Security Center\Notifications";
const REG_SECURITY_SYSTRAY: &str =
    r"SOFTWARE\Policies\Microsoft\Windows Defender Security Center\Systray";

const UAC_DISABLED: u32 = 0;
const DO_NOT_PRESERVE_ZONE: u32 = 1;
const AUTOPLAY_REMOVABLE_DISABLED: u32 = 0xB5;
const POLICY_ENABLED: u32 = 1;

#[must_use]
pub fn is_uac_disabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        windows_registry::LOCAL_MACHINE
            .open(REG_UAC)
            .ok()
            .and_then(|key| key.get_u32("EnableLUA").ok())
            == Some(UAC_DISABLED)
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_uac_disabled(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let key = windows_registry::LOCAL_MACHINE
            .create(REG_UAC)
            .map_err(|error| format!("Failed to open UAC policy: {error}"))?;
        key.set_u32("EnableLUA", u32::from(!applied))
            .map_err(|error| format!("Failed to update UAC policy: {error}"))
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = applied;
        Ok(())
    }
}

#[must_use]
pub fn is_download_warning_disabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        windows_registry::CURRENT_USER
            .open(REG_ATTACHMENTS)
            .ok()
            .and_then(|key| key.get_u32("SaveZoneInformation").ok())
            == Some(DO_NOT_PRESERVE_ZONE)
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_download_warning_disabled(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let key = windows_registry::CURRENT_USER
            .create(REG_ATTACHMENTS)
            .map_err(|error| format!("Failed to open Attachment Manager policy: {error}"))?;
        if applied {
            key.set_u32("SaveZoneInformation", DO_NOT_PRESERVE_ZONE)
                .map_err(|error| format!("Failed to update Attachment Manager policy: {error}"))
        } else {
            let _ = key.remove_value("SaveZoneInformation");
            Ok(())
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = applied;
        Ok(())
    }
}

#[must_use]
pub fn is_removable_autoplay_disabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        windows_registry::CURRENT_USER
            .open(REG_EXPLORER_POLICIES)
            .ok()
            .and_then(|key| key.get_u32("NoDriveTypeAutoRun").ok())
            == Some(AUTOPLAY_REMOVABLE_DISABLED)
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_removable_autoplay_disabled(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let key = windows_registry::CURRENT_USER
            .create(REG_EXPLORER_POLICIES)
            .map_err(|error| format!("Failed to open AutoPlay policy: {error}"))?;
        if applied {
            key.set_u32("NoDriveTypeAutoRun", AUTOPLAY_REMOVABLE_DISABLED)
                .map_err(|error| format!("Failed to update AutoPlay policy: {error}"))
        } else {
            let _ = key.remove_value("NoDriveTypeAutoRun");
            Ok(())
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = applied;
        Ok(())
    }
}

#[must_use]
pub fn is_quick_access_history_disabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        windows_registry::CURRENT_USER
            .open(REG_EXPLORER)
            .is_ok_and(|key| {
                key.get_u32("ShowRecent").ok() == Some(0)
                    && key.get_u32("ShowFrequent").ok() == Some(0)
            })
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_quick_access_history_disabled(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let key = windows_registry::CURRENT_USER
            .create(REG_EXPLORER)
            .map_err(|error| format!("Failed to open File Explorer settings: {error}"))?;
        if applied {
            key.set_u32("ShowRecent", 0)
                .and_then(|()| key.set_u32("ShowFrequent", 0))
                .map_err(|error| format!("Failed to update Quick Access history: {error}"))
        } else {
            let _ = key.remove_value("ShowRecent");
            let _ = key.remove_value("ShowFrequent");
            Ok(())
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = applied;
        Ok(())
    }
}

#[must_use]
pub fn are_security_center_notifications_disabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        windows_registry::LOCAL_MACHINE
            .open(REG_SECURITY_NOTIFICATIONS)
            .ok()
            .and_then(|key| key.get_u32("DisableNotifications").ok())
            == Some(POLICY_ENABLED)
            && windows_registry::LOCAL_MACHINE
                .open(REG_SECURITY_SYSTRAY)
                .ok()
                .and_then(|key| key.get_u32("HideSystray").ok())
                == Some(POLICY_ENABLED)
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_security_center_notifications_disabled(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let notifications = windows_registry::LOCAL_MACHINE
            .create(REG_SECURITY_NOTIFICATIONS)
            .map_err(|error| {
                format!("Failed to open Windows Security notification policy: {error}")
            })?;
        let systray = windows_registry::LOCAL_MACHINE
            .create(REG_SECURITY_SYSTRAY)
            .map_err(|error| format!("Failed to open Windows Security systray policy: {error}"))?;

        if applied {
            notifications
                .set_u32("DisableNotifications", POLICY_ENABLED)
                .and_then(|()| systray.set_u32("HideSystray", POLICY_ENABLED))
                .map_err(|error| format!("Failed to update Windows Security policy: {error}"))
        } else {
            let _ = notifications.remove_value("DisableNotifications");
            let _ = systray.remove_value("HideSystray");
            Ok(())
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = applied;
        Ok(())
    }
}

#[must_use]
pub fn is_startup_delay_disabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        windows_registry::CURRENT_USER
            .open(REG_EXPLORER_SERIALIZE)
            .ok()
            .and_then(|key| key.get_u32("StartupDelayInMSec").ok())
            == Some(0)
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_startup_delay_disabled(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let key = windows_registry::CURRENT_USER
            .create(REG_EXPLORER_SERIALIZE)
            .map_err(|error| format!("Failed to open startup settings: {error}"))?;
        if applied {
            key.set_u32("StartupDelayInMSec", 0)
                .map_err(|error| format!("Failed to update startup delay: {error}"))
        } else {
            let _ = key.remove_value("StartupDelayInMSec");
            Ok(())
        }
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
    fn policy_values_match_windows_definitions() {
        assert_eq!(UAC_DISABLED, 0);
        assert_eq!(DO_NOT_PRESERVE_ZONE, 1);
        assert_eq!(AUTOPLAY_REMOVABLE_DISABLED, 0xB5);
        assert_eq!(POLICY_ENABLED, 1);
    }
}
