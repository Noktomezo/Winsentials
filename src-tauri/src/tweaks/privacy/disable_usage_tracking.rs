use crate::tweaks::registry;
use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use std::process::Command;
use winreg::enums::*;

const EXPLORER_ADVANCED_PATH: &str =
  r"Software\Microsoft\Windows\CurrentVersion\Explorer\Advanced";
const START_TRACK_PROGS: &str = "Start_TrackProgs";
const TASK_NAME: &str =
  r"\Microsoft\Windows\Application Experience\ProgramDataUpdater";

fn is_task_disabled() -> Result<bool, String> {
  let output = Command::new("schtasks")
    .args(["/query", "/tn", TASK_NAME, "/xml"])
    .output()
    .map_err(|e| format!("Failed to query task: {}", e))?;

  if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    if stderr.contains("cannot find") || stderr.contains("does not exist") {
      return Ok(true);
    }
    return Err(format!("Failed to query task: {}", stderr.trim()));
  }

  let stdout = String::from_utf8_lossy(&output.stdout);

  let is_disabled = stdout.contains("<State>Disabled</State>")
    || stdout.contains("<Enabled>false</Enabled>");

  Ok(is_disabled)
}

pub struct DisableUsageTrackingTweak {
  meta: TweakMeta,
}

impl DisableUsageTrackingTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "disable_usage_tracking".to_string(),
        category: TweakCategory::Privacy,
        name_key: "tweaks.disableUsageTracking.name".to_string(),
        description_key: "tweaks.disableUsageTracking.description".to_string(),
        details_key: "tweaks.disableUsageTracking.details".to_string(),
        ui_type: TweakUiType::Toggle,
        options: vec![],
        requires_reboot: false,
        risk_level: RiskLevel::Low,
        min_windows_build: None,
      },
    }
  }
}

impl Tweak for DisableUsageTrackingTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let track_progs = registry::read_reg_u32(
      HKEY_CURRENT_USER,
      EXPLORER_ADVANCED_PATH,
      START_TRACK_PROGS,
    );
    let task_disabled = is_task_disabled()?;
    let is_applied =
      track_progs.map(|v| v == 0).unwrap_or(false) && task_disabled;

    Ok(TweakState {
      id: self.meta.id.clone(),
      current_value: Some(if is_applied { "1" } else { "0" }.to_string()),
      is_applied,
    })
  }

  fn apply(&self, _value: Option<&str>) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_CURRENT_USER,
      EXPLORER_ADVANCED_PATH,
      START_TRACK_PROGS,
      0,
    )
    .map_err(|e| e.to_string())?;

    let output = Command::new("schtasks")
      .args(["/change", "/tn", TASK_NAME, "/disable"])
      .output()
      .map_err(|e| format!("Failed to disable task: {}", e))?;

    if !output.status.success() {
      let stderr = String::from_utf8_lossy(&output.stderr);
      if !stderr.contains("cannot find") && !stderr.contains("does not exist") {
        return Err(format!("Failed to disable task: {}", stderr.trim()));
      }
    }

    Ok(())
  }

  fn revert(&self) -> Result<(), String> {
    registry::write_reg_u32(
      HKEY_CURRENT_USER,
      EXPLORER_ADVANCED_PATH,
      START_TRACK_PROGS,
      1,
    )
    .map_err(|e| e.to_string())?;

    let output = Command::new("schtasks")
      .args(["/change", "/tn", TASK_NAME, "/enable"])
      .output()
      .map_err(|e| format!("Failed to enable task: {}", e))?;

    if !output.status.success() {
      let stderr = String::from_utf8_lossy(&output.stderr);
      if !stderr.contains("cannot find") && !stderr.contains("does not exist") {
        return Err(format!("Failed to enable task: {}", stderr.trim()));
      }
    }

    Ok(())
  }
}
