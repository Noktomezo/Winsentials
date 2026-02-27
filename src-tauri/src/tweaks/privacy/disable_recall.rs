use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const WINDOWS_AI_PATH: &str = r"SOFTWARE\Policies\Microsoft\Windows\WindowsAI";
const ALLOW_RECALL: &str = "AllowRecallEnablement";
const WIN11_24H2_BUILD: u32 = 26100;

pub struct DisableRecallTweak {
  meta: TweakMeta,
}

impl DisableRecallTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "disable_recall".to_string(),
        category: TweakCategory::Privacy,
        name_key: "tweaks.disableRecall.name".to_string(),
        description_key: "tweaks.disableRecall.description".to_string(),
        details_key: "tweaks.disableRecall.details".to_string(),
        ui_type: TweakUiType::Toggle,
        options: vec![],
        requires_reboot: false,
        requires_logout: false,
        risk_level: RiskLevel::Low,
        min_windows_build: Some(WIN11_24H2_BUILD),
      },
    }
  }
}

impl Tweak for DisableRecallTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let value =
      registry::read_reg_u32(HKEY_LOCAL_MACHINE, WINDOWS_AI_PATH, ALLOW_RECALL);
    let is_applied = value.map(|v| v == 0).unwrap_or(false);

    Ok(TweakState {
      id: self.meta.id.clone(),
      current_value: Some(if is_applied { "1" } else { "0" }.to_string()),
      is_applied,
    })
  }

  fn apply(&self, _value: Option<&str>) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      WINDOWS_AI_PATH,
      ALLOW_RECALL,
      0,
    )
    .map_err(|e| e.to_string())
  }

  fn revert(&self) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      WINDOWS_AI_PATH,
      ALLOW_RECALL,
      1,
    )
    .map_err(|e| e.to_string())
  }
}
