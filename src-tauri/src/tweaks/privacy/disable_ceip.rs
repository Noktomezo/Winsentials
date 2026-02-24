use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use std::process::Command;
use winreg::enums::*;

const SQMCLIENT_PATH: &str = r"SOFTWARE\Microsoft\SQMClient\Windows";
const CEIP_ENABLE: &str = "CEIPEnable";

const CEIP_TASKS: [&str; 4] = [
  r"\Microsoft\Windows\Customer Experience Improvement Program\Consolidator",
  r"\Microsoft\Windows\Customer Experience Improvement Program\UsbCeip",
  r"\Microsoft\Windows\Customer Experience Improvement Program\KernelCeipLive",
  r"\Microsoft\Windows\Customer Experience Improvement Program\BthSQM",
];

fn task_exists(task: &str) -> bool {
  Command::new("schtasks")
    .args(["/query", "/tn", task])
    .output()
    .map(|o| o.status.success())
    .unwrap_or(false)
}

fn check_task_disabled(task: &str) -> bool {
  let output = match Command::new("schtasks")
    .args(["/query", "/tn", task, "/xml"])
    .output()
  {
    Ok(o) => o,
    Err(_) => return true,
  };

  if !output.status.success() {
    return true;
  }

  let stdout = String::from_utf8_lossy(&output.stdout);
  stdout.contains("<Enabled>false</Enabled>")
}

fn disable_ceip_tasks() -> Result<(), String> {
  for task in &CEIP_TASKS {
    if !task_exists(task) {
      continue;
    }

    let output = Command::new("schtasks")
      .args(["/change", "/tn", task, "/disable"])
      .output()
      .map_err(|e| format!("Failed to spawn schtasks: {}", e))?;

    if !output.status.success() {
      let stderr = String::from_utf8_lossy(&output.stderr);
      return Err(format!(
        "Failed to disable task '{}': {}",
        task,
        stderr.trim()
      ));
    }
  }
  Ok(())
}

fn enable_ceip_tasks() -> Result<(), String> {
  for task in &CEIP_TASKS {
    if !task_exists(task) {
      continue;
    }

    let output = Command::new("schtasks")
      .args(["/change", "/tn", task, "/enable"])
      .output()
      .map_err(|e| format!("Failed to spawn schtasks: {}", e))?;

    if !output.status.success() {
      let stderr = String::from_utf8_lossy(&output.stderr);
      return Err(format!(
        "Failed to enable task '{}': {}",
        task,
        stderr.trim()
      ));
    }
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
    let registry_disabled = value.map(|v| v == 0).unwrap_or(false);

    let tasks_disabled =
      CEIP_TASKS.iter().all(|task| check_task_disabled(task));

    let is_applied = registry_disabled && tasks_disabled;
    Ok(TweakState {
      id: self.meta.id.clone(),
      current_value: Some(if is_applied { "1" } else { "0" }.to_string()),
      is_applied,
    })
  }

  fn apply(&self, _value: Option<&str>) -> Result<(), String> {
    disable_ceip_tasks()?;
    registry::write_reg_u32(HKEY_LOCAL_MACHINE, SQMCLIENT_PATH, CEIP_ENABLE, 0)
      .map_err(|e| e.to_string())?;
    Ok(())
  }

  fn revert(&self) -> Result<(), String> {
    enable_ceip_tasks()?;
    registry::write_reg_u32(HKEY_LOCAL_MACHINE, SQMCLIENT_PATH, CEIP_ENABLE, 1)
      .map_err(|e| e.to_string())?;
    Ok(())
  }
}
