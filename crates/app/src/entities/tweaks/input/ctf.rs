#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CtfOptimizationPreset {
    Standard,
    Mild,
    Aggressive,
}

impl CtfOptimizationPreset {
    pub const ALL: [Self; 3] = [Self::Standard, Self::Mild, Self::Aggressive];

    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Standard => "standard",
            Self::Mild => "mild",
            Self::Aggressive => "aggressive",
        }
    }

    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "standard" => Some(Self::Standard),
            "mild" => Some(Self::Mild),
            "aggressive" => Some(Self::Aggressive),
            _ => None,
        }
    }
}

#[must_use]
pub fn current_ctf_preset() -> CtfOptimizationPreset {
    #[cfg(target_os = "windows")]
    {
        // 1. If Disable Thread Input Manager is set to 1, Aggressive is active
        if let Ok(key) = windows_registry::CURRENT_USER.open(r"Software\Microsoft\CTF") {
            if key
                .get_u32("Disable Thread Input Manager")
                .is_ok_and(|v| v == 1)
            {
                return CtfOptimizationPreset::Aggressive;
            }
        }

        // 2. If MsCtfMonitor scheduled task is disabled, Mild is active
        if is_ms_ctf_monitor_disabled() {
            return CtfOptimizationPreset::Mild;
        }

        CtfOptimizationPreset::Standard
    }
    #[cfg(not(target_os = "windows"))]
    {
        CtfOptimizationPreset::Standard
    }
}

#[cfg(target_os = "windows")]
fn is_ms_ctf_monitor_disabled() -> bool {
    let task_path =
        r"C:\Windows\System32\Tasks\Microsoft\Windows\TextServicesFramework\MsCtfMonitor";
    if let Ok(bytes) = std::fs::read(task_path) {
        let u16_vec: Vec<u16> = bytes
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();
        let utf16_content = String::from_utf16_lossy(&u16_vec);
        if utf16_content.contains("<Enabled>false</Enabled>") {
            return true;
        }
        let utf8_content = String::from_utf8_lossy(&bytes);
        if utf8_content.contains("<Enabled>false</Enabled>") {
            return true;
        }
    }
    false
}

#[cfg(target_os = "windows")]
fn run_hidden_command(program: &str, args: &[&str], action: &str) -> Result<(), String> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let status = std::process::Command::new(program)
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .map_err(|error| format!("{action}: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{action}: process exited with {status}"))
    }
}

#[cfg(target_os = "windows")]
fn remove_registry_value(
    key: &windows_registry::Key,
    name: &str,
    action: &str,
) -> Result<(), String> {
    if key.get_value(name).is_ok() {
        key.remove_value(name)
            .map_err(|error| format!("{action}: {error}"))?;
    }
    Ok(())
}

#[allow(clippy::too_many_lines)]
pub fn set_ctf_preset(preset: CtfOptimizationPreset) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;

        match preset {
            CtfOptimizationPreset::Standard => {
                // Re-enable MsCtfMonitor task
                run_hidden_command(
                    "schtasks",
                    &[
                        "/change",
                        "/tn",
                        r"\Microsoft\Windows\TextServicesFramework\MsCtfMonitor",
                        "/enable",
                    ],
                    "Failed to enable MsCtfMonitor",
                )?;

                run_hidden_command(
                    "schtasks",
                    &[
                        "/run",
                        "/tn",
                        r"\Microsoft\Windows\TextServicesFramework\MsCtfMonitor",
                    ],
                    "Failed to run MsCtfMonitor",
                )?;

                // Remove registry overrides (using create() to ensure KEY_SET_VALUE write access)
                let key = windows_registry::CURRENT_USER
                    .create(r"Software\Microsoft\CTF")
                    .map_err(|error| format!("Failed to open HKCU CTF settings: {error}"))?;
                remove_registry_value(
                    &key,
                    "Disable Thread Input Manager",
                    "Failed to remove Disable Thread Input Manager",
                )?;
                let key = windows_registry::LOCAL_MACHINE
                    .create(r"SOFTWARE\Microsoft\CTF\SystemShared")
                    .map_err(|error| format!("Failed to open HKLM CTF settings: {error}"))?;
                remove_registry_value(&key, "CUAS", "Failed to remove CUAS")?;

                // Start ctfmon.exe if not running
                run_hidden_command(
                    "cmd",
                    &["/C", "start", "", "ctfmon.exe"],
                    "Failed to start ctfmon.exe",
                )?;
            }
            CtfOptimizationPreset::Mild => {
                // Disable MsCtfMonitor task
                run_hidden_command(
                    "schtasks",
                    &[
                        "/change",
                        "/tn",
                        r"\Microsoft\Windows\TextServicesFramework\MsCtfMonitor",
                        "/disable",
                    ],
                    "Failed to disable MsCtfMonitor",
                )?;

                // Remove Disable Thread Input Manager so TSF hooks and language bar remain functional
                let key = windows_registry::CURRENT_USER
                    .create(r"Software\Microsoft\CTF")
                    .map_err(|error| format!("Failed to open HKCU CTF settings: {error}"))?;
                remove_registry_value(
                    &key,
                    "Disable Thread Input Manager",
                    "Failed to remove Disable Thread Input Manager",
                )?;
                let key = windows_registry::LOCAL_MACHINE
                    .create(r"SOFTWARE\Microsoft\CTF\SystemShared")
                    .map_err(|error| format!("Failed to open HKLM CTF settings: {error}"))?;
                remove_registry_value(&key, "CUAS", "Failed to remove CUAS")?;

                // Ensure ctfmon is running for language switching if it was killed previously in Aggressive mode
                run_hidden_command(
                    "cmd",
                    &["/C", "start", "", "ctfmon.exe"],
                    "Failed to start ctfmon.exe",
                )?;
            }
            CtfOptimizationPreset::Aggressive => {
                // Disable MsCtfMonitor task
                run_hidden_command(
                    "schtasks",
                    &[
                        "/change",
                        "/tn",
                        r"\Microsoft\Windows\TextServicesFramework\MsCtfMonitor",
                        "/disable",
                    ],
                    "Failed to disable MsCtfMonitor",
                )?;

                // Disable Thread Input Manager in HKCU
                let key = windows_registry::CURRENT_USER
                    .create(r"Software\Microsoft\CTF")
                    .map_err(|e| {
                        format!("Failed to create HKCU\\Software\\Microsoft\\CTF key: {e}")
                    })?;
                key.set_u32("Disable Thread Input Manager", 1)
                    .map_err(|e| format!("Failed to set Disable Thread Input Manager: {e}"))?;

                // Disable CUAS in HKLM
                let key = windows_registry::LOCAL_MACHINE
                    .create(r"SOFTWARE\Microsoft\CTF\SystemShared")
                    .map_err(|error| format!("Failed to open HKLM CTF settings: {error}"))?;
                key.set_u32("CUAS", 0)
                    .map_err(|error| format!("Failed to set CUAS: {error}"))?;

                // Terminate running ctfmon.exe process
                std::process::Command::new("taskkill")
                    .args(["/F", "/IM", "ctfmon.exe"])
                    .creation_flags(CREATE_NO_WINDOW)
                    .status()
                    .map_err(|error| format!("Failed to stop ctfmon.exe: {error}"))?;
            }
        }
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = preset;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ctf_presets_progressively_configured() {
        assert_eq!(
            CtfOptimizationPreset::from_id("standard"),
            Some(CtfOptimizationPreset::Standard)
        );
        assert_eq!(
            CtfOptimizationPreset::from_id("mild"),
            Some(CtfOptimizationPreset::Mild)
        );
        assert_eq!(
            CtfOptimizationPreset::from_id("aggressive"),
            Some(CtfOptimizationPreset::Aggressive)
        );
        assert_eq!(CtfOptimizationPreset::from_id("unknown"), None);
    }

    #[test]
    fn current_ctf_preset_runs_without_panic() {
        let _ = current_ctf_preset();
    }

    #[test]
    #[ignore = "modifies the live Windows CTF task, registry, and ctfmon.exe process"]
    fn ctf_preset_switching_cleans_registry() {
        assert!(set_ctf_preset(CtfOptimizationPreset::Aggressive).is_ok());
        assert_eq!(current_ctf_preset(), CtfOptimizationPreset::Aggressive);

        assert!(set_ctf_preset(CtfOptimizationPreset::Mild).is_ok());
        assert_eq!(current_ctf_preset(), CtfOptimizationPreset::Mild);

        assert!(set_ctf_preset(CtfOptimizationPreset::Standard).is_ok());
        assert_eq!(current_ctf_preset(), CtfOptimizationPreset::Standard);
    }
}
