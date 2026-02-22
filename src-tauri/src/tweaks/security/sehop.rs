use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const KERNEL_PATH: &str =
  r"SYSTEM\CurrentControlSet\Control\Session Manager\kernel";
const DISABLE_EXCEPTION_CHAIN_VALIDATION: &str =
  "DisableExceptionChainValidation";

pub struct SehopTweak {
  meta: TweakMeta,
}

impl SehopTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "sehop".to_string(),
        category: TweakCategory::Security,
        name_key: "tweaks.sehop.name".to_string(),
        description_key: "tweaks.sehop.description".to_string(),
        details_key: "tweaks.sehop.details".to_string(),
        ui_type: TweakUiType::Toggle,
        options: vec![],
        requires_reboot: true,
        risk_level: RiskLevel::High,
        min_windows_build: None,
      },
    }
  }
}

impl Tweak for SehopTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let value = registry::read_reg_u32(
      HKEY_LOCAL_MACHINE,
      KERNEL_PATH,
      DISABLE_EXCEPTION_CHAIN_VALIDATION,
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
      KERNEL_PATH,
      DISABLE_EXCEPTION_CHAIN_VALIDATION,
      1,
    )
    .map_err(|e| e.to_string())
  }

  fn revert(&self) -> Result<(), String> {
    registry::delete_reg_value(
      HKEY_LOCAL_MACHINE,
      KERNEL_PATH,
      DISABLE_EXCEPTION_CHAIN_VALIDATION,
    )
    .ok();
    Ok(())
  }
}
