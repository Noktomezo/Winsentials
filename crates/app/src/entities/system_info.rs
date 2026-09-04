use gpui::SharedString;

#[derive(Clone, Debug)]
pub struct SystemInfo {
    pub os_version: SharedString,
    pub motherboard: SharedString,
    pub username: SharedString,
    pub architecture: SharedString,
    pub build_number: SharedString,
    pub computer_name: SharedString,
    pub tweaks_applied: SharedString,
    pub is_activated: bool,
}

fn normalize_manufacturer(mfg: &str) -> &'static str {
    let lower = mfg.to_lowercase();
    if lower.contains("micro-star") || lower.contains("msi") {
        "MSI"
    } else if lower.contains("asustek") || lower.contains("asus") {
        "ASUS"
    } else if lower.contains("gigabyte") || lower.contains("giga-byte") {
        "Gigabyte"
    } else if lower.contains("asrock") {
        "ASRock"
    } else if lower.contains("evga") {
        "EVGA"
    } else if lower.contains("nzxt") {
        "NZXT"
    } else if lower.contains("biostar") {
        "Biostar"
    } else if lower.contains("colorful") {
        "Colorful"
    } else if lower.contains("hewlett-packard") || lower == "hp" {
        "HP"
    } else if lower.contains("dell") {
        "Dell"
    } else if lower.contains("lenovo") {
        "Lenovo"
    } else if lower.contains("acer") {
        "Acer"
    } else if lower.contains("supermicro") || lower.contains("super micro") {
        "Supermicro"
    } else {
        ""
    }
}

fn strip_parentheses_if_has_name(s: &str) -> String {
    if let Some(open_idx) = s.find('(') {
        if let Some(close_idx) = s.rfind(')') {
            if close_idx > open_idx {
                let outside = format!("{}{}", &s[..open_idx], &s[close_idx + 1..]);
                let trimmed = outside.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
        }
    }
    s.trim().to_string()
}

#[must_use]
pub fn format_motherboard(raw_mfg: &str, raw_product: &str) -> String {
    let mfg_clean = raw_mfg.trim();
    let product_clean = raw_product.trim();

    let is_placeholder = |s: &str| {
        let l = s.to_lowercase();
        l.is_empty()
            || l == "to be filled by o.e.m."
            || l == "default string"
            || l == "system product name"
            || l == "base board product"
    };

    let norm_mfg = normalize_manufacturer(mfg_clean);

    let product_valid = if is_placeholder(product_clean) {
        ""
    } else {
        product_clean
    };

    let product_stripped = strip_parentheses_if_has_name(product_valid);

    if norm_mfg.is_empty() {
        if !product_stripped.is_empty() {
            product_stripped
        } else if !mfg_clean.is_empty() && !is_placeholder(mfg_clean) {
            mfg_clean
                .replace("Co., Ltd.", "")
                .replace("Co.,Ltd.", "")
                .replace("Inc.", "")
                .replace("LLC", "")
                .replace("Corp.", "")
                .replace("Corporation", "")
                .replace("Limited", "")
                .replace("LTD", "")
                .trim()
                .to_string()
        } else {
            "Standard PC".to_string()
        }
    } else if product_stripped.is_empty() {
        norm_mfg.to_string()
    } else if product_stripped
        .to_lowercase()
        .starts_with(&norm_mfg.to_lowercase())
    {
        product_stripped
    } else {
        format!("{norm_mfg} {product_stripped}")
    }
}

use std::sync::Mutex;
use std::time::{Duration, Instant};

static SYSTEM_INFO_CACHE: Mutex<Option<(Instant, SystemInfo)>> = Mutex::new(None);
const SYSTEM_INFO_TTL: Duration = Duration::from_secs(5);

impl SystemInfo {
    #[must_use]
    pub fn fetch() -> Self {
        let now = Instant::now();
        if let Ok(guard) = SYSTEM_INFO_CACHE.lock() {
            if let Some((cached_at, ref info)) = *guard {
                if now.duration_since(cached_at) < SYSTEM_INFO_TTL {
                    return info.clone();
                }
            }
        }

        let info = Self::fetch_live();
        if let Ok(mut guard) = SYSTEM_INFO_CACHE.lock() {
            *guard = Some((now, info.clone()));
        }
        info
    }

    pub fn invalidate_cache() {
        if let Ok(mut guard) = SYSTEM_INFO_CACHE.lock() {
            *guard = None;
        }
    }

    #[must_use]
    fn fetch_live() -> Self {
        let username = std::env::var("USERNAME").unwrap_or_else(|_| "User".to_string());
        let computer_name =
            std::env::var("COMPUTERNAME").unwrap_or_else(|_| "DESKTOP-PC".to_string());
        let architecture = match std::env::consts::ARCH {
            "x86_64" => "x86_64",
            "aarch64" => "ARM64",
            "x86" => "x86 (32-bit)",
            other => other,
        };

        // Query Windows NT CurrentVersion
        let (os_version, build_number) = if let Ok(key) =
            windows_registry::LOCAL_MACHINE.open(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion")
        {
            let product_name: String = key
                .get_string("ProductName")
                .unwrap_or_else(|_| "Windows 11 Pro".to_string());
            let display_version: String = key
                .get_string("DisplayVersion")
                .unwrap_or_else(|_| "24H2".to_string());
            let current_build: String = key
                .get_string("CurrentBuild")
                .unwrap_or_else(|_| "26100".to_string());
            let ubr: u32 = key.get_u32("UBR").unwrap_or(0);

            // Windows 11 registry sometimes still has "Windows 10 Pro" in ProductName for backwards compatibility
            let os_name = if product_name.contains("Windows 10")
                && current_build.parse::<u32>().unwrap_or(0) >= 22000
            {
                product_name.replace("Windows 10", "Windows 11")
            } else {
                product_name
            };

            let os_full = if display_version.is_empty() {
                os_name
            } else {
                format!("{os_name} {display_version}")
            };

            let build_full = if ubr > 0 {
                format!("{current_build}.{ubr}")
            } else {
                current_build
            };

            (os_full, build_full)
        } else {
            ("Windows 11 Pro 24H2".to_string(), "26100.2605".to_string())
        };

        // Query BIOS / Motherboard info with brand normalization
        let motherboard = if let Ok(key) =
            windows_registry::LOCAL_MACHINE.open(r"HARDWARE\DESCRIPTION\System\BIOS")
        {
            let manufacturer: String = key.get_string("BaseBoardManufacturer").unwrap_or_default();
            let product: String = key.get_string("BaseBoardProduct").unwrap_or_default();
            format_motherboard(&manufacturer, &product)
        } else {
            "MSI X670E GAMING PLUS WIFI".to_string()
        };

        let build_u32 = build_number
            .split('.')
            .next()
            .and_then(|s| s.parse::<u32>().ok())
            .unwrap_or(22000);

        let (applied, total) = crate::entities::tweaks::count_applied_tweaks(build_u32);

        Self {
            os_version: os_version.into(),
            motherboard: motherboard.into(),
            username: username.into(),
            architecture: architecture.into(),
            build_number: build_number.into(),
            computer_name: computer_name.into(),
            tweaks_applied: format!("{applied}/{total}").into(),
            is_activated: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_motherboard_msi() {
        assert_eq!(
            format_motherboard("Micro-Star International Co., Ltd.", "MS-7E16"),
            "MSI MS-7E16"
        );
        assert_eq!(
            format_motherboard(
                "Micro-Star International Co., Ltd.",
                "MSI X670E GAMING PLUS WIFI (MS-7E16)"
            ),
            "MSI X670E GAMING PLUS WIFI"
        );
        assert_eq!(
            format_motherboard(
                "Micro-Star International Co., Ltd.",
                "X670E GAMING PLUS WIFI (MS-7E16)"
            ),
            "MSI X670E GAMING PLUS WIFI"
        );
    }

    #[test]
    fn test_format_motherboard_asus() {
        assert_eq!(
            format_motherboard("ASUSTeK COMPUTER INC.", "ROG STRIX B650E-E GAMING WIFI"),
            "ASUS ROG STRIX B650E-E GAMING WIFI"
        );
    }

    #[test]
    fn test_format_motherboard_gigabyte() {
        assert_eq!(
            format_motherboard("Gigabyte Technology Co., Ltd.", "B650 AORUS ELITE AX"),
            "Gigabyte B650 AORUS ELITE AX"
        );
    }
}
