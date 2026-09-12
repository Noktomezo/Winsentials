const REG_POWER_SCHEMES: &str = r"SYSTEM\CurrentControlSet\Control\Power\User\PowerSchemes";
const SUB_USB_GUID: &str = "2a737441-1930-4402-8d77-b2bebba308a3";
const SETTING_USB_SELECTIVE_SUSPEND_GUID: &str = "48e6b7a6-50f5-4782-a5d4-53bb8f07e226";

const REG_USBHUB3_PARAMS: &str = r"SYSTEM\CurrentControlSet\Services\USBHUB3\Parameters";
const REG_USBHUB_PARAMS: &str = r"SYSTEM\CurrentControlSet\Services\USBHUB\Parameters";
const VAL_DISABLE_SELECTIVE_SUSPEND: &str = "DisableSelectiveSuspend";

const REG_USB_ENUM: &str = r"SYSTEM\CurrentControlSet\Enum\USB";
const REG_USBSTOR_ENUM: &str = r"SYSTEM\CurrentControlSet\Enum\USBSTOR";

const REG_WINSENTIALS_USB_STATE: &str = r"Software\Winsentials";
const VAL_WINSENTIALS_USB_DISABLED: &str = "UsbPowerSavingDisabled";
const REG_USB_POWER_BACKUP: &str = r"Software\Winsentials\UsbPowerSavingBackup";

const TARGET_DEVICE_POWER_VALUES: &[&str] = &[
    "EnhancedPowerManagementEnabled",
    "AllowIdleIrpInD3",
    "EnableSelectiveSuspend",
    "DeviceSelectiveSuspended",
    "SelectiveSuspendEnabled",
    "SelectiveSuspendOn",
    "D3ColdSupported",
    "EnableIdlePowerManagement",
    "IdleInWorkingState",
];

#[must_use]
pub fn is_usb_power_saving_disabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        // 1. Check Winsentials explicit tweak toggle state
        if let Ok(key) = windows_registry::LOCAL_MACHINE.open(REG_WINSENTIALS_USB_STATE) {
            if let Ok(val) = key.get_u32(VAL_WINSENTIALS_USB_DISABLED) {
                return val == 1;
            }
        }
        if let Ok(key) = windows_registry::CURRENT_USER.open(REG_WINSENTIALS_USB_STATE) {
            if let Ok(val) = key.get_u32(VAL_WINSENTIALS_USB_DISABLED) {
                return val == 1;
            }
        }

        // 2. Check active Windows Power Scheme USB Selective Suspend and USBHUB3 driver parameter
        is_power_scheme_usb_suspend_disabled() && is_usbhub_selective_suspend_disabled()
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

#[cfg(target_os = "windows")]
fn is_power_scheme_usb_suspend_disabled() -> bool {
    let Ok(schemes_key) = windows_registry::LOCAL_MACHINE.open(REG_POWER_SCHEMES) else {
        return false;
    };
    let Ok(active_guid) = schemes_key.get_string("ActivePowerScheme") else {
        return false;
    };
    let path = format!(
        "{REG_POWER_SCHEMES}\\{active_guid}\\{SUB_USB_GUID}\\{SETTING_USB_SELECTIVE_SUSPEND_GUID}"
    );
    if let Ok(key) = windows_registry::LOCAL_MACHINE.open(&path) {
        if let Ok(ac_val) = key.get_u32("ACSettingIndex") {
            return ac_val == 0;
        }
    }
    false
}

#[cfg(target_os = "windows")]
fn is_usbhub_selective_suspend_disabled() -> bool {
    if let Ok(key) = windows_registry::LOCAL_MACHINE.open(REG_USBHUB3_PARAMS) {
        if let Ok(val) = key.get_u32(VAL_DISABLE_SELECTIVE_SUSPEND) {
            return val == 1;
        }
    }
    false
}

pub fn set_usb_power_saving_disabled(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        if applied {
            apply_usb_power_saving_disabled()
        } else {
            restore_usb_power_saving();
            Ok(())
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = applied;
        Ok(())
    }
}

#[cfg(target_os = "windows")]
fn apply_usb_power_saving_disabled() -> Result<(), String> {
    // 1. Collect and back up original values
    let mut backup_entries = Vec::new();
    collect_power_scheme_entries(&mut backup_entries);
    collect_driver_entries(&mut backup_entries);
    collect_device_entries(REG_USB_ENUM, &mut backup_entries);
    collect_device_entries(REG_USBSTOR_ENUM, &mut backup_entries);

    if let Ok(backup_key) = windows_registry::CURRENT_USER.create(REG_USB_POWER_BACKUP) {
        for (path, name, val) in &backup_entries {
            let backup_item = format!("{path}::{name}");
            let _ = backup_key.set_u32(&backup_item, *val);
        }
    }

    let mut applied_any = false;

    // 2. Disable USB Selective Suspend across all configured Windows Power Schemes
    if let Ok(schemes_key) = windows_registry::LOCAL_MACHINE.open(REG_POWER_SCHEMES) {
        if let Ok(subkeys) = schemes_key.keys() {
            for scheme_guid in subkeys {
                let path = format!(
                    "{REG_POWER_SCHEMES}\\{scheme_guid}\\{SUB_USB_GUID}\\{SETTING_USB_SELECTIVE_SUSPEND_GUID}"
                );
                if let Ok(key) = windows_registry::LOCAL_MACHINE.create(&path) {
                    let _ = key.set_u32("ACSettingIndex", 0);
                    let _ = key.set_u32("DCSettingIndex", 0);
                    applied_any = true;
                }
            }
        }
    }

    // 3. Configure USBHUB3 and USBHUB driver parameters
    if let Ok(key) = windows_registry::LOCAL_MACHINE.create(REG_USBHUB3_PARAMS) {
        let _ = key.set_u32(VAL_DISABLE_SELECTIVE_SUSPEND, 1);
        applied_any = true;
    }
    if let Ok(key) = windows_registry::LOCAL_MACHINE.create(REG_USBHUB_PARAMS) {
        let _ = key.set_u32(VAL_DISABLE_SELECTIVE_SUSPEND, 1);
    }

    // 4. Disable power management on root hubs and connected USB devices
    apply_to_usb_devices(REG_USB_ENUM);
    apply_to_usb_devices(REG_USBSTOR_ENUM);

    if !applied_any {
        return Err(
            "Failed to update USB power management settings. Administrator privileges required."
                .to_string(),
        );
    }

    // 5. Persist Winsentials state
    let _ = windows_registry::LOCAL_MACHINE
        .create(REG_WINSENTIALS_USB_STATE)
        .and_then(|key| key.set_u32(VAL_WINSENTIALS_USB_DISABLED, 1));
    let _ = windows_registry::CURRENT_USER
        .create(REG_WINSENTIALS_USB_STATE)
        .and_then(|key| key.set_u32(VAL_WINSENTIALS_USB_DISABLED, 1));

    Ok(())
}

#[cfg(target_os = "windows")]
fn restore_usb_power_saving() {
    let mut restored_from_backup = false;

    if let Ok(backup_key) = windows_registry::CURRENT_USER.open(REG_USB_POWER_BACKUP) {
        if let Ok(values) = backup_key.values() {
            for (backup_item, _) in values {
                if let Some((path, name)) = backup_item.split_once("::") {
                    if let Ok(orig_val) = backup_key.get_u32(&backup_item) {
                        if let Ok(key) = windows_registry::LOCAL_MACHINE.create(path) {
                            let _ = key.set_u32(name, orig_val);
                            restored_from_backup = true;
                        }
                    }
                }
            }
        }
        let _ = windows_registry::CURRENT_USER.remove_tree(REG_USB_POWER_BACKUP);
    }

    // If backup was absent, restore standard Windows defaults
    if !restored_from_backup {
        if let Ok(schemes_key) = windows_registry::LOCAL_MACHINE.open(REG_POWER_SCHEMES) {
            if let Ok(subkeys) = schemes_key.keys() {
                for scheme_guid in subkeys {
                    let path = format!(
                        "{REG_POWER_SCHEMES}\\{scheme_guid}\\{SUB_USB_GUID}\\{SETTING_USB_SELECTIVE_SUSPEND_GUID}"
                    );
                    if let Ok(key) = windows_registry::LOCAL_MACHINE.create(&path) {
                        let _ = key.set_u32("ACSettingIndex", 1);
                        let _ = key.set_u32("DCSettingIndex", 1);
                    }
                }
            }
        }

        if let Ok(key) = windows_registry::LOCAL_MACHINE.open(REG_USBHUB3_PARAMS) {
            let _ = key.remove_value(VAL_DISABLE_SELECTIVE_SUSPEND);
        }
        if let Ok(key) = windows_registry::LOCAL_MACHINE.open(REG_USBHUB_PARAMS) {
            let _ = key.remove_value(VAL_DISABLE_SELECTIVE_SUSPEND);
        }

        restore_default_usb_devices(REG_USB_ENUM);
        restore_default_usb_devices(REG_USBSTOR_ENUM);
    }

    // Persist Winsentials state
    let _ = windows_registry::LOCAL_MACHINE
        .create(REG_WINSENTIALS_USB_STATE)
        .and_then(|key| key.set_u32(VAL_WINSENTIALS_USB_DISABLED, 0));
    let _ = windows_registry::CURRENT_USER
        .create(REG_WINSENTIALS_USB_STATE)
        .and_then(|key| key.set_u32(VAL_WINSENTIALS_USB_DISABLED, 0));
}

#[cfg(target_os = "windows")]
fn collect_power_scheme_entries(entries: &mut Vec<(String, &'static str, u32)>) {
    let Ok(schemes_key) = windows_registry::LOCAL_MACHINE.open(REG_POWER_SCHEMES) else {
        return;
    };
    let Ok(subkeys) = schemes_key.keys() else {
        return;
    };
    for scheme_guid in subkeys {
        let path = format!(
            "{REG_POWER_SCHEMES}\\{scheme_guid}\\{SUB_USB_GUID}\\{SETTING_USB_SELECTIVE_SUSPEND_GUID}"
        );
        if let Ok(key) = windows_registry::LOCAL_MACHINE.open(&path) {
            if let Ok(ac) = key.get_u32("ACSettingIndex") {
                entries.push((path.clone(), "ACSettingIndex", ac));
            }
            if let Ok(dc) = key.get_u32("DCSettingIndex") {
                entries.push((path, "DCSettingIndex", dc));
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn collect_driver_entries(entries: &mut Vec<(String, &'static str, u32)>) {
    if let Ok(key) = windows_registry::LOCAL_MACHINE.open(REG_USBHUB3_PARAMS) {
        if let Ok(val) = key.get_u32(VAL_DISABLE_SELECTIVE_SUSPEND) {
            entries.push((
                REG_USBHUB3_PARAMS.to_string(),
                VAL_DISABLE_SELECTIVE_SUSPEND,
                val,
            ));
        }
    }
    if let Ok(key) = windows_registry::LOCAL_MACHINE.open(REG_USBHUB_PARAMS) {
        if let Ok(val) = key.get_u32(VAL_DISABLE_SELECTIVE_SUSPEND) {
            entries.push((
                REG_USBHUB_PARAMS.to_string(),
                VAL_DISABLE_SELECTIVE_SUSPEND,
                val,
            ));
        }
    }
}

#[cfg(target_os = "windows")]
fn collect_device_entries(enum_root: &str, entries: &mut Vec<(String, &'static str, u32)>) {
    let Ok(root_key) = windows_registry::LOCAL_MACHINE.open(enum_root) else {
        return;
    };
    let Ok(hw_keys) = root_key.keys() else {
        return;
    };

    for hw in hw_keys {
        let hw_path = format!("{enum_root}\\{hw}");
        let Ok(hw_key) = windows_registry::LOCAL_MACHINE.open(&hw_path) else {
            continue;
        };
        let Ok(instances) = hw_key.keys() else {
            continue;
        };

        for inst in instances {
            let inst_path = format!("{hw_path}\\{inst}");

            // Check Root Hub WDF IdleInWorkingState
            if hw.starts_with("ROOT_HUB") {
                let wdf_path = format!("{inst_path}\\Device Parameters\\WDF");
                if let Ok(wdf_key) = windows_registry::LOCAL_MACHINE.open(&wdf_path) {
                    if let Ok(val) = wdf_key.get_u32("IdleInWorkingState") {
                        entries.push((wdf_path, "IdleInWorkingState", val));
                    }
                }
            }

            // Check Device Parameters
            let dev_params_path = format!("{inst_path}\\Device Parameters");
            if let Ok(dp_key) = windows_registry::LOCAL_MACHINE.open(&dev_params_path) {
                for &target in TARGET_DEVICE_POWER_VALUES {
                    if let Ok(val) = dp_key.get_u32(target) {
                        entries.push((dev_params_path.clone(), target, val));
                    }
                }
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn apply_to_usb_devices(enum_root: &str) {
    let Ok(root_key) = windows_registry::LOCAL_MACHINE.open(enum_root) else {
        return;
    };
    let Ok(hw_keys) = root_key.keys() else {
        return;
    };

    for hw in hw_keys {
        let hw_path = format!("{enum_root}\\{hw}");
        let Ok(hw_key) = windows_registry::LOCAL_MACHINE.open(&hw_path) else {
            continue;
        };
        let Ok(instances) = hw_key.keys() else {
            continue;
        };

        for inst in instances {
            let inst_path = format!("{hw_path}\\{inst}");

            if hw.starts_with("ROOT_HUB") {
                let wdf_path = format!("{inst_path}\\Device Parameters\\WDF");
                if let Ok(wdf_key) = windows_registry::LOCAL_MACHINE.create(&wdf_path) {
                    let _ = wdf_key.set_u32("IdleInWorkingState", 0);
                }
            }

            let dev_params_path = format!("{inst_path}\\Device Parameters");
            if let Ok(dp_key) = windows_registry::LOCAL_MACHINE.create(&dev_params_path) {
                let _ = dp_key.set_u32("EnhancedPowerManagementEnabled", 0);
                let _ = dp_key.set_u32("AllowIdleIrpInD3", 0);
                let _ = dp_key.set_u32("DeviceSelectiveSuspended", 0);
                let _ = dp_key.set_u32("SelectiveSuspendEnabled", 0);
                let _ = dp_key.set_u32("SelectiveSuspendOn", 0);

                if dp_key.get_u32("D3ColdSupported").is_ok() {
                    let _ = dp_key.set_u32("D3ColdSupported", 0);
                }
                if dp_key.get_u32("EnableIdlePowerManagement").is_ok() {
                    let _ = dp_key.set_u32("EnableIdlePowerManagement", 0);
                }
                if dp_key.get_u32("IdleInWorkingState").is_ok() {
                    let _ = dp_key.set_u32("IdleInWorkingState", 0);
                }
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn restore_default_usb_devices(enum_root: &str) {
    let Ok(root_key) = windows_registry::LOCAL_MACHINE.open(enum_root) else {
        return;
    };
    let Ok(hw_keys) = root_key.keys() else {
        return;
    };

    for hw in hw_keys {
        let hw_path = format!("{enum_root}\\{hw}");
        let Ok(hw_key) = windows_registry::LOCAL_MACHINE.open(&hw_path) else {
            continue;
        };
        let Ok(instances) = hw_key.keys() else {
            continue;
        };

        for inst in instances {
            let inst_path = format!("{hw_path}\\{inst}");

            if hw.starts_with("ROOT_HUB") {
                let wdf_path = format!("{inst_path}\\Device Parameters\\WDF");
                if let Ok(wdf_key) = windows_registry::LOCAL_MACHINE.create(&wdf_path) {
                    let _ = wdf_key.set_u32("IdleInWorkingState", 1);
                }
            }

            let dev_params_path = format!("{inst_path}\\Device Parameters");
            if let Ok(dp_key) = windows_registry::LOCAL_MACHINE.create(&dev_params_path) {
                let _ = dp_key.set_u32("EnhancedPowerManagementEnabled", 1);
                let _ = dp_key.set_u32("AllowIdleIrpInD3", 1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usb_power_saving_check_runs_without_panic() {
        let _ = is_usb_power_saving_disabled();
    }

    #[test]
    fn usb_power_saving_target_values_not_empty() {
        assert!(!TARGET_DEVICE_POWER_VALUES.is_empty());
        assert_eq!(TARGET_DEVICE_POWER_VALUES.len(), 9);
    }
}
