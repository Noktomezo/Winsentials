use std::process::Command;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const CACHE_TTL: Duration = Duration::from_secs(10);

static RSS_CACHED_STATE: AtomicBool = AtomicBool::new(false);
static RSS_LAST_CHECK: Mutex<Option<Instant>> = Mutex::new(None);

#[must_use]
pub fn is_rss_applied() -> bool {
    #[cfg(target_os = "windows")]
    {
        // 1. Instant registry check (1 microsecond)
        if let Ok(key) = windows_registry::LOCAL_MACHINE
            .open(r"SYSTEM\CurrentControlSet\Services\Tcpip\Parameters")
        {
            if let Ok(val) = key.get_u32("EnableRSS") {
                let applied = val == 1;
                RSS_CACHED_STATE.store(applied, Ordering::Relaxed);
                return applied;
            }
        }

        // 2. TTL cache for fallback netsh check
        let now = Instant::now();
        if let Ok(mut last) = RSS_LAST_CHECK.lock() {
            if let Some(instant) = *last {
                if now.duration_since(instant) < CACHE_TTL {
                    return RSS_CACHED_STATE.load(Ordering::Relaxed);
                }
            }

            let applied = query_rss_live();
            RSS_CACHED_STATE.store(applied, Ordering::Relaxed);
            *last = Some(now);
            applied
        } else {
            RSS_CACHED_STATE.load(Ordering::Relaxed)
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

#[cfg(target_os = "windows")]
fn query_rss_live() -> bool {
    let output = Command::new("netsh")
        .args(["int", "tcp", "show", "global"])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    output.is_ok_and(|out| {
        let text = String::from_utf8_lossy(&out.stdout).to_lowercase();
        for line in text.lines() {
            if (line.contains("receive-side scaling")
                || line.contains("масштабирования на стороне приема"))
                && line.contains("enabled")
            {
                return true;
            }
        }
        false
    })
}

pub fn set_rss(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let arg = if applied {
            "rss=enabled"
        } else {
            "rss=disabled"
        };
        let status = Command::new("netsh")
            .args(["int", "tcp", "set", "global", arg])
            .creation_flags(CREATE_NO_WINDOW)
            .status()
            .map_err(|e| format!("Failed to execute netsh: {e}"))?;

        if !status.success() {
            return Err("netsh command failed to set RSS".to_string());
        }

        if let Ok(key) = windows_registry::LOCAL_MACHINE
            .create(r"SYSTEM\CurrentControlSet\Services\Tcpip\Parameters")
        {
            let _ = key.set_u32("EnableRSS", u32::from(applied));
        }

        RSS_CACHED_STATE.store(applied, Ordering::Relaxed);
        if let Ok(mut last) = RSS_LAST_CHECK.lock() {
            *last = Some(Instant::now());
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
