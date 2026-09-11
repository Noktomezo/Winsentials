const REG_NET_CLASS: &str =
    r"SYSTEM\CurrentControlSet\Control\Class\{4d36e972-e325-11ce-bfc1-08002be10318}";
const REG_NET_POWER_BACKUP: &str = r"Software\Winsentials\NetworkPowerSavingBackup";
const MAX_ADAPTER_SLOTS: u32 = 64;

const PROP_PNP_CAPABILITIES: &str = "PnPCapabilities";
/// Bitmask value 24 (0x18) in NDIS `PnPCapabilities` disables "Allow the computer to turn off this device to save power".
const PNP_CAP_DISABLE_POWER_OFF: u32 = 24;

const TARGET_NET_PROPERTIES: &[&str] = &[
    "SipsEnabled",
    "*SipsEnabled",
    "EEE",
    "*EEE",
    "ReduceSpeedOnPowerDown",
    "*ReduceSpeedOnPowerDown",
    "ULPMode",
    "*ULPMode",
    "EEELinkAdvertisement",
    "*EEELinkAdvertisement",
    "EnableGreenEthernet",
    "*EnableGreenEthernet",
    "AdvancedEEE",
    "*AdvancedEEE",
    "GigaLite",
    "*GigaLite",
    "PowerSavingMode",
    "*PowerSavingMode",
    "ASPM",
    "*ASPM",
    "SelectiveSuspend",
    "*SelectiveSuspend",
];

#[must_use]
pub fn is_network_power_saving_disabled() -> bool {
    #[cfg(target_os = "windows")]
    {
        let mut found_any = false;
        for i in 0..MAX_ADAPTER_SLOTS {
            let sub = format!(r"{REG_NET_CLASS}\{i:04}");
            if let Ok(key) = windows_registry::LOCAL_MACHINE.open(&sub) {
                for &prop in TARGET_NET_PROPERTIES {
                    if let Ok(s) = key.get_string(prop) {
                        found_any = true;
                        if s.trim() != "0" {
                            return false;
                        }
                    } else if let Ok(v) = key.get_u32(prop) {
                        found_any = true;
                        if v != 0 {
                            return false;
                        }
                    }
                }
            }
        }
        found_any
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_network_power_saving_disabled(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        if applied {
            // 1. Scan and backup existing properties
            let mut entries: Vec<(String, &'static str, Option<String>, Option<u32>)> = Vec::new();
            for i in 0..MAX_ADAPTER_SLOTS {
                let sub = format!(r"{REG_NET_CLASS}\{i:04}");
                if let Ok(key) = windows_registry::LOCAL_MACHINE.open(&sub) {
                    for &prop in TARGET_NET_PROPERTIES {
                        if let Ok(s) = key.get_string(prop) {
                            entries.push((sub.clone(), prop, Some(s), None));
                        } else if let Ok(v) = key.get_u32(prop) {
                            entries.push((sub.clone(), prop, None, Some(v)));
                        }
                    }

                    // Manage OS-level power saving checkbox via PnPCapabilities directly in registry
                    if key.get_string("NetCfgInstanceId").is_ok() {
                        if let Ok(cap) = key.get_u32(PROP_PNP_CAPABILITIES) {
                            entries.push((sub.clone(), PROP_PNP_CAPABILITIES, None, Some(cap)));
                        } else {
                            entries.push((sub.clone(), PROP_PNP_CAPABILITIES, None, Some(0)));
                        }
                    }
                }
            }

            if entries.is_empty() {
                return Err(
                    "No network adapter power management properties found in registry".to_string(),
                );
            }

            // Save backup in HKCU
            if let Ok(backup_key) = windows_registry::CURRENT_USER.create(REG_NET_POWER_BACKUP) {
                for (path, prop, str_val, u32_val) in &entries {
                    let item_key = format!("{path}::{prop}");
                    if let Some(s) = str_val {
                        let _ = backup_key.set_string(format!("str::{item_key}"), s);
                    } else if let Some(v) = u32_val {
                        let _ = backup_key.set_u32(format!("u32::{item_key}"), *v);
                    }
                }
            }

            // 2. Set all found properties (target props to 0, PnPCapabilities to 24)
            let mut success_count = 0;
            let mut error_count = 0;
            for (path, prop, str_val, _) in &entries {
                if let Ok(key) = windows_registry::LOCAL_MACHINE.create(path) {
                    let res = if *prop == PROP_PNP_CAPABILITIES {
                        key.set_u32(prop, PNP_CAP_DISABLE_POWER_OFF)
                    } else if str_val.is_some() {
                        key.set_string(prop, "0")
                    } else {
                        key.set_u32(prop, 0)
                    };
                    if res.is_ok() {
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
                    "Failed to update network adapter registry keys. Administrator privileges may be required."
                        .to_string(),
                );
            }
        } else {
            // Revert: restore from backup if present
            let mut restored = false;
            if let Ok(backup_key) = windows_registry::CURRENT_USER.open(REG_NET_POWER_BACKUP) {
                if let Ok(values) = backup_key.values() {
                    for (item_name, _) in values {
                        if let Some(rest) = item_name.strip_prefix("str::") {
                            if let Some((path, prop)) = rest.split_once("::") {
                                if let Ok(orig_str) = backup_key.get_string(&item_name) {
                                    if let Ok(key) = windows_registry::LOCAL_MACHINE.create(path) {
                                        let _ = key.set_string(prop, &orig_str);
                                        restored = true;
                                    }
                                }
                            }
                        } else if let Some(rest) = item_name.strip_prefix("u32::") {
                            if let Some((path, prop)) = rest.split_once("::") {
                                if let Ok(orig_val) = backup_key.get_u32(&item_name) {
                                    if let Ok(key) = windows_registry::LOCAL_MACHINE.create(path) {
                                        if prop == PROP_PNP_CAPABILITIES && orig_val == 0 {
                                            let _ = key.remove_value(prop);
                                        } else {
                                            let _ = key.set_u32(prop, orig_val);
                                        }
                                        restored = true;
                                    }
                                }
                            }
                        }
                    }
                }
                let _ = windows_registry::CURRENT_USER.remove_tree(REG_NET_POWER_BACKUP);
            }

            // Fallback if no backup: restore known defaults to "1" and remove PnPCapabilities override
            if !restored {
                for i in 0..MAX_ADAPTER_SLOTS {
                    let sub = format!(r"{REG_NET_CLASS}\{i:04}");
                    if let Ok(key) = windows_registry::LOCAL_MACHINE.create(&sub) {
                        let _ = key.remove_value(PROP_PNP_CAPABILITIES);
                        for &prop in TARGET_NET_PROPERTIES {
                            if key.get_string(prop).is_ok() {
                                let _ = key.set_string(prop, "1");
                            } else if key.get_u32(prop).is_ok() {
                                let _ = key.set_u32(prop, 1);
                            }
                        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn network_power_saving_check_runs_without_panic() {
        let _ = is_network_power_saving_disabled();
    }

    #[test]
    fn target_net_properties_not_empty() {
        assert!(!TARGET_NET_PROPERTIES.is_empty());
        assert_eq!(TARGET_NET_PROPERTIES.len(), 22);
    }
}
