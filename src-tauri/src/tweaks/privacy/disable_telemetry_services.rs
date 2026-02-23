use crate::tweaks::{
  RiskLevel, Tweak, TweakCategory, TweakMeta, TweakState, TweakUiType,
};
use std::process::Command;

const DIAG_TRACKVC: &str = "DiagTrack";
const DMWAPPUSH_SERVICE: &str = "dmwappushservice";

fn check_service(service: &str) -> Result<bool, String> {
  let output = Command::new("sc")
    .args(["query", service])
    .output()
    .map_err(|e| format!("Failed to query service: {}", e))?;

  let stdout = String::from_utf8_lossy(&output.stdout);
  let stderr = String::from_utf8_lossy(&output.stderr);

  if stderr.contains("1060") || stdout.contains("does not exist") {
    return Ok(true);
  }

  let qc_output = Command::new("sc")
    .args(["qc", service])
    .output()
    .map_err(|e| format!("Failed to query service config: {}", e))?;

  let qc_stdout = String::from_utf8_lossy(&qc_output.stdout);
  let qc_stderr = String::from_utf8_lossy(&qc_output.stderr);

  if qc_stderr.contains("1060") || qc_stdout.contains("does not exist") {
    return Ok(true);
  }

  let is_disabled = parse_start_type(&qc_stdout) == Some(4);

  Ok(is_disabled)
}

fn parse_start_type(qc_stdout: &str) -> Option<u32> {
  for line in qc_stdout.lines() {
    let line = line.trim();
    if line.starts_with("START_TYPE") {
      let parts: Vec<&str> = line.split_whitespace().collect();
      if parts.len() >= 3
        && let Ok(start_type) = parts[2].parse::<u32>() {
          return Some(start_type);
        }
    }
  }
  None
}

fn set_service_disabled(service: &str) -> Result<(), String> {
  let output = Command::new("sc")
    .args(["query", service])
    .output()
    .map_err(|e| format!("Failed to query service: {}", e))?;

  let stdout = String::from_utf8_lossy(&output.stdout);
  let stderr = String::from_utf8_lossy(&output.stderr);

  if stderr.contains("1060") || stdout.contains("does not exist") {
    return Ok(());
  }

  let output = Command::new("sc")
    .args(["config", service, "start=", "disabled"])
    .output()
    .map_err(|e| format!("Failed to disable {}: {}", service, e))?;

  if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    return Err(format!("Failed to disable {}: {}", service, stderr.trim()));
  }

  let _ = Command::new("sc").args(["stop", service]).output();

  Ok(())
}

fn set_service_auto(service: &str) -> Result<(), String> {
  let output = Command::new("sc")
    .args(["query", service])
    .output()
    .map_err(|e| format!("Failed to query service: {}", e))?;

  let stdout = String::from_utf8_lossy(&output.stdout);
  let stderr = String::from_utf8_lossy(&output.stderr);

  if stderr.contains("1060") || stdout.contains("does not exist") {
    return Ok(());
  }

  let output = Command::new("sc")
    .args(["config", service, "start=", "auto"])
    .output()
    .map_err(|e| format!("Failed to enable {}: {}", service, e))?;

  if !output.status.success() {
    let stderr = String::from_utf8_lossy(&output.stderr);
    return Err(format!("Failed to enable {}: {}", service, stderr.trim()));
  }

  let _ = Command::new("sc").args(["start", service]).output();

  Ok(())
}

pub struct DisableTelemetryServicesTweak {
  meta: TweakMeta,
}

impl DisableTelemetryServicesTweak {
  pub fn new() -> Self {
    Self {
      meta: TweakMeta {
        id: "disable_telemetry_services".to_string(),
        category: TweakCategory::Privacy,
        name_key: "tweaks.disableTelemetryServices.name".to_string(),
        description_key: "tweaks.disableTelemetryServices.description"
          .to_string(),
        details_key: "tweaks.disableTelemetryServices.details".to_string(),
        ui_type: TweakUiType::Toggle,
        options: vec![],
        requires_reboot: false,
        risk_level: RiskLevel::Low,
        min_windows_build: None,
      },
    }
  }
}

impl Tweak for DisableTelemetryServicesTweak {
  fn meta(&self) -> &TweakMeta {
    &self.meta
  }

  fn check(&self) -> Result<TweakState, String> {
    let diag_disabled = check_service(DIAG_TRACKVC)?;
    let dmw_disabled = check_service(DMWAPPUSH_SERVICE)?;
    let is_applied = diag_disabled && dmw_disabled;

    Ok(TweakState {
      id: self.meta.id.clone(),
      current_value: Some(if is_applied { "1" } else { "0" }.to_string()),
      is_applied,
    })
  }

  fn apply(&self, _value: Option<&str>) -> Result<(), String> {
    set_service_disabled(DIAG_TRACKVC)?;
    set_service_disabled(DMWAPPUSH_SERVICE)?;
    Ok(())
  }

  fn revert(&self) -> Result<(), String> {
    set_service_auto(DIAG_TRACKVC)?;
    set_service_auto(DMWAPPUSH_SERVICE)?;
    Ok(())
  }
}
