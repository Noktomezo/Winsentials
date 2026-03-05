use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakOption, TweakState,
  TweakUiType,
};
use winreg::enums::*;

const PREFETCH_PATH: &str = r"SYSTEM\CurrentControlSet\Control\Session Manager\Memory Management\PrefetchParameters";
const ENABLE_PREFETCHER: &str = "EnablePrefetcher";

pub struct PrefetcherTweak {
  meta: TweakMeta,
}

impl PrefetcherTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "prefetcher".to_string(),
        category: TweakCategory::System,
        name_key: "tweaks.prefetcher.name".to_string(),
        description_key: "tweaks.prefetcher.description".to_string(),
        details_key: "tweaks.prefetcher.details".to_string(),
        risk_details_key: None,
        ui_type: TweakUiType::Radio,
        options: vec![
          TweakOption {
            value: "0".to_string(),
            label_key: "tweaks.prefetcher.options.disabled".to_string(),
            is_default: false,
            is_recommended: false,
          },
          TweakOption {
            value: "1".to_string(),
            label_key: "tweaks.prefetcher.options.appOnly".to_string(),
            is_default: false,
            is_recommended: false,
          },
          TweakOption {
            value: "2".to_string(),
            label_key: "tweaks.prefetcher.options.bootOnly".to_string(),
            is_default: false,
            is_recommended: false,
          },
          TweakOption {
            value: "3".to_string(),
            label_key: "tweaks.prefetcher.options.both".to_string(),
            is_default: true,
            is_recommended: true,
          },
        ],
        requires_reboot: true,
        requires_logout: false,
        risk_level: RiskLevel::Low,
        min_windows_build: None,
      },
    }
  }
}

impl Tweak for PrefetcherTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let value = registry::read_reg_u32(
      HKEY_LOCAL_MACHINE,
      PREFETCH_PATH,
      ENABLE_PREFETCHER,
    )
    .map(|v| v.to_string());
    let is_applied = value.as_ref().map(|v| v != "3").unwrap_or(false);
    Ok(TweakState {
      id: self.meta.id.clone(),
      current_value: value,
      is_applied,
    })
  }

  fn apply(&self, value: Option<&str>) -> Result<(), String> {
    let val = value.and_then(|v| v.parse::<u32>().ok()).unwrap_or(3);
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      PREFETCH_PATH,
      ENABLE_PREFETCHER,
      val,
    )
    .map_err(|e| e.to_string())
  }

  fn revert(&self) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      PREFETCH_PATH,
      ENABLE_PREFETCHER,
      3,
    )
    .map_err(|e| e.to_string())
  }
}
