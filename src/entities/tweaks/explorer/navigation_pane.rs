const CLSID_NETWORK: &str = "{F02C1A0D-BE21-4350-88B0-7367FC96EF3C}";
const CLSID_HOME: &str = "{f874310e-b6b7-47dc-bc84-b9e6b38f5903}";
const CLSID_GALLERY: &str = "{e88865ea-0e1c-4e20-9aa6-edcd0212c87c}";
const CLSID_LINUX: &str = "{B2B4A4D1-2754-4140-A2EB-9A76D9D7CDC6}";

#[must_use]
pub fn is_wsl_installed() -> bool {
    #[cfg(target_os = "windows")]
    {
        windows_registry::CURRENT_USER
            .open(r"Software\Microsoft\Windows\CurrentVersion\Lxss")
            .is_ok()
            || windows_registry::LOCAL_MACHINE
                .open(r"SOFTWARE\Microsoft\Windows\CurrentVersion\Lxss")
                .is_ok()
            || windows_registry::CURRENT_USER
                .open(format!(r"Software\Classes\CLSID\{CLSID_LINUX}"))
                .is_ok()
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

#[must_use]
pub fn is_hide_network_applied() -> bool {
    is_clsid_hidden(CLSID_NETWORK)
}

pub fn set_hide_network(applied: bool) -> Result<(), String> {
    set_clsid_hidden(CLSID_NETWORK, applied)
}

#[must_use]
pub fn is_hide_home_applied() -> bool {
    is_clsid_hidden(CLSID_HOME)
}

pub fn set_hide_home(applied: bool) -> Result<(), String> {
    set_clsid_hidden(CLSID_HOME, applied)
}

#[must_use]
pub fn is_hide_gallery_applied() -> bool {
    is_clsid_hidden(CLSID_GALLERY)
}

pub fn set_hide_gallery(applied: bool) -> Result<(), String> {
    set_clsid_hidden(CLSID_GALLERY, applied)
}

#[must_use]
pub fn is_hide_linux_applied() -> bool {
    is_clsid_hidden(CLSID_LINUX)
}

pub fn set_hide_linux(applied: bool) -> Result<(), String> {
    set_clsid_hidden(CLSID_LINUX, applied)
}

#[must_use]
pub fn is_open_to_this_pc_applied() -> bool {
    #[cfg(target_os = "windows")]
    {
        if let Ok(key) = windows_registry::CURRENT_USER
            .open(r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced")
        {
            key.get_u32("LaunchTo").unwrap_or(2) == 1
        } else {
            false
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_open_to_this_pc(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let key = windows_registry::CURRENT_USER
            .create(r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced")
            .map_err(|e| format!("Failed to open registry key: {e}"))?;
        let val: u32 = if applied { 1 } else { 2 };
        key.set_u32("LaunchTo", val)
            .map_err(|e| format!("Failed to set LaunchTo value: {e}"))?;
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = applied;
        Ok(())
    }
}

fn is_clsid_hidden(clsid: &str) -> bool {
    #[cfg(target_os = "windows")]
    {
        let key_path = format!(r"Software\Classes\CLSID\{clsid}");
        if let Ok(key) = windows_registry::CURRENT_USER.open(key_path) {
            key.get_u32("System.IsPinnedToNameSpaceTree").unwrap_or(1) == 0
        } else {
            false
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = clsid;
        false
    }
}

fn set_clsid_hidden(clsid: &str, hidden: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let key_path = format!(r"Software\Classes\CLSID\{clsid}");
        if hidden {
            let key = windows_registry::CURRENT_USER
                .create(&key_path)
                .map_err(|e| format!("Failed to create registry key: {e}"))?;
            if clsid == CLSID_LINUX {
                let _ = key.set_string("", "Linux");
            }
            key.set_u32("System.IsPinnedToNameSpaceTree", 0)
                .map_err(|e| format!("Failed to set registry value: {e}"))?;
        } else {
            let _ = windows_registry::CURRENT_USER.remove_tree(key_path);
        }
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (clsid, hidden);
        Ok(())
    }
}
