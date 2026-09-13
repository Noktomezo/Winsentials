#[cfg(target_os = "windows")]
use super::service_helper::{remove_reg_value, set_reg_u32};

const REG_APP_PRIVACY_POLICIES: &str = r"SOFTWARE\Policies\Microsoft\Windows\AppPrivacy";

const VALUE_FORCE_DENY: u32 = 2;

#[cfg(target_os = "windows")]
const PERMISSION_VALUES: [&str; 8] = [
    "LetAppsAccessContacts",
    "LetAppsAccessCalendar",
    "LetAppsAccessEmail",
    "LetAppsAccessTasks",
    "LetAppsAccessMessaging",
    "LetAppsAccessPhone",
    "LetAppsAccessCallHistory",
    "LetAppsAccessAccountInfo",
];

#[cfg(target_os = "windows")]
const FS_VALUES: [&str; 5] = [
    "LetAppsAccessDocumentsLibrary",
    "LetAppsAccessPicturesLibrary",
    "LetAppsAccessVideosLibrary",
    "LetAppsAccessBroadFileSystemAccess",
    "LetAppsAccessDiagnosticInfo",
];

// ----------------------------------------------------------------------------
// 16. App Permissions: Camera & Microphone
// ----------------------------------------------------------------------------

#[must_use]
pub fn is_app_camera_mic_access_disabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        windows_registry::LOCAL_MACHINE
            .open(REG_APP_PRIVACY_POLICIES)
            .ok()
            .and_then(|k| k.get_u32("LetAppsAccessCamera").ok())
            == Some(VALUE_FORCE_DENY)
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_app_camera_mic_access_disabled(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let key = windows_registry::LOCAL_MACHINE
            .create(REG_APP_PRIVACY_POLICIES)
            .map_err(|e| format!("Failed to open AppPrivacy policy: {e}"))?;

        if applied {
            set_reg_u32(
                &key,
                REG_APP_PRIVACY_POLICIES,
                "LetAppsAccessCamera",
                VALUE_FORCE_DENY,
            )?;
            set_reg_u32(
                &key,
                REG_APP_PRIVACY_POLICIES,
                "LetAppsAccessMicrophone",
                VALUE_FORCE_DENY,
            )?;
            set_reg_u32(
                &key,
                REG_APP_PRIVACY_POLICIES,
                "LetAppsAccessWebcam",
                VALUE_FORCE_DENY,
            )?;
        } else {
            remove_reg_value(&key, REG_APP_PRIVACY_POLICIES, "LetAppsAccessCamera")?;
            remove_reg_value(&key, REG_APP_PRIVACY_POLICIES, "LetAppsAccessMicrophone")?;
            remove_reg_value(&key, REG_APP_PRIVACY_POLICIES, "LetAppsAccessWebcam")?;
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
// 17. App Permissions: Personal Data (Contacts, Calendar, Email, Tasks, Phone)
// ----------------------------------------------------------------------------

#[must_use]
pub fn is_app_personal_data_access_disabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        windows_registry::LOCAL_MACHINE
            .open(REG_APP_PRIVACY_POLICIES)
            .ok()
            .and_then(|k| k.get_u32("LetAppsAccessContacts").ok())
            == Some(VALUE_FORCE_DENY)
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_app_personal_data_access_disabled(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let key = windows_registry::LOCAL_MACHINE
            .create(REG_APP_PRIVACY_POLICIES)
            .map_err(|e| format!("Failed to open AppPrivacy policy: {e}"))?;

        if applied {
            for val in PERMISSION_VALUES {
                set_reg_u32(&key, REG_APP_PRIVACY_POLICIES, val, VALUE_FORCE_DENY)?;
            }
        } else {
            for val in PERMISSION_VALUES {
                remove_reg_value(&key, REG_APP_PRIVACY_POLICIES, val)?;
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

// ----------------------------------------------------------------------------
// 18. App Permissions: File System & Library Access
// ----------------------------------------------------------------------------

#[must_use]
pub fn is_app_file_system_access_disabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        windows_registry::LOCAL_MACHINE
            .open(REG_APP_PRIVACY_POLICIES)
            .ok()
            .and_then(|k| k.get_u32("LetAppsAccessDocumentsLibrary").ok())
            == Some(VALUE_FORCE_DENY)
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_app_file_system_access_disabled(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let key = windows_registry::LOCAL_MACHINE
            .create(REG_APP_PRIVACY_POLICIES)
            .map_err(|e| format!("Failed to open AppPrivacy policy: {e}"))?;

        if applied {
            for val in FS_VALUES {
                set_reg_u32(&key, REG_APP_PRIVACY_POLICIES, val, VALUE_FORCE_DENY)?;
            }
        } else {
            for val in FS_VALUES {
                remove_reg_value(&key, REG_APP_PRIVACY_POLICIES, val)?;
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
