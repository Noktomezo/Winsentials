use serde::{Deserialize, Serialize};
use winreg::RegValue;
use winreg::enums::RegType;
use winreg::types::{FromRegValue, ToRegValue};

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

impl StartupScope {
    fn as_serialized(&self) -> &'static str {
        match self {
            Self::CurrentUser => "current_user",
            Self::AllUsers => "all_users",
        }
    }
}

impl ToRegValue for StartupScope {
    fn to_reg_value(&self) -> RegValue<'_> {
        let mut bytes = Vec::with_capacity((self.as_serialized().len() + 1) * 2);
        for code_unit in self
            .as_serialized()
            .encode_utf16()
            .chain(std::iter::once(0))
        {
            bytes.extend_from_slice(&code_unit.to_le_bytes());
        }

        RegValue {
            bytes: bytes.into(),
            vtype: RegType::REG_SZ,
        }
    }
}

impl FromRegValue for StartupScope {
    fn from_reg_value(value: &RegValue) -> std::io::Result<Self> {
        match String::from_reg_value(value)?.as_str() {
            "current_user" => Ok(Self::CurrentUser),
            "all_users" => Ok(Self::AllUsers),
            other => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("invalid startup scope: {other}"),
            )),
        }
    }
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
