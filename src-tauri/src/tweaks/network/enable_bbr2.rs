use crate::error::AppError;
use crate::shell::run_netsh;
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";

// (template, default_provider) — Compat uses NewReno, everything else CUBIC.
const TEMPLATES: &[(&str, &str)] = &[
    ("Internet", "CUBIC"),
    ("InternetCustom", "CUBIC"),
    ("Datacenter", "CUBIC"),
    ("DatacenterCustom", "CUBIC"),
    ("Compat", "NewReno"),
];

pub struct EnableBbr2Tweak {
    meta: TweakMeta,
}

impl Default for EnableBbr2Tweak {
    fn default() -> Self {
        Self::new()
    }
}

impl EnableBbr2Tweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "enable_bbr2_congestion_control".into(),
                category: "network".into(),
                name: "network.tweaks.enableBbr2.name".into(),
                short_description: "network.tweaks.enableBbr2.shortDescription".into(),
                detail_description: "network.tweaks.enableBbr2.detailDescription".into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: ENABLED_VALUE.into(),
                risk: RiskLevel::Low,
                risk_description: Some("network.tweaks.enableBbr2.riskDescription".into()),
                requires_action: RequiresAction::RestartPc,
                min_os_build: Some(19041),
                min_os_ubr: None,
            },
        }
    }

    fn set_templates(provider_override: Option<&str>, loopback_mtu: &str) -> Result<(), AppError> {
        for (t, default) in TEMPLATES {
            let provider = provider_override.unwrap_or(default);
            run_netsh(&[
                "int",
                "tcp",
                "set",
                "supplemental",
                &format!("template={t}"),
                &format!("congestionprovider={provider}"),
            ])?;
        }
        run_netsh(&[
            "int",
            "ipv4",
            "set",
            "global",
            &format!("loopbacklargemtu={loopback_mtu}"),
        ])?;
        run_netsh(&[
            "int",
            "ipv6",
            "set",
            "global",
            &format!("loopbacklargemtu={loopback_mtu}"),
        ])?;
        Ok(())
    }
}

impl Tweak for EnableBbr2Tweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            // loopbacklargemtu must be disabled — incompatible with BBR2.
            ENABLED_VALUE => Self::set_templates(Some("BBR2"), "disable"),
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        // None → each template gets its own default provider from TEMPLATES.
        Self::set_templates(None, "enable")
    }

    fn get_status(&self) -> Result<TweakStatus, AppError> {
        // Checking the Internet template is a fast proxy for the whole set —
        // we always apply/reset all templates together so they stay in sync.
        // "bbr2" / "BBR2" — case varies by locale; normalise before check.
        let output = run_netsh(&["int", "tcp", "show", "supplemental", "template=Internet"])?;
        let enabled = output.to_ascii_uppercase().contains("BBR2");
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
