use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const WINDOWS_SEARCH_PATH: &str =
  r"SOFTWARE\Policies\Microsoft\Windows\Windows Search";
const DISABLE_WEB_SEARCH: &str = "DisableWebSearch";
const CONNECTED_SEARCH_USE_WEB: &str = "ConnectedSearchUseWeb";

pub struct DisableCortanaTweak {
  meta: TweakMeta,
}

impl DisableCortanaTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "disable_cortana".to_string(),
        category: TweakCategory::Privacy,
        name_key: "tweaks.disableCortana.name".to_string(),
        description_key: "tweaks.disableCortana.description".to_string(),
        details_key: "tweaks.disableCortana.details".to_string(),
        ui_type: TweakUiType::Toggle,
        options: vec![],
        requires_reboot: false,
        risk_level: RiskLevel::Low,
        min_windows_build: None,
      },
    }
  }
}

impl Tweak for DisableCortanaTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let web_search = registry::read_reg_u32(
      HKEY_LOCAL_MACHINE,
      WINDOWS_SEARCH_PATH,
      DISABLE_WEB_SEARCH,
    );
    let is_applied = web_search.map(|v| v == 1).unwrap_or(false);
    Ok(TweakState {
      id: self.meta.id.clone(),
      current_value: Some(if is_applied { "1" } else { "0" }.to_string()),
      is_applied,
    })
  }

  fn apply(&self, _value: Option<&str>) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      WINDOWS_SEARCH_PATH,
      DISABLE_WEB_SEARCH,
      1,
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      WINDOWS_SEARCH_PATH,
      CONNECTED_SEARCH_USE_WEB,
      0,
    )
    .map_err(|e| e.to_string())
  }

  fn revert(&self) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      WINDOWS_SEARCH_PATH,
      DISABLE_WEB_SEARCH,
      0,
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      WINDOWS_SEARCH_PATH,
      CONNECTED_SEARCH_USE_WEB,
      1,
    )
    .map_err(|e| e.to_string())
  }
}
