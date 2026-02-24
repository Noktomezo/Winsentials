use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use crate::utils::command::hidden_command;

const TASK_NAME: &str = r"\Microsoft\Windows\Application Experience\Microsoft Compatibility Appraiser";

fn task_exists(task: &str) -> bool {
  hidden_command("schtasks")
    .args(["/query", "/tn", task])
    .output()
    .map(|o| o.status.success())
    .unwrap_or(false)
}

fn check_task() -> Result<bool, String> {
  let output = hidden_command("schtasks")
    .args(["/query", "/tn", TASK_NAME, "/xml"])
    .output()
    .map_err(|e| format!("Failed to query task: {}", e))?;

  if !output.status.success() {
    return Ok(true);
  }

  let stdout = String::from_utf8_lossy(&output.stdout);
  let is_disabled = stdout.contains("<Enabled>false</Enabled>");

  Ok(is_disabled)
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
    if !task_exists(TASK_NAME) {
      return Ok(());
    }

    let output = hidden_command("schtasks")
      .args(["/change", "/tn", TASK_NAME, "/disable"])
      .output()
      .map_err(|e| format!("Failed to disable task: {}", e))?;

    if !output.status.success() {
      let stderr = String::from_utf8_lossy(&output.stderr);
      return Err(format!("Failed to disable task: {}", stderr.trim()));
    }

    Ok(())
  }

  fn revert(&self) -> Result<(), String> {
    if !task_exists(TASK_NAME) {
      return Ok(());
    }

    let output = hidden_command("schtasks")
      .args(["/change", "/tn", TASK_NAME, "/enable"])
      .output()
      .map_err(|e| format!("Failed to enable task: {}", e))?;

    if !output.status.success() {
      let stderr = String::from_utf8_lossy(&output.stderr);
      return Err(format!("Failed to enable task: {}", stderr.trim()));
    }

    Ok(())
  }
}
