use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StartupSource {
    Registry,
    StartupFolder,
    ScheduledTask,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StartupScope {
    CurrentUser,
    AllUsers,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StartupStatus {
    Enabled,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupEntry {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub source: StartupSource,
    pub scope: StartupScope,
    pub status: StartupStatus,
    pub command: Option<String>,
    pub target_path: Option<String>,
    pub arguments: Option<String>,
    pub working_directory: Option<String>,
    pub location_label: String,
    pub source_display: String,
    pub run_once: bool,
    pub publisher: Option<String>,
    pub icon_data_url: Option<String>,
    pub registry_path: Option<String>,
    pub task_path: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupEntryDetails {
    #[serde(flatten)]
    pub entry: StartupEntry,
    pub registry_hive: Option<String>,
    pub registry_path: Option<String>,
    pub registry_value_name: Option<String>,
    pub startup_folder_path: Option<String>,
    pub startup_file_path: Option<String>,
    pub task_path: Option<String>,
    pub task_author: Option<String>,
    pub task_description: Option<String>,
    pub task_triggers: Vec<String>,
    pub task_actions: Vec<String>,
    pub raw_xml_preview: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupSourceListResponse {
    pub source: StartupSource,
    pub entries: Vec<StartupEntry>,
    pub error: Option<String>,
}
