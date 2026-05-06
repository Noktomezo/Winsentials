use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";
const CUSTOM_VALUE: &str = "custom";
const EDGE_EXTENSION_BLOCKLIST_ENTRY: &str = "1000";

enum RegistryValue {
    Dword(u32),
    String(&'static str),
}

struct PolicyValue {
    key: RegKey,
    name: &'static str,
    value: RegistryValue,
}

pub struct BrowserPolicyDebloatTweak {
    policies: &'static [PolicyValue],
    meta: TweakMeta,
}

impl BrowserPolicyDebloatTweak {
    pub fn new_edge() -> Self {
        Self::new(
            "microsoft_edge_debloat",
            "debloat.tweaks.microsoftEdgeDebloat.name",
            "debloat.tweaks.microsoftEdgeDebloat.shortDescription",
            "debloat.tweaks.microsoftEdgeDebloat.detailDescription",
            "debloat.tweaks.microsoftEdgeDebloat.riskDescription",
            "Microsoft Edge",
            EDGE_POLICIES,
        )
    }

    pub fn new_brave() -> Self {
        Self::new(
            "brave_browser_debloat",
            "debloat.tweaks.braveBrowserDebloat.name",
            "debloat.tweaks.braveBrowserDebloat.shortDescription",
            "debloat.tweaks.braveBrowserDebloat.detailDescription",
            "debloat.tweaks.braveBrowserDebloat.riskDescription",
            "Brave",
            BRAVE_POLICIES,
        )
    }

    fn new(
        id: &'static str,
        name: &'static str,
        short_description: &'static str,
        detail_description: &'static str,
        risk_description: &'static str,
        app_name: &'static str,
        policies: &'static [PolicyValue],
    ) -> Self {
        Self {
            policies,
            meta: TweakMeta {
                id: id.into(),
                category: "debloat".into(),
                name: name.into(),
                short_description: short_description.into(),
                detail_description: detail_description.into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: ENABLED_VALUE.into(),
                risk: RiskLevel::Low,
                risk_description: Some(risk_description.into()),
                conflicts: None,
                requires_action: RequiresAction::RestartApp {
                    app_name: app_name.into(),
                },
                min_os_build: Some(10240),
                min_os_ubr: None,
                min_required_memory_gb: None,
            },
        }
    }

    fn write_enabled_values(&self) -> Result<(), AppError> {
        for policy in self.policies {
            match policy.value {
                RegistryValue::Dword(value) => policy.key.set_dword(policy.name, value)?,
                RegistryValue::String(value) => policy.key.set_string(policy.name, value)?,
            }
        }

        Ok(())
    }

    fn delete_policy_values(&self) -> Result<(), AppError> {
        for policy in self.policies {
            policy.key.delete_value(policy.name)?;
        }

        Ok(())
    }

    fn policy_matches(policy: &PolicyValue) -> Result<bool, AppError> {
        match policy.value {
            RegistryValue::Dword(expected) => match policy.key.get_dword(policy.name) {
                Ok(value) => Ok(value == expected),
                Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
                    Ok(false)
                }
                Err(error) => Err(error),
            },
            RegistryValue::String(expected) => match policy.key.get_string(policy.name) {
                Ok(value) => Ok(value == expected),
                Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
                    Ok(false)
                }
                Err(error) => Err(error),
            },
        }
    }

    fn policy_missing(policy: &PolicyValue) -> Result<bool, AppError> {
        match policy.value {
            RegistryValue::Dword(_) => match policy.key.get_dword(policy.name) {
                Ok(_) => Ok(false),
                Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
                    Ok(true)
                }
                Err(error) => Err(error),
            },
            RegistryValue::String(_) => match policy.key.get_string(policy.name) {
                Ok(_) => Ok(false),
                Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => {
                    Ok(true)
                }
                Err(error) => Err(error),
            },
        }
    }

    fn is_enabled(&self) -> Result<bool, AppError> {
        for policy in self.policies {
            if !Self::policy_matches(policy)? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn is_default(&self) -> Result<bool, AppError> {
        for policy in self.policies {
            if !Self::policy_missing(policy)? {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

impl Tweak for BrowserPolicyDebloatTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => self.write_enabled_values(),
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        self.delete_policy_values()
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

const EDGE_UPDATE_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SOFTWARE\Policies\Microsoft\EdgeUpdate",
};

const EDGE_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SOFTWARE\Policies\Microsoft\Edge",
};

const EDGE_EXTENSION_BLOCKLIST_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SOFTWARE\Policies\Microsoft\Edge\ExtensionInstallBlocklist",
};

const BRAVE_KEY: RegKey = RegKey {
    hive: Hive::LocalMachine,
    path: r"SOFTWARE\Policies\BraveSoftware\Brave",
};

const EDGE_POLICIES: &[PolicyValue] = &[
    PolicyValue {
        key: EDGE_UPDATE_KEY,
        name: "CreateDesktopShortcutDefault",
        value: RegistryValue::Dword(0),
    },
    PolicyValue {
        key: EDGE_KEY,
        name: "PersonalizationReportingEnabled",
        value: RegistryValue::Dword(0),
    },
    PolicyValue {
        key: EDGE_EXTENSION_BLOCKLIST_KEY,
        // Use a high policy index to avoid clobbering user-owned low numeric entries.
        name: EDGE_EXTENSION_BLOCKLIST_ENTRY,
        value: RegistryValue::String("ofefcgjbeghpigppfmkologfjadafddi"),
    },
    PolicyValue {
        key: EDGE_KEY,
        name: "ShowRecommendationsEnabled",
        value: RegistryValue::Dword(0),
    },
    PolicyValue {
        key: EDGE_KEY,
        name: "HideFirstRunExperience",
        value: RegistryValue::Dword(1),
    },
    PolicyValue {
        key: EDGE_KEY,
        name: "UserFeedbackAllowed",
        value: RegistryValue::Dword(0),
    },
    PolicyValue {
        key: EDGE_KEY,
        name: "ConfigureDoNotTrack",
        value: RegistryValue::Dword(1),
    },
    PolicyValue {
        key: EDGE_KEY,
        name: "AlternateErrorPagesEnabled",
        value: RegistryValue::Dword(0),
    },
    PolicyValue {
        key: EDGE_KEY,
        name: "EdgeCollectionsEnabled",
        value: RegistryValue::Dword(0),
    },
    PolicyValue {
        key: EDGE_KEY,
        name: "EdgeShoppingAssistantEnabled",
        value: RegistryValue::Dword(0),
    },
    PolicyValue {
        key: EDGE_KEY,
        name: "MicrosoftEdgeInsiderPromotionEnabled",
        value: RegistryValue::Dword(0),
    },
    PolicyValue {
        key: EDGE_KEY,
        name: "ShowMicrosoftRewards",
        value: RegistryValue::Dword(0),
    },
    PolicyValue {
        key: EDGE_KEY,
        name: "WebWidgetAllowed",
        value: RegistryValue::Dword(0),
    },
    PolicyValue {
        key: EDGE_KEY,
        name: "DiagnosticData",
        value: RegistryValue::Dword(0),
    },
    PolicyValue {
        key: EDGE_KEY,
        name: "EdgeAssetDeliveryServiceEnabled",
        value: RegistryValue::Dword(0),
    },
    PolicyValue {
        key: EDGE_KEY,
        name: "WalletDonationEnabled",
        value: RegistryValue::Dword(0),
    },
    PolicyValue {
        key: EDGE_KEY,
        name: "DefaultBrowserSettingsCampaignEnabled",
        value: RegistryValue::Dword(0),
    },
];

const BRAVE_POLICIES: &[PolicyValue] = &[
    PolicyValue {
        key: BRAVE_KEY,
        name: "BraveRewardsDisabled",
        value: RegistryValue::Dword(1),
    },
    PolicyValue {
        key: BRAVE_KEY,
        name: "BraveWalletDisabled",
        value: RegistryValue::Dword(1),
    },
    PolicyValue {
        key: BRAVE_KEY,
        name: "BraveVPNDisabled",
        value: RegistryValue::Dword(1),
    },
    PolicyValue {
        key: BRAVE_KEY,
        name: "BraveAIChatEnabled",
        value: RegistryValue::Dword(0),
    },
    PolicyValue {
        key: BRAVE_KEY,
        name: "BraveStatsPingEnabled",
        value: RegistryValue::Dword(0),
    },
    PolicyValue {
        key: BRAVE_KEY,
        name: "BraveNewsDisabled",
        value: RegistryValue::Dword(1),
    },
    PolicyValue {
        key: BRAVE_KEY,
        name: "BraveTalkDisabled",
        value: RegistryValue::Dword(1),
    },
    PolicyValue {
        key: BRAVE_KEY,
        name: "TorDisabled",
        value: RegistryValue::Dword(1),
    },
    PolicyValue {
        key: BRAVE_KEY,
        name: "BraveP3AEnabled",
        value: RegistryValue::Dword(0),
    },
    PolicyValue {
        key: BRAVE_KEY,
        name: "UrlKeyedAnonymizedDataCollectionEnabled",
        value: RegistryValue::Dword(0),
    },
    PolicyValue {
        key: BRAVE_KEY,
        name: "SafeBrowsingExtendedReportingEnabled",
        value: RegistryValue::Dword(0),
    },
    PolicyValue {
        key: BRAVE_KEY,
        name: "MetricsReportingEnabled",
        value: RegistryValue::Dword(0),
    },
];
