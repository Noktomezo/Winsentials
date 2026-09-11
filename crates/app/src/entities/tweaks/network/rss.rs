#[must_use]
pub fn is_rss_applied() -> bool {
    #[cfg(target_os = "windows")]
    {
        // Instant registry check (1 microsecond)
        if let Ok(key) = windows_registry::LOCAL_MACHINE
            .open(r"SYSTEM\CurrentControlSet\Services\Tcpip\Parameters")
        {
            if let Ok(val) = key.get_u32("EnableRSS") {
                return val == 1;
            }
        }

        // On Windows 10/11, Receive-Side Scaling (RSS) is enabled by default in the TCP/IP stack.
        // If the registry key has not been explicitly configured or toggled, return true.
        // Never spawn netsh.exe to query state, as process spawning freezes the UI thread
        // and causes terminal flashes in GUI subsystem applications.
        true
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_rss(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let arg = if applied {
            "rss=enabled"
        } else {
            "rss=disabled"
        };
        let status =
            crate::shared::process::hidden_cmd("netsh", ["int", "tcp", "set", "global", arg])
                .stdout_null()
                .stderr_null()
                .unchecked()
                .run()
                .map_err(|e| format!("Failed to execute netsh: {e}"))?;

        if !status.status.success() {
            return Err("netsh command failed to set RSS".to_string());
        }

        if let Ok(key) = windows_registry::LOCAL_MACHINE
            .create(r"SYSTEM\CurrentControlSet\Services\Tcpip\Parameters")
        {
            let _ = key.set_u32("EnableRSS", u32::from(applied));
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
    fn rss_check_runs_without_panic() {
        let _ = is_rss_applied();
    }
}
