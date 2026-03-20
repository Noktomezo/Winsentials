use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::shell::refresh_shell_namespace;
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";

const GALLERY_KEY: RegKey = RegKey {
    hive: Hive::CurrentUser,
    path: r"Software\Classes\CLSID\{e8886546-4521-4456-a1a5-e0e4e62243e8}",
};

pub struct HideGalleryNavigationPaneTweak {
    meta: TweakMeta,
}

impl Default for HideGalleryNavigationPaneTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl HideGalleryNavigationPaneTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "hide_gallery_navigation_pane".into(),
                category: "appearance".into(),
                name: "appearance.tweaks.hideGalleryNavigationPane.name".into(),
                short_description: "appearance.tweaks.hideGalleryNavigationPane.shortDescription"
                    .into(),
                detail_description: "appearance.tweaks.hideGalleryNavigationPane.detailDescription"
                    .into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: DISABLED_VALUE.into(),
                risk: RiskLevel::None,
                risk_description: None,
                requires_action: RequiresAction::None,
                min_os_build: Some(22621),
                min_os_ubr: Some(2361),
            },
        }
    }

    fn is_enabled(&self) -> Result<bool, AppError> {
        match GALLERY_KEY.get_dword("System.IsPinnedToNameSpaceTree") {
            Ok(value) => Ok(value == 0),
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }
}

impl Tweak for HideGalleryNavigationPaneTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => {
                GALLERY_KEY.set_dword("System.IsPinnedToNameSpaceTree", 0)?;
                refresh_shell_namespace()
            }
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        GALLERY_KEY.set_dword("System.IsPinnedToNameSpaceTree", 1)?;
        refresh_shell_namespace()
    }

    fn get_status(&self) -> Result<TweakStatus, AppError> {
        let enabled = self.is_enabled()?;
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
