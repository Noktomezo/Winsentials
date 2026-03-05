use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const MEMORY_MGMT_PATH: &str =
  r"SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management";
const DISABLE_PAGE_COMBINING: &str = "DisablePageCombining";

pub struct DisablePageCombiningTweak {
  meta: TweakMeta,
}

impl DisablePageCombiningTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "disable_page_combining".to_string(),
        category: TweakCategory::Memory,
        name_key: "tweaks.disablePageCombining.name".to_string(),
        description_key: "tweaks.disablePageCombining.description".to_string(),
        details_key: "tweaks.disablePageCombining.details".to_string(),
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

impl Tweak for DisablePageCombiningTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let value = registry::read_reg_u32(
      HKEY_LOCAL_MACHINE,
      MEMORY_MGMT_PATH,
      DISABLE_PAGE_COMBINING,
    );
    let is_applied = value.map(|v| v == 1).unwrap_or(false);
    Ok(TweakState {
      id: self.meta.id.clone(),
      current_value: Some(if is_applied { "1" } else { "0" }.to_string()),
      is_applied,
    })
  }

  fn apply(&self, _value: Option<&str>) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      MEMORY_MGMT_PATH,
      DISABLE_PAGE_COMBINING,
      1,
    )
    .map_err(|e| e.to_string())
  }

  fn revert(&self) -> Result<(), String> {
    registry::delete_reg_value(
      HKEY_LOCAL_MACHINE,
      MEMORY_MGMT_PATH,
      DISABLE_PAGE_COMBINING,
    )
    .ok();
    Ok(())
  }
}
