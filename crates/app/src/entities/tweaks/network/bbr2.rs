const REG_WINSENTIALS: &str = r"Software\Winsentials";
const REG_BBR2_ENABLED: &str = "Bbr2Enabled";

#[must_use]
pub fn is_bbr2_applied() -> bool {
    #[cfg(target_os = "windows")]
    {
        // 1. Check Winsentials recorded state
        if let Ok(key) = windows_registry::LOCAL_MACHINE.open(REG_WINSENTIALS) {
            if let Ok(val) = key.get_u32(REG_BBR2_ENABLED) {
                return val == 1;
            }
        }
        if let Ok(key) = windows_registry::CURRENT_USER.open(REG_WINSENTIALS) {
            if let Ok(val) = key.get_u32(REG_BBR2_ENABLED) {
                return val == 1;
            }
        }

        // 2. Instant registry check via NSI TCP templates (byte 12 == 6 corresponds to BBR2)
        if let Ok(key) = windows_registry::LOCAL_MACHINE
            .open(r"SYSTEM\CurrentControlSet\Control\Nsi\{eb004a03-9b1a-11d4-9123-0050047759bc}\26")
        {
            if let Ok(bytes) = key.get_value("00000000") {
                if bytes.len() > 12 && bytes[12] == 6 {
                    return true;
                }
            }
        }

        // On Windows 10/11, BBR2 is NEVER enabled by default (default is CUBIC/NewReno).
        // Never spawn netsh.exe to query state, as process spawning freezes the UI thread
        // and causes terminal flashes in GUI subsystem applications.
        false
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

#[cfg(target_os = "windows")]
fn run_netsh(args: &[&str], action: &str) -> Result<(), String> {
    crate::shared::process::hidden_cmd("netsh", args)
        .stdout_null()
        .stderr_null()
        .unchecked()
        .run()
        .map_err(|error| format!("{action}: {error}"))
        .and_then(|output| {
            if output.status.success() {
                Ok(())
            } else {
                Err(format!("{action}: netsh exited with {}", output.status))
            }
        })
}

pub fn set_bbr2(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        if applied {
            let templates = [
                "Internet",
                "InternetCustom",
                "Datacenter",
                "DatacenterCustom",
                "Compat",
            ];
            for template in templates {
                let template_arg = format!("template={template}");
                run_netsh(
                    &[
                        "int",
                        "tcp",
                        "set",
                        "supplemental",
                        &template_arg,
                        "congestionprovider=BBR2",
                    ],
                    &format!("Failed to enable BBR2 for {template}"),
                )?;
            }

            // Disabling loopback large MTU prevents connection stalls on localhost
            // (e.g., Steam, WSL, Hyper-V, ADB, local proxies).
            run_netsh(
                &["int", "ipv4", "set", "global", "loopbacklargemtu=disable"],
                "Failed to disable IPv4 loopback large MTU",
            )?;
            run_netsh(
                &["int", "ipv6", "set", "global", "loopbacklargemtu=disable"],
                "Failed to disable IPv6 loopback large MTU",
            )?;
        } else {
            let cubics = [
                "Internet",
                "InternetCustom",
                "Datacenter",
                "DatacenterCustom",
            ];
            for template in cubics {
                let template_arg = format!("template={template}");
                run_netsh(
                    &[
                        "int",
                        "tcp",
                        "set",
                        "supplemental",
                        &template_arg,
                        "congestionprovider=CUBIC",
                    ],
                    &format!("Failed to restore CUBIC for {template}"),
                )?;
            }
            run_netsh(
                &[
                    "int",
                    "tcp",
                    "set",
                    "supplemental",
                    "template=Compat",
                    "congestionprovider=NewReno",
                ],
                "Failed to restore NewReno for Compat",
            )?;

            run_netsh(
                &["int", "ipv4", "set", "global", "loopbacklargemtu=enable"],
                "Failed to restore IPv4 loopback large MTU",
            )?;
            run_netsh(
                &["int", "ipv6", "set", "global", "loopbacklargemtu=enable"],
                "Failed to restore IPv6 loopback large MTU",
            )?;
        }

        let val = u32::from(applied);
        if let Ok(key) = windows_registry::LOCAL_MACHINE.create(REG_WINSENTIALS) {
            let _ = key.set_u32(REG_BBR2_ENABLED, val);
        }
        if let Ok(key) = windows_registry::CURRENT_USER.create(REG_WINSENTIALS) {
            let _ = key.set_u32(REG_BBR2_ENABLED, val);
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
    fn bbr2_check_runs_without_panic() {
        let _ = is_bbr2_applied();
    }
}
