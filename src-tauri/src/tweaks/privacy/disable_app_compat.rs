use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const APPCOMPAT_PATH: &str = r"SOFTWARE\Policies\Microsoft\Windows\AppCompat";
const DISABLE_INVENTORY: &str = "DisableInventory";
const DISABLE_PCA: &str = "DisablePCA";

pub struct DisableAppCompatTweak {
  meta: TweakMeta,
}

impl DisableAppCompatTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "disable_app_compat".to_string(),
        category: TweakCategory::Privacy,
        name_key: "tweaks.disableAppCompat.name".to_string(),
        description_key: "tweaks.disableAppCompat.description".to_string(),
        details_key: "tweaks.disableAppCompat.details".to_string(),
        ui_type: TweakUiType::Toggle,
        options: vec![],
        requires_reboot: false,
        risk_level: RiskLevel::Low,
        min_windows_build: None,
      },
    }
  }
}

impl Tweak for DisableAppCompatTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let inventory = registry::read_reg_u32(
      HKEY_LOCAL_MACHINE,
      APPCOMPAT_PATH,
      DISABLE_INVENTORY,
    );
    let pca =
      registry::read_reg_u32(HKEY_LOCAL_MACHINE, APPCOMPAT_PATH, DISABLE_PCA);
    let is_applied = inventory.map(|v| v == 1).unwrap_or(false)
      && pca.map(|v| v == 1).unwrap_or(false);
    Ok(TweakState {
      id: self.meta.id.clone(),
      current_value: Some(if is_applied { "1" } else { "0" }.to_string()),
      is_applied,
    })
  }

  fn apply(&self, _value: Option<&str>) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      APPCOMPAT_PATH,
      DISABLE_INVENTORY,
      1,
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_u32(HKEY_LOCAL_MACHINE, APPCOMPAT_PATH, DISABLE_PCA, 1)
      .map_err(|e| e.to_string())
  }

  fn revert(&self) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      APPCOMPAT_PATH,
      DISABLE_INVENTORY,
      0,
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_u32(HKEY_LOCAL_MACHINE, APPCOMPAT_PATH, DISABLE_PCA, 0)
      .map_err(|e| e.to_string())
  }
}
