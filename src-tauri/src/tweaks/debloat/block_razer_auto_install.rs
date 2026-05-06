use std::fs;
use std::path::PathBuf;

use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::shell::run_duct;
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";
const CUSTOM_VALUE: &str = "custom";

const DEFAULT_SEARCH_ORDER_CONFIG: u32 = 1;
const BLOCKED_SEARCH_ORDER_CONFIG: u32 = 0;
const DEFAULT_DISABLE_COINSTALLERS: u32 = 0;
const BLOCKED_DISABLE_COINSTALLERS: u32 = 1;

const DRIVER_SEARCHING_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SOFTWARE\Microsoft\Windows\CurrentVersion\DriverSearching",
};

const DEVICE_INSTALLER_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SOFTWARE\Microsoft\Windows\CurrentVersion\Device Installer",
};

pub struct BlockRazerAutoInstallTweak {
    meta: TweakMeta,
}

impl Default for BlockRazerAutoInstallTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl BlockRazerAutoInstallTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "block_razer_auto_install".into(),
                category: "debloat".into(),
                name: "debloat.tweaks.blockRazerAutoInstall.name".into(),
                short_description: "debloat.tweaks.blockRazerAutoInstall.shortDescription".into(),
                detail_description: "debloat.tweaks.blockRazerAutoInstall.detailDescription".into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: ENABLED_VALUE.into(),
                risk: RiskLevel::Low,
                risk_description: Some(
                    "debloat.tweaks.blockRazerAutoInstall.riskDescription".into(),
                ),
                conflicts: None,
                requires_action: RequiresAction::None,
                min_os_build: Some(10240),
                min_os_ubr: None,
                min_required_memory_gb: None,
            },
        }
    }

    fn read_dword_or_default(key: &RegKey, name: &str, default: u32) -> Result<u32, AppError> {
        match key.get_dword(name) {
            Ok(value) => Ok(value),
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => Ok(default),
            Err(error) => Err(error),
        }
    }

    fn is_enabled(&self) -> Result<bool, AppError> {
        Ok(Self::read_dword_or_default(
            &DRIVER_SEARCHING_KEY,
            "SearchOrderConfig",
            DEFAULT_SEARCH_ORDER_CONFIG,
        )? == BLOCKED_SEARCH_ORDER_CONFIG
            && Self::read_dword_or_default(
                &DEVICE_INSTALLER_KEY,
                "DisableCoInstallers",
                DEFAULT_DISABLE_COINSTALLERS,
            )? == BLOCKED_DISABLE_COINSTALLERS)
    }

    fn is_default(&self) -> Result<bool, AppError> {
        Ok(Self::read_dword_or_default(
            &DRIVER_SEARCHING_KEY,
            "SearchOrderConfig",
            DEFAULT_SEARCH_ORDER_CONFIG,
        )? == DEFAULT_SEARCH_ORDER_CONFIG
            && Self::read_dword_or_default(
                &DEVICE_INSTALLER_KEY,
                "DisableCoInstallers",
                DEFAULT_DISABLE_COINSTALLERS,
            )? == DEFAULT_DISABLE_COINSTALLERS)
    }

    fn block_auto_install() -> Result<(), AppError> {
        DRIVER_SEARCHING_KEY.set_dword("SearchOrderConfig", BLOCKED_SEARCH_ORDER_CONFIG)?;
        DEVICE_INSTALLER_KEY.set_dword("DisableCoInstallers", BLOCKED_DISABLE_COINSTALLERS)?;

        let razer_path = razer_installer_path()?;
        if !razer_path.exists() {
            fs::create_dir_all(&razer_path)?;
        }

        run_duct(
            "icacls",
            &[
                razer_path.to_string_lossy().as_ref(),
                "/deny",
                "*S-1-1-0:(W)",
            ],
        )
    }

    fn unblock_auto_install() -> Result<(), AppError> {
        DRIVER_SEARCHING_KEY.set_dword("SearchOrderConfig", DEFAULT_SEARCH_ORDER_CONFIG)?;
        DEVICE_INSTALLER_KEY.set_dword("DisableCoInstallers", DEFAULT_DISABLE_COINSTALLERS)?;
        let razer_path = razer_installer_path()?;
        if !razer_path.exists() {
            return Ok(());
        }

        run_duct(
            "icacls",
            &[
                razer_path.to_string_lossy().as_ref(),
                "/remove:d",
                "*S-1-1-0",
            ],
        )
    }
}

impl Tweak for BlockRazerAutoInstallTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => Self::block_auto_install(),
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        Self::unblock_auto_install()
    }

    fn get_status(&self) -> Result<TweakStatus, AppError> {
        let is_enabled = self.is_enabled()?;
        let is_default = self.is_default()?;

        Ok(TweakStatus {
            current_value: if is_enabled {
                ENABLED_VALUE.into()
            } else if is_default {
                DISABLED_VALUE.into()
            } else {
                CUSTOM_VALUE.into()
            },
            is_default,
        })
    }
}

fn razer_installer_path() -> Result<PathBuf, AppError> {
    Ok(system_root()?.join("Installer").join("Razer"))
}

fn system_root() -> Result<PathBuf, AppError> {
    std::env::var_os("SystemRoot")
        .map(PathBuf::from)
        .ok_or_else(|| AppError::message("SystemRoot is not set"))
}
