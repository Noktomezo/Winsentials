use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::shell::refresh_shell_namespace;
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";
const HOME_NAMESPACE_DEFAULT_VALUE: &str = "CLSID_MSGraphHomeFolder";

const HOME_NAMESPACE_KEYS: [RegKey; 2] = [
    RegKey {
        hive: Hive::LocalMachine,
        path: r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Desktop\NameSpace_36354489\{f874310e-b6b7-47dc-bc84-b9e6b38f5903}",
    },
    RegKey {
        hive: Hive::LocalMachine,
        path: r"SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Desktop\NameSpace_36354489\{f8743c1a-8487-11cf-b54b-c0d081744f91}",
    },
];

pub struct HideHomeNavigationPaneTweak {
    meta: TweakMeta,
}

impl Default for HideHomeNavigationPaneTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl HideHomeNavigationPaneTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "hide_home_navigation_pane".into(),
                category: "appearance".into(),
                name: "appearance.tweaks.hideHomeNavigationPane.name".into(),
                short_description: "appearance.tweaks.hideHomeNavigationPane.shortDescription"
                    .into(),
                detail_description: "appearance.tweaks.hideHomeNavigationPane.detailDescription"
                    .into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: DISABLED_VALUE.into(),
                risk: RiskLevel::None,
                risk_description: None,
                conflicts: None,
                requires_action: RequiresAction::None,
                min_os_build: Some(22621),
                min_os_ubr: None,
                min_required_memory_gb: None,
            },
        }
    }

    fn hide(&self) -> Result<(), AppError> {
        for key in HOME_NAMESPACE_KEYS {
            key.delete_subkey_tree()?;
        }

        Ok(())
    }

    fn is_enabled(&self) -> Result<bool, AppError> {
        for key in HOME_NAMESPACE_KEYS {
            if key.key_exists()? {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

impl Tweak for HideHomeNavigationPaneTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => {
                self.hide()?;
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
        HOME_NAMESPACE_KEYS[0].set_string("", HOME_NAMESPACE_DEFAULT_VALUE)?;
        refresh_shell_namespace()
    }

    fn get_status(&self) -> Result<TweakStatus, AppError> {
        Ok(TweakStatus {
            current_value: if self.is_enabled()? {
                ENABLED_VALUE.into()
            } else {
                DISABLED_VALUE.into()
            },
            is_default: !self.is_enabled()?,
        })
    }
}
