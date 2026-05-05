use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CleanupEntryStatus {
    Clean,
    Pending,
    Busy,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupEntry {
    pub id: String,
    pub name: String,
    pub path: String,
    pub status: CleanupEntryStatus,
    pub size_bytes: u64,
    pub error: Option<String>,
    pub icon_data_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupCategoryReport {
    pub id: String,
    pub entries: Vec<CleanupEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupScheduleEntry {
    pub path: String,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupScheduleReport {
    pub entries: Vec<CleanupScheduleEntry>,
}
