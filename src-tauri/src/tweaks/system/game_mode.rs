use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const GAMEBAR_PATH: &str = r"Software\Microsoft\GameBar";
const AUTO_GAME_MODE: &str = "AutoGameModeEnabled";
const ALLOW_AUTO_GAME_MODE: &str = "AllowAutoGameMode";

pub struct GameModeTweak {
  meta: TweakMeta,
}

impl GameModeTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "game_mode".to_string(),
        category: TweakCategory::System,
        name_key: "tweaks.gameMode.name".to_string(),
        description_key: "tweaks.gameMode.description".to_string(),
        details_key: "tweaks.gameMode.details".to_string(),
        ui_type: TweakUiType::Toggle,
        options: vec![],
        requires_reboot: false,
        risk_level: RiskLevel::Low,
        min_windows_build: None,
      },
    }
  }
}

impl Tweak for GameModeTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let value =
      registry::read_reg_u32(HKEY_CURRENT_USER, GAMEBAR_PATH, AUTO_GAME_MODE);
    let is_applied = value.map(|v| v == 1).unwrap_or(false);
    Ok(TweakState {
      id: self.meta.id.clone(),
      current_value: Some(if is_applied { "1" } else { "0" }.to_string()),
      is_applied,
    })
  }

  fn apply(&self, _value: Option<&str>) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_CURRENT_USER,
      GAMEBAR_PATH,
      ALLOW_AUTO_GAME_MODE,
      1,
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_u32(HKEY_CURRENT_USER, GAMEBAR_PATH, AUTO_GAME_MODE, 1)
      .map_err(|e| e.to_string())
  }

  fn revert(&self) -> Result<(), String> {
    registry::delete_reg_value(
      HKEY_CURRENT_USER,
      GAMEBAR_PATH,
      ALLOW_AUTO_GAME_MODE,
    )
    .ok();
    registry::delete_reg_value(HKEY_CURRENT_USER, GAMEBAR_PATH, AUTO_GAME_MODE)
      .ok();
    Ok(())
  }
}
