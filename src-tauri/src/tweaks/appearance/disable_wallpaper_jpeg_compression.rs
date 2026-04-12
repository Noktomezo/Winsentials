use std::io::ErrorKind;

use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";
const MIN_WINDOWS_10_BUILD: u32 = 10240;

const MAX_JPEG_IMPORT_QUALITY: u32 = 100;

const DESKTOP_KEY: RegKey = RegKey {
    hive: Hive::CurrentUser,
    path: r"Control Panel\Desktop",
};

pub struct DisableWallpaperJpegCompressionTweak {
    meta: TweakMeta,
}

impl Default for DisableWallpaperJpegCompressionTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl DisableWallpaperJpegCompressionTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "disable_wallpaper_jpeg_compression".into(),
                category: "appearance".into(),
                name: "appearance.tweaks.disableWallpaperJpegCompression.name".into(),
                short_description:
                    "appearance.tweaks.disableWallpaperJpegCompression.shortDescription".into(),
                detail_description:
                    "appearance.tweaks.disableWallpaperJpegCompression.detailDescription".into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: ENABLED_VALUE.into(),
                risk: RiskLevel::None,
                risk_description: Some(
                    "appearance.tweaks.disableWallpaperJpegCompression.riskDescription".into(),
                ),
                conflicts: None,
                requires_action: RequiresAction::Logout,
                min_os_build: Some(MIN_WINDOWS_10_BUILD),
                min_os_ubr: None,
                min_required_memory_gb: None,
            },
        }
    }

    fn is_enabled(&self) -> Result<bool, AppError> {
        match DESKTOP_KEY.get_dword("JPEGImportQuality") {
            Ok(value) => Ok(value == MAX_JPEG_IMPORT_QUALITY),
            Err(AppError::Io(error)) if error.kind() == ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }
}

impl Tweak for DisableWallpaperJpegCompressionTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => DESKTOP_KEY.set_dword("JPEGImportQuality", MAX_JPEG_IMPORT_QUALITY),
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        DESKTOP_KEY.delete_value("JPEGImportQuality")
    }

    fn get_status(&self) -> Result<TweakStatus, AppError> {
        let is_enabled = self.is_enabled()?;

        Ok(TweakStatus {
            current_value: if is_enabled {
                ENABLED_VALUE.into()
            } else {
                DISABLED_VALUE.into()
            },
            is_default: !is_enabled,
        })
    }
}
