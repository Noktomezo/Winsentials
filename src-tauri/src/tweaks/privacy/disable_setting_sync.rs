use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const SETTING_SYNC_PATH: &str =
  r"SOFTWARE\Policies\Microsoft\Windows\SettingSync";
const DISABLE_SETTING_SYNC: &str = "DisableSettingSync";
const DISABLE_SETTING_SYNC_OVERRIDE: &str = "DisableSettingSyncUserOverride";

pub struct DisableSettingSyncTweak {
  meta: TweakMeta,
}

impl DisableSettingSyncTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "disable_setting_sync".to_string(),
        category: TweakCategory::Privacy,
        name_key: "tweaks.disableSettingSync.name".to_string(),
        description_key: "tweaks.disableSettingSync.description".to_string(),
        details_key: "tweaks.disableSettingSync.details".to_string(),
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

impl Tweak for DisableSettingSyncTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let value = registry::read_reg_u32(
      HKEY_LOCAL_MACHINE,
      SETTING_SYNC_PATH,
      DISABLE_SETTING_SYNC,
    );
    let is_applied = value.map(|v| v == 2).unwrap_or(false);
    Ok(TweakState {
      id: self.meta.id.clone(),
      current_value: Some(if is_applied { "1" } else { "0" }.to_string()),
      is_applied,
    })
  }

  fn apply(&self, _value: Option<&str>) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      SETTING_SYNC_PATH,
      DISABLE_SETTING_SYNC,
      2,
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      SETTING_SYNC_PATH,
      DISABLE_SETTING_SYNC_OVERRIDE,
      1,
    )
    .map_err(|e| e.to_string())
  }

  fn revert(&self) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      SETTING_SYNC_PATH,
      DISABLE_SETTING_SYNC,
      0,
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      SETTING_SYNC_PATH,
      DISABLE_SETTING_SYNC_OVERRIDE,
      0,
    )
    .map_err(|e| e.to_string())
  }
}
