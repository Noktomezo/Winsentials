use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const MEMORY_MGMT_PATH: &str =
  r"SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management";
const FEATURE_SETTINGS: &str = "FeatureSettings";
const FEATURE_SETTINGS_OVERRIDE: &str = "FeatureSettingsOverride";
const FEATURE_SETTINGS_OVERRIDE_MASK: &str = "FeatureSettingsOverrideMask";

pub struct SpectreMeltdownTweak {
  meta: TweakMeta,
}

impl SpectreMeltdownTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "spectre_meltdown".to_string(),
        category: TweakCategory::Security,
        name_key: "tweaks.spectreMeltdown.name".to_string(),
        description_key: "tweaks.spectreMeltdown.description".to_string(),
        details_key: "tweaks.spectreMeltdown.details".to_string(),
        ui_type: TweakUiType::Toggle,
        options: vec![],
        requires_reboot: true,
        risk_level: RiskLevel::High,
        min_windows_build: None,
      },
    }
  }
}

impl Tweak for SpectreMeltdownTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let feature_settings = registry::read_reg_u32(
      HKEY_LOCAL_MACHINE,
      MEMORY_MGMT_PATH,
      FEATURE_SETTINGS,
    );
    let feature_settings_override = registry::read_reg_u32(
      HKEY_LOCAL_MACHINE,
      MEMORY_MGMT_PATH,
      FEATURE_SETTINGS_OVERRIDE,
    );
    let is_applied =
      feature_settings == Some(1) && feature_settings_override == Some(3);
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
      FEATURE_SETTINGS,
      1,
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      MEMORY_MGMT_PATH,
      FEATURE_SETTINGS_OVERRIDE,
      3,
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      MEMORY_MGMT_PATH,
      FEATURE_SETTINGS_OVERRIDE_MASK,
      3,
    )
    .map_err(|e| e.to_string())
  }

  fn revert(&self) -> Result<(), String> {
    registry::delete_reg_value(
      HKEY_LOCAL_MACHINE,
      MEMORY_MGMT_PATH,
      FEATURE_SETTINGS,
    )
    .ok();
    registry::delete_reg_value(
      HKEY_LOCAL_MACHINE,
      MEMORY_MGMT_PATH,
      FEATURE_SETTINGS_OVERRIDE,
    )
    .ok();
    registry::delete_reg_value(
      HKEY_LOCAL_MACHINE,
      MEMORY_MGMT_PATH,
      FEATURE_SETTINGS_OVERRIDE_MASK,
    )
    .ok();
    Ok(())
  }
}
