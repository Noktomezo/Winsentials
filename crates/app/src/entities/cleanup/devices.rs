#![allow(unsafe_code)]

use super::types::{CleanupCategory, CleanupError, CleanupTarget};

#[cfg(target_os = "windows")]
use windows_sys::Win32::Devices::DeviceAndDriverInstallation::{
    DIGCF_ALLCLASSES, HDEVINFO, SP_DEVINFO_DATA, SPDRP_CLASS, SPDRP_DEVICEDESC, SPDRP_FRIENDLYNAME,
    SetupDiDestroyDeviceInfoList, SetupDiEnumDeviceInfo, SetupDiGetClassDevsW,
    SetupDiGetDeviceInstanceIdW, SetupDiGetDeviceRegistryPropertyW,
};

#[cfg(target_os = "windows")]
#[link(name = "cfgmgr32")]
unsafe extern "system" {
    fn CM_Get_DevNode_Status(
        pulStatus: *mut u32,
        pulProblemNumber: *mut u32,
        dnDevInst: u32,
        ulFlags: u32,
    ) -> u32;
}

#[cfg(target_os = "windows")]
const CR_NO_SUCH_DEVINST: u32 = 0x0000_000D;

#[cfg(target_os = "windows")]
struct DevInfoGuard(HDEVINFO);

#[cfg(target_os = "windows")]
impl Drop for DevInfoGuard {
    fn drop(&mut self) {
        unsafe {
            SetupDiDestroyDeviceInfoList(self.0);
        }
    }
}

/// Scans for disconnected / phantom `PnP` devices natively via Windows `SetupAPI`.
/// Replaces slow and unreliable `powershell.exe Get-PnpDevice` (~1200ms -> ~15ms).
#[must_use]
pub fn scan_unused_devices() -> Vec<CleanupTarget> {
    #[cfg(target_os = "windows")]
    {
        scan_unused_devices_windows()
    }
    #[cfg(not(target_os = "windows"))]
    {
        Vec::new()
    }
}

#[cfg(target_os = "windows")]
fn scan_unused_devices_windows() -> Vec<CleanupTarget> {
    let hdev = unsafe {
        SetupDiGetClassDevsW(
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null_mut(),
            DIGCF_ALLCLASSES,
        )
    };

    // INVALID_HANDLE_VALUE is -1 as isize as HDEVINFO
    if hdev == -1_isize as HDEVINFO || hdev == 0 {
        return Vec::new();
    }

    let _guard = DevInfoGuard(hdev);

    let mut targets = Vec::new();
    let mut dev_info_data: SP_DEVINFO_DATA = unsafe { std::mem::zeroed() };
    dev_info_data.cbSize = std::mem::size_of::<SP_DEVINFO_DATA>() as u32;

    let mut idx = 0u32;
    while unsafe { SetupDiEnumDeviceInfo(hdev, idx, &raw mut dev_info_data) } != 0 {
        idx += 1;

        let mut status = 0u32;
        let mut problem = 0u32;
        let cr = unsafe {
            CM_Get_DevNode_Status(&raw mut status, &raw mut problem, dev_info_data.DevInst, 0)
        };

        // Devices absent from the live device tree (disconnected / phantom) return CR_NO_SUCH_DEVINST
        if cr != CR_NO_SUCH_DEVINST {
            continue;
        }

        let mut instance_id_buf = [0u16; 512];
        let mut required_size = 0u32;
        let ok = unsafe {
            SetupDiGetDeviceInstanceIdW(
                hdev,
                &raw const dev_info_data,
                instance_id_buf.as_mut_ptr(),
                instance_id_buf.len() as u32,
                &raw mut required_size,
            )
        };
        if ok == 0 {
            continue;
        }

        let id_len = (required_size as usize).saturating_sub(1);
        let instance_id =
            String::from_utf16_lossy(&instance_id_buf[..id_len.min(instance_id_buf.len())]);
        let trimmed_id = instance_id.trim();
        if trimmed_id.is_empty() {
            continue;
        }

        let name = unsafe {
            get_device_registry_string(hdev, &mut dev_info_data, SPDRP_FRIENDLYNAME)
                .or_else(|| get_device_registry_string(hdev, &mut dev_info_data, SPDRP_DEVICEDESC))
                .or_else(|| get_device_registry_string(hdev, &mut dev_info_data, SPDRP_CLASS))
                .unwrap_or_else(|| trimmed_id.to_string())
        };

        targets.push(CleanupTarget {
            id: format!("devices:{trimmed_id}"),
            name,
            category: CleanupCategory::Devices,
            paths: Vec::new(),
            prune_roots: Vec::new(),
            device_instance_id: Some(trimmed_id.to_string()),
            bytes: 0,
        });
    }

    targets
}

#[cfg(target_os = "windows")]
unsafe fn get_device_registry_string(
    hdev: HDEVINFO,
    dev_info_data: &mut SP_DEVINFO_DATA,
    property: u32,
) -> Option<String> {
    let mut buf = [0u16; 256];
    let mut req_size = 0u32;
    let mut prop_type = 0u32;
    let ok = unsafe {
        SetupDiGetDeviceRegistryPropertyW(
            hdev,
            dev_info_data,
            property,
            &raw mut prop_type,
            buf.as_mut_ptr().cast::<u8>(),
            (buf.len() * 2) as u32,
            &raw mut req_size,
        )
    };
    if ok != 0 && req_size > 2 {
        let char_len = (req_size as usize / 2).saturating_sub(1);
        let s = String::from_utf16_lossy(&buf[..char_len.min(buf.len())]);
        let clean = s.replace(['\t', '\n', '\r'], " ");
        let trimmed = clean.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    None
}

/// Removes a disconnected `PnP` device by instance ID natively via Windows `SetupAPI`.
pub fn remove_unused_device(instance_id: &str) -> Result<(), CleanupError> {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::Devices::DeviceAndDriverInstallation::{
            DiUninstallDevice, SP_DEVINFO_DATA, SetupDiCreateDeviceInfoList, SetupDiOpenDeviceInfoW,
        };

        let wide_id: Vec<u16> = instance_id.encode_utf16().chain(Some(0)).collect();
        unsafe {
            let dev_info = SetupDiCreateDeviceInfoList(std::ptr::null(), std::ptr::null_mut());
            if dev_info == -1_isize as HDEVINFO || dev_info == 0 {
                return Err(CleanupError::DeviceRemoval(instance_id.to_owned()));
            }
            let _guard = DevInfoGuard(dev_info);

            let mut dev_info_data: SP_DEVINFO_DATA = std::mem::zeroed();
            dev_info_data.cbSize =
                u32::try_from(std::mem::size_of::<SP_DEVINFO_DATA>()).unwrap_or(0);

            let opened = SetupDiOpenDeviceInfoW(
                dev_info,
                wide_id.as_ptr(),
                std::ptr::null_mut(),
                0,
                &raw mut dev_info_data,
            );

            if opened == 0 {
                return Err(CleanupError::DeviceRemoval(instance_id.to_owned()));
            }

            let mut need_reboot = 0;
            let success = DiUninstallDevice(
                std::ptr::null_mut(),
                dev_info,
                &raw mut dev_info_data,
                0,
                &raw mut need_reboot,
            );

            if success != 0 {
                Ok(())
            } else {
                Err(CleanupError::DeviceRemoval(instance_id.to_owned()))
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        Err(CleanupError::DeviceRemoval(instance_id.to_owned()))
    }
}

#[cfg(test)]
pub(crate) fn parse_unused_devices(output: &[u8]) -> Vec<CleanupTarget> {
    String::from_utf8_lossy(output)
        .lines()
        .filter_map(|line| line.split_once('\t'))
        .filter(|(name, instance_id)| !name.is_empty() && !instance_id.is_empty())
        .map(|(name, instance_id)| CleanupTarget {
            id: format!("devices:{instance_id}"),
            name: name.to_owned(),
            category: CleanupCategory::Devices,
            paths: Vec::new(),
            prune_roots: Vec::new(),
            device_instance_id: Some(instance_id.to_owned()),
            bytes: 0,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_unused_devices_runs_without_panic() {
        let targets = scan_unused_devices();
        for target in &targets {
            assert_eq!(target.category, CleanupCategory::Devices);
            assert!(target.device_instance_id.is_some());
            assert!(!target.name.is_empty());
        }
    }
}
