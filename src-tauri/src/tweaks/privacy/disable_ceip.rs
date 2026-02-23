use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use std::process::Command;
use winreg::enums::*;

const SQMCLIENT_PATH: &str = r"SOFTWARE\Microsoft\SQMClient\Windows";
const CEIP_ENABLE: &str = "CEIPEnable";

fn disable_ceip_tasks() -> Result<(), String> {
  let tasks = [
    r"\Microsoft\Windows\Customer Experience Improvement Program\Consolidator",
    r"\Microsoft\Windows\Customer Experience Improvement Program\UsbCeip",
    r"\Microsoft\Windows\Customer Experience Improvement Program\KernelCeipLive",
    r"\Microsoft\Windows\Customer Experience Improvement Program\BthSQM",
  ];
  for task in &tasks {
    let _ = Command::new("schtasks")
      .args(["/change", "/tn", task, "/disable"])
      .status();
  }
  Ok(())
}

fn enable_ceip_tasks() -> Result<(), String> {
  let tasks = [
    r"\Microsoft\Windows\Customer Experience Improvement Program\Consolidator",
    r"\Microsoft\Windows\Customer Experience Improvement Program\UsbCeip",
    r"\Microsoft\Windows\Customer Experience Improvement Program\KernelCeipLive",
    r"\Microsoft\Windows\Customer Experience Improvement Program\BthSQM",
  ];
  for task in &tasks {
    let _ = Command::new("schtasks")
      .args(["/change", "/tn", task, "/enable"])
      .status();
  }
  Ok(())
}

pub struct DisableCEIPTweak {
  meta: TweakMeta,
}

impl DisableCEIPTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "disable_ceip".to_string(),
        category: TweakCategory::Privacy,
        name_key: "tweaks.disableCEIP.name".to_string(),
        description_key: "tweaks.disableCEIP.description".to_string(),
        details_key: "tweaks.disableCEIP.details".to_string(),
        ui_type: TweakUiType::Toggle,
        options: vec![],
        requires_reboot: false,
        risk_level: RiskLevel::Low,
        min_windows_build: None,
      },
    }
  }
}

impl Tweak for DisableCEIPTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let value =
      registry::read_reg_u32(HKEY_LOCAL_MACHINE, SQMCLIENT_PATH, CEIP_ENABLE);
    let is_applied = value.map(|v| v == 0).unwrap_or(false);
    Ok(TweakState {
      id: self.meta.id.clone(),
      current_value: Some(if is_applied { "1" } else { "0" }.to_string()),
      is_applied,
    })
  }

  fn apply(&self, _value: Option<&str>) -> Result<(), String> {
    registry::write_reg_u32(HKEY_LOCAL_MACHINE, SQMCLIENT_PATH, CEIP_ENABLE, 0)
      .map_err(|e| e.to_string())?;
    disable_ceip_tasks()?;
    Ok(())
  }

  fn revert(&self) -> Result<(), String> {
    registry::write_reg_u32(HKEY_LOCAL_MACHINE, SQMCLIENT_PATH, CEIP_ENABLE, 1)
      .map_err(|e| e.to_string())?;
    enable_ceip_tasks()?;
    Ok(())
  }
}
