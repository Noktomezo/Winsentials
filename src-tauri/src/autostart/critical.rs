use crate::autostart::types::CriticalLevel;

const CRITICAL_NAMES: &[&str] = &[
  "explorer.exe",
  "dwm.exe",
  "svchost.exe",
  "csrss.exe",
  "lsass.exe",
  "wininit.exe",
  "services.exe",
  "securityhealth",
  "msmpeng.exe",
  "antimalware",
  "nissrv.exe",
  "senseir.exe",
  "onedrive.exe",
  "securityhealthservice.exe",
];

const WARNING_PATTERNS: &[&str] =
  &[r"\Temp\", r"\AppData\Local\Temp\", r"\Downloads\"];

pub fn get_critical_level(name: &str, command: &str) -> CriticalLevel {
  let name_lower = name.to_lowercase();
  let command_lower = command.to_lowercase();

  for critical in CRITICAL_NAMES {
    if name_lower.contains(&critical.to_lowercase()) {
      return CriticalLevel::Critical;
    }
  }

  for pattern in WARNING_PATTERNS {
    let pattern_lower = pattern.to_lowercase();
    if command_lower.contains(&pattern_lower) {
      return CriticalLevel::Warning;
    }
  }

  CriticalLevel::None
}
