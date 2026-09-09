const REG_ENUM_ROOT: &str = r"SYSTEM\CurrentControlSet\Enum";
const REG_USB_POWER_BACKUP: &str = r"Software\Winsentials\UsbPowerSavingBackup";

const TARGET_POWER_VALUES: &[&str] = &[
    "EnhancedPowerManagementEnabled",
    "AllowIdleIrpInD3",
    "EnableSelectiveSuspend",
    "DeviceSelectiveSuspended",
    "SelectiveSuspendEnabled",
    "SelectiveSuspendOn",
    "EnumerationRetryCount",
    "ExtPropDescSemaphore",
    "WaitWakeEnabled",
    "D3ColdSupported",
    "WdfDirectedPowerTransitionEnable",
    "EnableIdlePowerManagement",
    "IdleInWorkingState",
];

#[must_use]
pub fn is_usb_power_saving_disabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        let mut found_any = false;
        let any_enabled = has_any_power_saving_enabled("", &mut found_any, 0);
        found_any && !any_enabled
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_usb_power_saving_disabled(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        if applied {
            let mut entries = Vec::new();
            scan_power_saving_entries("", &mut entries, 0);
            if entries.is_empty() {
                return Err("No USB power management entries found in registry".to_string());
            }

            // Save original values into HKCU backup so reverting restores exact values
            if let Ok(backup_key) = windows_registry::CURRENT_USER.create(REG_USB_POWER_BACKUP) {
                for (path, name, val) in &entries {
                    let backup_item = format!("{path}::{name}");
                    let _ = backup_key.set_u32(&backup_item, *val);
                }
            }

            let mut success_count = 0;
            let mut error_count = 0;
            for (path, name, _) in &entries {
                if let Ok(key) = windows_registry::LOCAL_MACHINE.create(path) {
                    if key.set_u32(name, 0).is_ok() {
                        success_count += 1;
                    } else {
                        error_count += 1;
                    }
                } else {
                    error_count += 1;
                }
            }

            if success_count == 0 && error_count > 0 {
                return Err(
                    "Failed to update USB power management settings. Administrator privileges may be required."
                        .to_string(),
                );
            }
        } else {
            // Restore from backup if present
            let mut restored_count = 0;
            if let Ok(backup_key) = windows_registry::CURRENT_USER.open(REG_USB_POWER_BACKUP) {
                if let Ok(values) = backup_key.values() {
                    for (backup_item, _) in values {
                        if let Some((path, name)) = backup_item.split_once("::") {
                            if let Ok(orig_val) = backup_key.get_u32(&backup_item) {
                                if let Ok(key) = windows_registry::LOCAL_MACHINE.create(path) {
                                    if key.set_u32(name, orig_val).is_ok() {
                                        restored_count += 1;
                                    }
                                }
                            }
                        }
                    }
                }
                let _ = windows_registry::CURRENT_USER.remove_tree(REG_USB_POWER_BACKUP);
            }

            // If backup was missing, fallback to setting power saving defaults back to 1
            if restored_count == 0 {
                let mut entries = Vec::new();
                scan_power_saving_entries("", &mut entries, 0);
                for (path, name, _) in entries {
                    let default_val = u32::from(name != "EnumerationRetryCount");
                    if let Ok(key) = windows_registry::LOCAL_MACHINE.create(&path) {
                        let _ = key.set_u32(name, default_val);
                    }
                }
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

#[cfg(target_os = "windows")]
fn has_any_power_saving_enabled(rel_path: &str, found_any: &mut bool, depth: usize) -> bool {
    if depth > 8 {
        return false;
    }
    let full_path = if rel_path.is_empty() {
        REG_ENUM_ROOT.to_string()
    } else {
        format!("{REG_ENUM_ROOT}\\{rel_path}")
    };

    let Ok(key) = windows_registry::LOCAL_MACHINE.open(&full_path) else {
        return false;
    };

    for &target in TARGET_POWER_VALUES {
        if let Ok(val) = key.get_u32(target) {
            *found_any = true;
            if val != 0 {
                return true;
            }
        }
    }

    if let Ok(subkeys) = key.keys() {
        for subkey in subkeys {
            let child_rel = if rel_path.is_empty() {
                subkey
            } else {
                format!("{rel_path}\\{subkey}")
            };
            if has_any_power_saving_enabled(&child_rel, found_any, depth + 1) {
                return true;
            }
        }
    }

    false
}

#[cfg(target_os = "windows")]
fn scan_power_saving_entries(
    rel_path: &str,
    entries: &mut Vec<(String, &'static str, u32)>,
    depth: usize,
) {
    if depth > 8 {
        return;
    }
    let full_path = if rel_path.is_empty() {
        REG_ENUM_ROOT.to_string()
    } else {
        format!("{REG_ENUM_ROOT}\\{rel_path}")
    };

    let Ok(key) = windows_registry::LOCAL_MACHINE.open(&full_path) else {
        return;
    };

    for &target in TARGET_POWER_VALUES {
        if let Ok(val) = key.get_u32(target) {
            entries.push((full_path.clone(), target, val));
        }
    }

    if let Ok(subkeys) = key.keys() {
        for subkey in subkeys {
            let child_rel = if rel_path.is_empty() {
                subkey
            } else {
                format!("{rel_path}\\{subkey}")
            };
            scan_power_saving_entries(&child_rel, entries, depth + 1);
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
        assert!(!TARGET_POWER_VALUES.is_empty());
        assert_eq!(TARGET_POWER_VALUES.len(), 13);
    }
}
