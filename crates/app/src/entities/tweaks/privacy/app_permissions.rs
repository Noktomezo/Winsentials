const REG_APP_PRIVACY_POLICIES: &str = r"SOFTWARE\Policies\Microsoft\Windows\AppPrivacy";

const VALUE_FORCE_DENY: u32 = 2;

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
            let _ = key.set_u32("LetAppsAccessCamera", VALUE_FORCE_DENY);
            let _ = key.set_u32("LetAppsAccessMicrophone", VALUE_FORCE_DENY);
            let _ = key.set_u32("LetAppsAccessWebcam", VALUE_FORCE_DENY);
        } else {
            let _ = key.remove_value("LetAppsAccessCamera");
            let _ = key.remove_value("LetAppsAccessMicrophone");
            let _ = key.remove_value("LetAppsAccessWebcam");
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
            let _ = key.set_u32("LetAppsAccessContacts", VALUE_FORCE_DENY);
            let _ = key.set_u32("LetAppsAccessCalendar", VALUE_FORCE_DENY);
            let _ = key.set_u32("LetAppsAccessEmail", VALUE_FORCE_DENY);
            let _ = key.set_u32("LetAppsAccessTasks", VALUE_FORCE_DENY);
            let _ = key.set_u32("LetAppsAccessMessaging", VALUE_FORCE_DENY);
            let _ = key.set_u32("LetAppsAccessPhone", VALUE_FORCE_DENY);
            let _ = key.set_u32("LetAppsAccessCallHistory", VALUE_FORCE_DENY);
            let _ = key.set_u32("LetAppsAccessAccountInfo", VALUE_FORCE_DENY);
        } else {
            let _ = key.remove_value("LetAppsAccessContacts");
            let _ = key.remove_value("LetAppsAccessCalendar");
            let _ = key.remove_value("LetAppsAccessEmail");
            let _ = key.remove_value("LetAppsAccessTasks");
            let _ = key.remove_value("LetAppsAccessMessaging");
            let _ = key.remove_value("LetAppsAccessPhone");
            let _ = key.remove_value("LetAppsAccessCallHistory");
            let _ = key.remove_value("LetAppsAccessAccountInfo");
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
            let _ = key.set_u32("LetAppsAccessDocumentsLibrary", VALUE_FORCE_DENY);
            let _ = key.set_u32("LetAppsAccessPicturesLibrary", VALUE_FORCE_DENY);
            let _ = key.set_u32("LetAppsAccessVideosLibrary", VALUE_FORCE_DENY);
            let _ = key.set_u32("LetAppsAccessBroadFileSystemAccess", VALUE_FORCE_DENY);
            let _ = key.set_u32("LetAppsAccessDiagnosticInfo", VALUE_FORCE_DENY);
        } else {
            let _ = key.remove_value("LetAppsAccessDocumentsLibrary");
            let _ = key.remove_value("LetAppsAccessPicturesLibrary");
            let _ = key.remove_value("LetAppsAccessVideosLibrary");
            let _ = key.remove_value("LetAppsAccessBroadFileSystemAccess");
            let _ = key.remove_value("LetAppsAccessDiagnosticInfo");
        }
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = applied;
        Ok(())
    }
}
