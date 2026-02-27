use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const BOOT_OPT_PATH: &str = r"SOFTWARE\Microsoft\Dfrg\BootOptimizeFunction";
const ENABLE_VALUE: &str = "Enable";

pub struct BootOptimizeFunctionTweak {
  meta: TweakMeta,
}

impl BootOptimizeFunctionTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "boot_optimize_function".to_string(),
        category: TweakCategory::System,
        name_key: "tweaks.bootOptimizeFunction.name".to_string(),
        description_key: "tweaks.bootOptimizeFunction.description".to_string(),
        details_key: "tweaks.bootOptimizeFunction.details".to_string(),
        ui_type: TweakUiType::Toggle,
        options: vec![],
        requires_reboot: true,
        requires_logout: false,
        risk_level: RiskLevel::Low,
        min_windows_build: None,
      },
    }
  }
}

impl Tweak for BootOptimizeFunctionTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let value = registry::read_reg_string(
      HKEY_LOCAL_MACHINE,
      BOOT_OPT_PATH,
      ENABLE_VALUE,
    );
    let is_applied = value.map(|v| v == "Y").unwrap_or(false);
    Ok(TweakState {
      id: self.meta.id.clone(),
      current_value: Some(if is_applied { "Y" } else { "N" }.to_string()),
      is_applied,
    })
  }

  fn apply(&self, _value: Option<&str>) -> Result<(), String> {
    registry::write_reg_string(
      HKEY_LOCAL_MACHINE,
      BOOT_OPT_PATH,
      ENABLE_VALUE,
      "Y",
    )
    .map_err(|e| e.to_string())
  }

  fn revert(&self) -> Result<(), String> {
    registry::write_reg_string(
      HKEY_LOCAL_MACHINE,
      BOOT_OPT_PATH,
      ENABLE_VALUE,
      "N",
    )
    .map_err(|e| e.to_string())
  }
}
