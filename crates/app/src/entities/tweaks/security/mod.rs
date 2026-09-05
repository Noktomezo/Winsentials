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
const TIMEQ_FOREVER: u32 = u32::MAX;

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

#[must_use]
#[allow(unsafe_code, clippy::cast_ptr_alignment)]
pub fn is_password_expiration_disabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::NetworkManagement::NetManagement::{
            NetApiBufferFree, NetUserModalsGet, USER_MODALS_INFO_0,
        };

        unsafe {
            let mut buf_ptr: *mut u8 = std::ptr::null_mut();
            let status = NetUserModalsGet(std::ptr::null(), 0, &raw mut buf_ptr);
            if status == 0 && !buf_ptr.is_null() {
                let modals = &*(buf_ptr.cast::<USER_MODALS_INFO_0>());
                let max_age = modals.usrmod0_max_passwd_age;
                NetApiBufferFree(buf_ptr.cast());
                max_age == TIMEQ_FOREVER
            } else {
                false
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

#[allow(
    unsafe_code,
    clippy::cast_possible_truncation,
    clippy::cast_ptr_alignment
)]
pub fn set_password_expiration_disabled(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::NetworkManagement::NetManagement::{
            FILTER_NORMAL_ACCOUNT, NetApiBufferFree, NetUserEnum, NetUserGetInfo, NetUserModalsGet,
            NetUserModalsSet, NetUserSetInfo, UF_DONT_EXPIRE_PASSWD, USER_INFO_0, USER_INFO_1,
            USER_INFO_1008, USER_MODALS_INFO_0,
        };

        unsafe {
            let mut buf_ptr: *mut u8 = std::ptr::null_mut();
            let status = NetUserModalsGet(std::ptr::null(), 0, &raw mut buf_ptr);
            if status != 0 || buf_ptr.is_null() {
                return Err(format!("NetUserModalsGet failed with code {status}"));
            }

            let mut modals = *(buf_ptr.cast::<USER_MODALS_INFO_0>());
            NetApiBufferFree(buf_ptr.cast());

            // 42 days (default in Windows) or TIMEQ_FOREVER (unlimited)
            modals.usrmod0_max_passwd_age = if applied {
                TIMEQ_FOREVER
            } else {
                42 * 24 * 60 * 60
            };

            let mut parm_err = 0u32;
            let set_status = NetUserModalsSet(
                std::ptr::null(),
                0,
                (&raw mut modals).cast(),
                &raw mut parm_err,
            );
            if set_status != 0 {
                return Err(format!(
                    "NetUserModalsSet failed with code {set_status} (parm_err: {parm_err})"
                ));
            }

            // Also update UF_DONT_EXPIRE_PASSWD flag on all local user accounts
            let mut resume_handle = 0u32;
            let mut users_ptr: *mut u8 = std::ptr::null_mut();
            let mut entries_read = 0u32;
            let mut total_entries = 0u32;

            let enum_status = NetUserEnum(
                std::ptr::null(),
                0,
                FILTER_NORMAL_ACCOUNT,
                &raw mut users_ptr,
                u32::MAX,
                &raw mut entries_read,
                &raw mut total_entries,
                &raw mut resume_handle,
            );

            if (enum_status == 0 || enum_status == 234) && !users_ptr.is_null() {
                let user_slice = std::slice::from_raw_parts(
                    users_ptr.cast::<USER_INFO_0>(),
                    entries_read as usize,
                );

                for user in user_slice {
                    let mut info1_ptr: *mut u8 = std::ptr::null_mut();
                    if NetUserGetInfo(std::ptr::null(), user.usri0_name, 1, &raw mut info1_ptr) == 0
                        && !info1_ptr.is_null()
                    {
                        let info1 = &*(info1_ptr.cast::<USER_INFO_1>());
                        let current_flags = info1.usri1_flags;
                        NetApiBufferFree(info1_ptr.cast());

                        let new_flags = if applied {
                            current_flags | UF_DONT_EXPIRE_PASSWD
                        } else {
                            current_flags & !UF_DONT_EXPIRE_PASSWD
                        };

                        if new_flags != current_flags {
                            let mut info1008 = USER_INFO_1008 {
                                usri1008_flags: new_flags,
                            };
                            let mut set_user_err = 0u32;
                            let _ = NetUserSetInfo(
                                std::ptr::null(),
                                user.usri0_name,
                                1008,
                                (&raw mut info1008).cast(),
                                &raw mut set_user_err,
                            );
                        }
                    }
                }

                NetApiBufferFree(users_ptr.cast());
            }

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
