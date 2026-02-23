use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const PERSONALIZATION_PATH: &str =
  r"Software\Microsoft\Personalization\Settings";
const ACCEPTED_PRIVACY_POLICY: &str = "AcceptedPrivacyPolicy";
const TABLETPC_PATH: &str = r"SOFTWARE\Policies\Microsoft\Windows\TabletPC";
const PREVENT_HANDWRITING_SHARING: &str = "PreventHandwritingDataSharing";

pub struct DisableHandwritingTweak {
  meta: TweakMeta,
}

impl DisableHandwritingTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "disable_handwriting".to_string(),
        category: TweakCategory::Privacy,
        name_key: "tweaks.disableHandwriting.name".to_string(),
        description_key: "tweaks.disableHandwriting.description".to_string(),
        details_key: "tweaks.disableHandwriting.details".to_string(),
        ui_type: TweakUiType::Toggle,
        options: vec![],
        requires_reboot: false,
        risk_level: RiskLevel::Low,
        min_windows_build: None,
      },
    }
  }
}

impl Tweak for DisableHandwritingTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let privacy = registry::read_reg_u32(
      HKEY_CURRENT_USER,
      PERSONALIZATION_PATH,
      ACCEPTED_PRIVACY_POLICY,
    );
    let handwriting = registry::read_reg_u32(
      HKEY_LOCAL_MACHINE,
      TABLETPC_PATH,
      PREVENT_HANDWRITING_SHARING,
    );
    let is_applied = privacy.map(|v| v == 0).unwrap_or(false)
      && handwriting.map(|v| v == 1).unwrap_or(false);

    Ok(TweakState {
      id: self.meta.id.clone(),
      current_value: Some(if is_applied { "1" } else { "0" }.to_string()),
      is_applied,
    })
  }

  fn apply(&self, _value: Option<&str>) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      TABLETPC_PATH,
      PREVENT_HANDWRITING_SHARING,
      1,
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_u32(
      HKEY_CURRENT_USER,
      PERSONALIZATION_PATH,
      ACCEPTED_PRIVACY_POLICY,
      0,
    )
    .map_err(|e| e.to_string())?;
    Ok(())
  }

  fn revert(&self) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      TABLETPC_PATH,
      PREVENT_HANDWRITING_SHARING,
      0,
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_u32(
      HKEY_CURRENT_USER,
      PERSONALIZATION_PATH,
      ACCEPTED_PRIVACY_POLICY,
      1,
    )
    .map_err(|e| e.to_string())?;
    Ok(())
  }
}
