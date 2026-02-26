use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AutostartSource {
  Registry,
  Folder,
  Task,
  Service,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CriticalLevel {
  None,
  Warning,
  Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutostartItem {
  pub id: String,
  pub name: String,
  pub publisher: String,
  pub command: String,
  pub location: String,
  pub source: AutostartSource,
  pub is_enabled: bool,
  pub is_delayed: bool,
  pub icon_base64: Option<String>,
  pub critical_level: CriticalLevel,
  pub file_path: Option<String>,
  pub start_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichmentData {
  pub id: String,
  pub icon_base64: Option<String>,
  pub publisher: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnrichRequest {
  pub id: String,
  pub file_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileProperties {
  pub name: String,
  pub path: String,
  pub size: String,
  pub created: String,
  pub modified: String,
  pub version: Option<String>,
  pub publisher: Option<String>,
  pub description: Option<String>,
}
