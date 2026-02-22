use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const CLASSIC_MENU_CLSID: &str = "{86ca1aa0-34aa-4e8b-a509-50c905bae2a2}";
const COMMAND_BAR_CLSID: &str = "{d93ed569-3b3e-4bff-8355-3c44f6a52bb5}";
const CLSID_BASE_PATH: &str = r"Software\Classes\CLSID";
const WIN11_BUILD: u32 = 22000;

pub struct ClassicContextMenuTweak {
  meta: TweakMeta,
}

impl ClassicContextMenuTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "classic_context_menu".to_string(),
        category: TweakCategory::Appearance,
        name_key: "tweaks.classicContextMenu.name".to_string(),
        description_key: "tweaks.classicContextMenu.description".to_string(),
        details_key: "tweaks.classicContextMenu.details".to_string(),
        ui_type: TweakUiType::Toggle,
        options: vec![],
        requires_reboot: false,
        risk_level: RiskLevel::Low,
        min_windows_build: Some(WIN11_BUILD),
      },
    }
  }

  fn get_classic_menu_path() -> String {
    format!(r"{}\{}\InprocServer32", CLSID_BASE_PATH, CLASSIC_MENU_CLSID)
  }

  fn get_command_bar_path() -> String {
    format!(r"{}\{}\InprocServer32", CLSID_BASE_PATH, COMMAND_BAR_CLSID)
  }

  fn get_classic_menu_key_path() -> String {
    format!(r"{}\{}", CLSID_BASE_PATH, CLASSIC_MENU_CLSID)
  }

  fn get_command_bar_key_path() -> String {
    format!(r"{}\{}", CLSID_BASE_PATH, COMMAND_BAR_CLSID)
  }
}

impl Tweak for ClassicContextMenuTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let classic_path = Self::get_classic_menu_path();
    let value = registry::read_reg_string(HKEY_CURRENT_USER, &classic_path, "");

    let is_applied = value.is_some();

    Ok(TweakState {
      id: self.meta.id.clone(),
      current_value: Some(if is_applied { "1" } else { "0" }.to_string()),
      is_applied,
    })
  }

  fn apply(&self, _value: Option<&str>) -> Result<(), String> {
    let classic_path = Self::get_classic_menu_path();
    let command_bar_path = Self::get_command_bar_path();

    registry::write_reg_string(HKEY_CURRENT_USER, &classic_path, "", "")
      .map_err(|e| format!("Failed to create classic menu key: {}", e))?;

    registry::write_reg_string(HKEY_CURRENT_USER, &command_bar_path, "", "")
      .map_err(|e| format!("Failed to create command bar key: {}", e))?;

    registry::restart_explorer();

    Ok(())
  }

  fn revert(&self) -> Result<(), String> {
    let classic_key_path = Self::get_classic_menu_key_path();
    let command_bar_key_path = Self::get_command_bar_key_path();

    let _ = registry::delete_reg_key(HKEY_CURRENT_USER, &classic_key_path);
    let _ = registry::delete_reg_key(HKEY_CURRENT_USER, &command_bar_key_path);

    registry::restart_explorer();

    Ok(())
  }
}
