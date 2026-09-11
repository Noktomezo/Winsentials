use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

const CACHE_TTL: Duration = Duration::from_secs(10);

static BBR2_CACHED_STATE: AtomicBool = AtomicBool::new(false);
static BBR2_LAST_CHECK: Mutex<Option<Instant>> = Mutex::new(None);

#[must_use]
pub fn is_bbr2_applied() -> bool {
    #[cfg(target_os = "windows")]
    {
        let now = Instant::now();
        if let Ok(mut last) = BBR2_LAST_CHECK.lock() {
            if let Some(instant) = *last {
                if now.duration_since(instant) < CACHE_TTL {
                    return BBR2_CACHED_STATE.load(Ordering::Relaxed);
                }
            }

            let applied = query_bbr2_live();
            BBR2_CACHED_STATE.store(applied, Ordering::Relaxed);
            *last = Some(now);
            applied
        } else {
            BBR2_CACHED_STATE.load(Ordering::Relaxed)
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

#[cfg(target_os = "windows")]
fn query_bbr2_live() -> bool {
    let output = duct::cmd!(
        "netsh",
        "int",
        "tcp",
        "show",
        "supplemental",
        "template=Internet"
    )
    .stdout_capture()
    .stderr_null()
    .unchecked()
    .run();

    output.is_ok_and(|out| {
        let text = String::from_utf8_lossy(&out.stdout).to_lowercase();
        text.contains("bbr2")
    })
}

#[cfg(target_os = "windows")]
fn run_netsh(args: &[&str], action: &str) -> Result<(), String> {
    duct::cmd("netsh", args)
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

        BBR2_CACHED_STATE.store(applied, Ordering::Relaxed);
        if let Ok(mut last) = BBR2_LAST_CHECK.lock() {
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
    fn bbr2_check_runs_without_panic() {
        let _ = is_bbr2_applied();
    }
}
