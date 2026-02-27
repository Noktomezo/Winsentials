use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use winreg::enums::*;

const CSRSS_PERF_OPTIONS_PATH: &str = r"SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\csrss.exe\PerfOptions";
const CPU_PRIORITY_CLASS: &str = "CpuPriorityClass";
const IO_PRIORITY: &str = "IoPriority";

pub struct CsrssPriorityTweak {
  meta: TweakMeta,
}

impl CsrssPriorityTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "csrss_priority".to_string(),
        category: TweakCategory::Input,
        name_key: "tweaks.cssrssPriority.name".to_string(),
        description_key: "tweaks.cssrssPriority.description".to_string(),
        details_key: "tweaks.cssrssPriority.details".to_string(),
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

impl Tweak for CsrssPriorityTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let cpu_priority = registry::read_reg_u32(
      HKEY_LOCAL_MACHINE,
      CSRSS_PERF_OPTIONS_PATH,
      CPU_PRIORITY_CLASS,
    );
    let is_applied = cpu_priority.map(|v| v == 4).unwrap_or(false);
    Ok(TweakState {
      id: self.meta.id.clone(),
      current_value: Some(if is_applied { "1" } else { "0" }.to_string()),
      is_applied,
    })
  }

  fn apply(&self, _value: Option<&str>) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      CSRSS_PERF_OPTIONS_PATH,
      CPU_PRIORITY_CLASS,
      4,
    )
    .map_err(|e| e.to_string())?;
    registry::write_reg_u32(
      HKEY_LOCAL_MACHINE,
      CSRSS_PERF_OPTIONS_PATH,
      IO_PRIORITY,
      3,
    )
    .map_err(|e| e.to_string())
  }

  fn revert(&self) -> Result<(), String> {
    registry::delete_reg_value(
      HKEY_LOCAL_MACHINE,
      CSRSS_PERF_OPTIONS_PATH,
      CPU_PRIORITY_CLASS,
    )
    .ok();
    registry::delete_reg_value(
      HKEY_LOCAL_MACHINE,
      CSRSS_PERF_OPTIONS_PATH,
      IO_PRIORITY,
    )
    .ok();
    Ok(())
  }
}
