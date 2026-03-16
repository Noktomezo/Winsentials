use winreg::RegKey as WinRegKey;
use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ, KEY_SET_VALUE};

use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::shell::run_netsh;
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";

const TCPIP_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SYSTEM\CurrentControlSet\Services\Tcpip\Parameters",
};

// Network adapter class GUID — identifies all NICs in the device registry.
const NIC_CLASS_PATH: &str =
    r"SYSTEM\CurrentControlSet\Control\Class\{4d36e972-e325-11ce-bfc1-08002be10318}";

/// Sets the `*RSS` registry value on every NIC subkey.
/// Silently skips adapters that don't have the value (virtual/software adapters).
fn set_rss_on_all_adapters(enabled: bool) -> Result<(), AppError> {
    let hklm = WinRegKey::predef(HKEY_LOCAL_MACHINE);
    let class_key = hklm
        .open_subkey_with_flags(NIC_CLASS_PATH, KEY_READ)
        .map_err(AppError::from)?;

    let rss_value: u32 = if enabled { 1 } else { 0 };

    for name in class_key.enum_keys().flatten() {
        // Only process numeric subkeys ("0000", "0001", …); skip "Properties" etc.
        if !name.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }

        let subkey_path = format!(r"{NIC_CLASS_PATH}\{name}");
        let subkey = match hklm.open_subkey_with_flags(&subkey_path, KEY_SET_VALUE) {
            Ok(k) => k,
            Err(_) => continue,
        };

        // Ignore errors — virtual or non-RSS adapters simply won't have this value.
        let _ = subkey.set_value("*RSS", &rss_value);
    }

    Ok(())
}

pub struct EnableNetworkOffloadingTweak {
    meta: TweakMeta,
}

impl Default for EnableNetworkOffloadingTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl EnableNetworkOffloadingTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "enable_network_offloading_rss".into(),
                category: "network".into(),
                name: "network.tweaks.enableNetworkOffloadingRss.name".into(),
                short_description: "network.tweaks.enableNetworkOffloadingRss.shortDescription"
                    .into(),
                detail_description: "network.tweaks.enableNetworkOffloadingRss.detailDescription"
                    .into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: ENABLED_VALUE.into(),
                risk: RiskLevel::Low,
                risk_description: Some(
                    "network.tweaks.enableNetworkOffloadingRss.riskDescription".into(),
                ),
                requires_action: RequiresAction::RestartPc,
                min_os_build: Some(10240),
                min_os_ubr: None,
            },
        }
    }
}

impl Tweak for EnableNetworkOffloadingTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => {
                TCPIP_KEY.set_dword("EnableRSS", 1)?;
                run_netsh(&["int", "tcp", "set", "global", "rss=enabled"])?;
                set_rss_on_all_adapters(true)?;
                Ok(())
            }
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        TCPIP_KEY.delete_value("EnableRSS")?;
        run_netsh(&["int", "tcp", "set", "global", "rss=disabled"])?;
        set_rss_on_all_adapters(false)?;
        Ok(())
    }

    fn get_status(&self) -> Result<TweakStatus, AppError> {
        // Registry read is instant — reflects whether this tweak has been applied.
        let enabled = matches!(TCPIP_KEY.get_dword("EnableRSS"), Ok(1));
        Ok(TweakStatus {
            current_value: if enabled {
                ENABLED_VALUE.into()
            } else {
                DISABLED_VALUE.into()
            },
            is_default: !enabled,
        })
    }
}
