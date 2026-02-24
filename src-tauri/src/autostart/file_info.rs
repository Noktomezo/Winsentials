use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use pelite::pe::Pe;
use pelite::pe::PeFile;

use crate::autostart::types::FileProperties;

static VERSION_CACHE: LazyLock<RwLock<HashMap<String, VersionInfoResult>>> =
  LazyLock::new(|| RwLock::new(HashMap::new()));

fn format_size(size: u64) -> String {
  const KB: u64 = 1024;
  const MB: u64 = KB * 1024;
  const GB: u64 = MB * 1024;

  if size >= GB {
    format!("{:.2} GB", size as f64 / GB as f64)
  } else if size >= MB {
    format!("{:.2} MB", size as f64 / MB as f64)
  } else if size >= KB {
    format!("{:.2} KB", size as f64 / KB as f64)
  } else {
    format!("{} B", size)
  }
}

fn format_datetime(dt: DateTime<Utc>) -> String {
  dt.format("%Y-%m-%d %H:%M:%S").to_string()
}

#[derive(Clone)]
pub struct VersionInfoResult {
  pub file_version: Option<String>,
  pub company_name: Option<String>,
  pub file_description: Option<String>,
}

pub fn get_file_version_info(path: &str) -> Result<VersionInfoResult, String> {
  let normalized_key = path.to_lowercase();

  {
    let cache = VERSION_CACHE.read();
    if let Some(cached) = cache.get(&normalized_key) {
      return Ok(cached.clone());
    }
  }

  let buffer = std::fs::read(path).map_err(|e| e.to_string())?;

  let pe_file = PeFile::from_bytes(&buffer)
    .map_err(|e| format!("Failed to parse PE: {:?}", e))?;

  let resources = pe_file
    .resources()
    .map_err(|_| "No resources".to_string())?;

  let version_info = resources
    .version_info()
    .map_err(|_| "No version info".to_string())?;

  let fixed_info = version_info.fixed();
  let file_version = fixed_info.map(|fi| {
    let v = fi.dwFileVersion;
    format!("{}", v)
  });

  let strings = version_info.file_info();

  let company_name = strings
    .strings
    .values()
    .next()
    .and_then(|lang| lang.get("CompanyName"))
    .map(|s| s.to_string());

  let file_description = strings
    .strings
    .values()
    .next()
    .and_then(|lang| lang.get("FileDescription"))
    .map(|s| s.to_string());

  let result = VersionInfoResult {
    file_version,
    company_name,
    file_description,
  };

  {
    let mut cache = VERSION_CACHE.write();
    if let Some(cached) = cache.get(&normalized_key) {
      return Ok(cached.clone());
    }
    cache.insert(normalized_key, result.clone());
  }

  Ok(result)
}

pub fn get_file_properties(path: &str) -> Result<FileProperties, String> {
  let file_path = Path::new(path);

  if !file_path.exists() {
    return Err("File does not exist".to_string());
  }

  let name = file_path
    .file_name()
    .map(|n| n.to_string_lossy().to_string())
    .unwrap_or_else(|| path.to_string());

  let metadata = fs::metadata(file_path)
    .map_err(|e| format!("Failed to read file metadata: {}", e))?;

  let size = format_size(metadata.len());

  let created = metadata
    .created()
    .ok()
    .map(|t| {
      let datetime: DateTime<Utc> = t.into();
      format_datetime(datetime)
    })
    .unwrap_or_else(|| "Unknown".to_string());

  let modified = metadata
    .modified()
    .ok()
    .map(|t| {
      let datetime: DateTime<Utc> = t.into();
      format_datetime(datetime)
    })
    .unwrap_or_else(|| "Unknown".to_string());

  let (version, publisher, description) = get_version_info_safe(path);

  Ok(FileProperties {
    name,
    path: path.to_string(),
    size,
    created,
    modified,
    version,
    publisher,
    description,
  })
}

fn get_version_info_safe(
  path: &str,
) -> (Option<String>, Option<String>, Option<String>) {
  match get_file_version_info(path) {
    Ok(info) => (info.file_version, info.company_name, info.file_description),
    Err(_) => (None, None, None),
  }
}

pub fn open_file_location(path: &str) -> Result<(), String> {
  let file_path = Path::new(path);

  if !file_path.exists() {
    return Err("File does not exist".to_string());
  }

  let parent = file_path
    .parent()
    .ok_or("Cannot determine parent directory")?;

  std::process::Command::new("explorer")
    .arg(parent)
    .spawn()
    .map_err(|e| format!("Failed to open explorer: {}", e))?;

  Ok(())
}
