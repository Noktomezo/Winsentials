use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use std::process::Command;

const TASK_NAME: &str = r"\Microsoft\Windows\Application Experience\Microsoft Compatibility Appraiser";

fn check_task() -> Result<bool, String> {
  let output = Command::new("schtasks")
    .args(["/query", "/tn", TASK_NAME])
    .output();

  match output {
    Ok(o) => {
      let stdout = String::from_utf8_lossy(&o.stdout);
      if stdout.contains("Disabled") {
        return Ok(true);
      }
      if stdout.contains("Ready") || stdout.contains("Running") {
        return Ok(false);
      }
      Ok(true)
    }
    Err(_) => Ok(true),
  }
}

pub struct DisableCompatAppraiserTweak {
  meta: TweakMeta,
}

impl DisableCompatAppraiserTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "disable_compat_appraiser".to_string(),
        category: TweakCategory::Privacy,
        name_key: "tweaks.disableCompatAppraiser.name".to_string(),
        description_key: "tweaks.disableCompatAppraiser.description"
          .to_string(),
        details_key: "tweaks.disableCompatAppraiser.details".to_string(),
        ui_type: TweakUiType::Toggle,
        options: vec![],
        requires_reboot: false,
        risk_level: RiskLevel::Low,
        min_windows_build: None,
      },
    }
  }
}

impl Tweak for DisableCompatAppraiserTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let is_applied = check_task()?;

    Ok(TweakState {
      id: self.meta.id.clone(),
      current_value: Some(if is_applied { "1" } else { "0" }.to_string()),
      is_applied,
    })
  }

  fn apply(&self, _value: Option<&str>) -> Result<(), String> {
    let output = Command::new("schtasks")
      .args(["/query", "/tn", TASK_NAME])
      .output();

    match output {
      Ok(o) => {
        let stdout = String::from_utf8_lossy(&o.stdout);
        if stdout.contains("does not exist") {
          return Ok(());
        }
      }
      Err(_) => return Ok(()),
    }

    Command::new("schtasks")
      .args(["/change", "/tn", TASK_NAME, "/disable"])
      .status()
      .map_err(|e| format!("Failed to disable task: {}", e))?;
    Ok(())
  }

  fn revert(&self) -> Result<(), String> {
    let output = Command::new("schtasks")
      .args(["/query", "/tn", TASK_NAME])
      .output();

    match output {
      Ok(o) => {
        let stdout = String::from_utf8_lossy(&o.stdout);
        if stdout.contains("does not exist") {
          return Ok(());
        }
      }
      Err(_) => return Ok(()),
    }

    Command::new("schtasks")
      .args(["/change", "/tn", TASK_NAME, "/enable"])
      .status()
      .map_err(|e| format!("Failed to enable task: {}", e))?;
    Ok(())
  }
}
