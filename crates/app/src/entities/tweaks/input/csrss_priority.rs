const REG_CSRSS_PERF: &str = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\csrss.exe\PerfOptions";
const CPU_PRIORITY_CLASS: &str = "CpuPriorityClass";
const IO_PRIORITY: &str = "IoPriority";
const HIGH_PRIORITY: u32 = 3;

#[must_use]
pub fn is_csrss_priority_applied() -> bool {
    #[cfg(target_os = "windows")]
    {
        windows_registry::LOCAL_MACHINE
            .open(REG_CSRSS_PERF)
            .ok()
            .is_some_and(|key| {
                key.get_u32(CPU_PRIORITY_CLASS) == Ok(HIGH_PRIORITY)
                    && key.get_u32(IO_PRIORITY) == Ok(HIGH_PRIORITY)
            })
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_csrss_priority(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        if applied {
            let key = windows_registry::LOCAL_MACHINE
                .create(REG_CSRSS_PERF)
                .map_err(|e| format!("Failed to create csrss.exe PerfOptions key: {e}"))?;
            key.set_u32(CPU_PRIORITY_CLASS, HIGH_PRIORITY)
                .map_err(|e| format!("Failed to set CpuPriorityClass: {e}"))?;
            key.set_u32(IO_PRIORITY, HIGH_PRIORITY)
                .map_err(|e| format!("Failed to set IoPriority: {e}"))?;
        } else if let Ok(key) = windows_registry::LOCAL_MACHINE.open(REG_CSRSS_PERF) {
            let _ = key.remove_value(CPU_PRIORITY_CLASS);
            let _ = key.remove_value(IO_PRIORITY);
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
    fn csrss_priority_check_runs_without_panic() {
        let _ = is_csrss_priority_applied();
    }
}
