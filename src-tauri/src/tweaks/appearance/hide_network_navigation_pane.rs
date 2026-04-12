use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::shell::refresh_shell_namespace;
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";

const NETWORK_KEY: RegKey = RegKey {
    hive: Hive::CurrentUser,
    path: r"Software\Classes\CLSID\{F02C1A0D-BE21-4350-88B0-7367FC96EF3C}",
};

pub struct HideNetworkNavigationPaneTweak {
    meta: TweakMeta,
}

impl Default for HideNetworkNavigationPaneTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl HideNetworkNavigationPaneTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "hide_network_navigation_pane".into(),
                category: "appearance".into(),
                name: "appearance.tweaks.hideNetworkNavigationPane.name".into(),
                short_description: "appearance.tweaks.hideNetworkNavigationPane.shortDescription"
                    .into(),
                detail_description: "appearance.tweaks.hideNetworkNavigationPane.detailDescription"
                    .into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: DISABLED_VALUE.into(),
                risk: RiskLevel::None,
                risk_description: None,
                conflicts: None,
                requires_action: RequiresAction::None,
                min_os_build: Some(10240),
                min_os_ubr: None,
                min_required_memory_gb: None,
            },
        }
    }

    fn is_enabled(&self) -> Result<bool, AppError> {
        match NETWORK_KEY.get_dword("System.IsPinnedToNameSpaceTree") {
            Ok(value) => Ok(value == 0),
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }
}

impl Tweak for HideNetworkNavigationPaneTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => {
                NETWORK_KEY.set_dword("System.IsPinnedToNameSpaceTree", 0)?;
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
        NETWORK_KEY.set_dword("System.IsPinnedToNameSpaceTree", 1)?;
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
