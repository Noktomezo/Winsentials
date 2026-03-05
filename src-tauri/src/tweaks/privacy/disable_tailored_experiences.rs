use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const PRIVACY_PATH: &str = r"Software\Microsoft\Windows\CurrentVersion\Privacy";
const TAILORED_EXPERIENCES: &str =
  "TailoredExperiencesWithDiagnosticDataEnabled";

pub struct DisableTailoredExperiencesTweak {
  meta: TweakMeta,
}

impl DisableTailoredExperiencesTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "disable_tailored_experiences".to_string(),
        category: TweakCategory::Privacy,
        name_key: "tweaks.disableTailoredExperiences.name".to_string(),
        description_key: "tweaks.disableTailoredExperiences.description"
          .to_string(),
        details_key: "tweaks.disableTailoredExperiences.details".to_string(),
        risk_details_key: None,
        ui_type: TweakUiType::Toggle,
        options: vec![],
        requires_reboot: false,
        requires_logout: false,
        risk_level: RiskLevel::Low,
        min_windows_build: None,
      },
    }
  }
}

impl Tweak for DisableTailoredExperiencesTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let value = registry::read_reg_u32(
      HKEY_CURRENT_USER,
      PRIVACY_PATH,
      TAILORED_EXPERIENCES,
    );
    let is_applied = value.map(|v| v == 0).unwrap_or(false);

    Ok(TweakState {
      id: self.meta.id.clone(),
      current_value: Some(if is_applied { "1" } else { "0" }.to_string()),
      is_applied,
    })
  }

  fn apply(&self, _value: Option<&str>) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_CURRENT_USER,
      PRIVACY_PATH,
      TAILORED_EXPERIENCES,
      0,
    )
    .map_err(|e| e.to_string())
  }

  fn revert(&self) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_CURRENT_USER,
      PRIVACY_PATH,
      TAILORED_EXPERIENCES,
      1,
    )
    .map_err(|e| e.to_string())
  }
}
