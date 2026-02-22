use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const GAME_CONFIG_PATH: &str = r"System\GameConfigStore";
const DVR_ENABLED: &str = "GameDVR_Enabled";
const DVR_FSE_BEHAVIOR_MODE: &str = "GameDVR_FSEBehaviorMode";
const DVR_FSE_BEHAVIOR: &str = "GameDVR_FSEBehavior";
const DVR_HONOR_USER_FSE: &str = "GameDVR_HonorUserFSEBehaviorMode";
const DVR_DXGI_HONOR_FSE: &str = "GameDVR_DXGIHonorFSEWindowsCompatible";
const DVR_EFSE_FEATURE_FLAGS: &str = "GameDVR_EFSEFeatureFlags";
const DVR_DSE_BEHAVIOR: &str = "GameDVR_DSEBehavior";

pub struct DisableFullsreenOptimizationsTweak {
  meta: TweakMeta,
}

impl DisableFullsreenOptimizationsTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "disable_fso".to_string(),
        category: TweakCategory::System,
        name_key: "tweaks.disableFso.name".to_string(),
        description_key: "tweaks.disableFso.description".to_string(),
        details_key: "tweaks.disableFso.details".to_string(),
        ui_type: TweakUiType::Toggle,
        options: vec![],
        requires_reboot: false,
        risk_level: RiskLevel::Low,
        min_windows_build: None,
      },
    }
  }
}

impl Tweak for DisableFullsreenOptimizationsTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let value = registry::read_reg_u32(
      HKEY_CURRENT_USER,
      GAME_CONFIG_PATH,
      DVR_FSE_BEHAVIOR_MODE,
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
      HKEY_CURRENT_USER,
      GAME_CONFIG_PATH,
      DVR_ENABLED,
      0,
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_u32(
      HKEY_CURRENT_USER,
      GAME_CONFIG_PATH,
      DVR_FSE_BEHAVIOR_MODE,
      2,
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_u32(
      HKEY_CURRENT_USER,
      GAME_CONFIG_PATH,
      DVR_FSE_BEHAVIOR,
      2,
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_u32(
      HKEY_CURRENT_USER,
      GAME_CONFIG_PATH,
      DVR_HONOR_USER_FSE,
      1,
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_u32(
      HKEY_CURRENT_USER,
      GAME_CONFIG_PATH,
      DVR_DXGI_HONOR_FSE,
      1,
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_u32(
      HKEY_CURRENT_USER,
      GAME_CONFIG_PATH,
      DVR_EFSE_FEATURE_FLAGS,
      0,
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_u32(
      HKEY_CURRENT_USER,
      GAME_CONFIG_PATH,
      DVR_DSE_BEHAVIOR,
      2,
    )
    .map_err(|e| e.to_string())
  }

  fn revert(&self) -> Result<(), String> {
    registry::delete_reg_value(
      HKEY_CURRENT_USER,
      GAME_CONFIG_PATH,
      DVR_ENABLED,
    )
    .ok();
    registry::delete_reg_value(
      HKEY_CURRENT_USER,
      GAME_CONFIG_PATH,
      DVR_FSE_BEHAVIOR_MODE,
    )
    .ok();
    registry::delete_reg_value(
      HKEY_CURRENT_USER,
      GAME_CONFIG_PATH,
      DVR_FSE_BEHAVIOR,
    )
    .ok();
    registry::delete_reg_value(
      HKEY_CURRENT_USER,
      GAME_CONFIG_PATH,
      DVR_HONOR_USER_FSE,
    )
    .ok();
    registry::delete_reg_value(
      HKEY_CURRENT_USER,
      GAME_CONFIG_PATH,
      DVR_DXGI_HONOR_FSE,
    )
    .ok();
    registry::delete_reg_value(
      HKEY_CURRENT_USER,
      GAME_CONFIG_PATH,
      DVR_EFSE_FEATURE_FLAGS,
    )
    .ok();
    registry::delete_reg_value(
      HKEY_CURRENT_USER,
      GAME_CONFIG_PATH,
      DVR_DSE_BEHAVIOR,
    )
    .ok();
    Ok(())
  }
}
