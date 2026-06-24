use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock, RwLock};

use serde::{Deserialize, Serialize};

use crate::error::AppError;

const WINAPP2_BUNDLED: &str = include_str!("../../assets/Winapp2.ini");
const WINAPPX_BUNDLED: &str = include_str!("../../assets/Winappx.ini");

const WINAPP2_URL: &str = "https://raw.githubusercontent.com/MoscaDotTo/Winapp2/main/Winapp2.ini";
const WINAPPX_URL: &str = "https://raw.githubusercontent.com/MoscaDotTo/Winapp2/main/Winappx.ini";

static APP_DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

static WINAPP2_CACHE: RwLock<Option<Arc<Vec<IniEntry>>>> = RwLock::new(None);
static WINAPPX_CACHE: RwLock<Option<Arc<Vec<IniEntry>>>> = RwLock::new(None);
static EXCLUDE_RULES_CACHE: RwLock<Option<Arc<Vec<ExcludeRule>>>> = RwLock::new(None);

static SETTINGS: OnceLock<RwLock<WinappDbSettings>> = OnceLock::new();

fn settings() -> &'static RwLock<WinappDbSettings> {
    SETTINGS.get_or_init(|| RwLock::new(WinappDbSettings::default()))
}

#[derive(Default, Serialize, Deserialize)]
struct WinappDbSettings {
    #[serde(default)]
    custom_winapp2_path: Option<PathBuf>,
}

#[derive(Clone)]
pub(super) struct IniEntry {
    pub(super) name: String,
    pub(super) values: HashMap<String, Vec<String>>,
}

impl IniEntry {
    pub(super) fn first(&self, key: &str) -> Option<String> {
        self.values
            .get(key)
            .and_then(|values| values.first())
            .cloned()
    }
}

#[derive(Clone)]
pub(super) enum ExcludeRuleType {
    File,
    Path,
}

#[derive(Clone)]
pub(super) struct ExcludeRule {
    pub(super) rule_type: ExcludeRuleType,
    pub(super) path: String,
    pub(super) file_pattern: Option<String>,
}

#[derive(Serialize)]
pub struct WinappDbStatus {
    pub source: &'static str,
    pub last_updated: Option<u64>,
    pub custom_path: Option<PathBuf>,
    pub cache_path: Option<PathBuf>,
}

#[derive(Serialize)]
pub struct DownloadReport {
    pub bytes: usize,
    pub path: PathBuf,
}

pub fn init(app_data_dir: PathBuf) {
    let cleanup_dir = app_data_dir.join("cleanup");
    let _ = std::fs::create_dir_all(&cleanup_dir);
    let _ = APP_DATA_DIR.set(cleanup_dir);

    if let Some(loaded) = load_settings() {
        *settings().write().expect("winapp settings lock poisoned") = loaded;
    }
}

fn cleanup_dir() -> Option<&'static PathBuf> {
    APP_DATA_DIR.get()
}

fn settings_path() -> Option<PathBuf> {
    cleanup_dir().map(|dir| dir.join("winapp-db-settings.json"))
}

fn load_settings() -> Option<WinappDbSettings> {
    let path = settings_path()?;
    let content = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

fn save_settings(settings: &WinappDbSettings) -> Result<(), AppError> {
    let path = settings_path()
        .ok_or_else(|| AppError::message("cleanup data directory is not initialised"))?;
    let content = serde_json::to_string(settings)
        .map_err(|e| AppError::message(format!("failed to serialise winapp settings: {e}")))?;
    std::fs::write(&path, content)
        .map_err(|e| AppError::message(format!("failed to write winapp settings: {e}")))?;
    Ok(())
}

pub(super) fn winapp2_entries() -> Arc<Vec<IniEntry>> {
    if let Some(cached) = WINAPP2_CACHE
        .read()
        .expect("winapp2 cache lock poisoned")
        .clone()
    {
        return cached;
    }
    let content = load_winapp2_content();
    let parsed = Arc::new(parse_ini(&content));
    *WINAPP2_CACHE.write().expect("winapp2 cache lock poisoned") = Some(parsed.clone());
    parsed
}

pub(super) fn winappx_entries() -> Arc<Vec<IniEntry>> {
    if let Some(cached) = WINAPPX_CACHE
        .read()
        .expect("winappx cache lock poisoned")
        .clone()
    {
        return cached;
    }
    let content = load_winappx_content();
    let parsed = Arc::new(parse_ini(&content));
    *WINAPPX_CACHE.write().expect("winappx cache lock poisoned") = Some(parsed.clone());
    parsed
}

pub(super) fn all_exclude_rules() -> Arc<Vec<ExcludeRule>> {
    if let Some(cached) = EXCLUDE_RULES_CACHE
        .read()
        .expect("exclude rules cache lock poisoned")
        .clone()
    {
        return cached;
    }
    let rules = Arc::new(
        winapp2_entries()
            .iter()
            .flat_map(exclude_rules_from_entry)
            .collect::<Vec<_>>(),
    );
    *EXCLUDE_RULES_CACHE
        .write()
        .expect("exclude rules cache lock poisoned") = Some(rules.clone());
    rules
}

pub fn refresh_cache() {
    *WINAPP2_CACHE.write().expect("winapp2 cache lock poisoned") = None;
    *WINAPPX_CACHE.write().expect("winappx cache lock poisoned") = None;
    *EXCLUDE_RULES_CACHE
        .write()
        .expect("exclude rules cache lock poisoned") = None;
}

fn resolve_winapp2_path() -> Option<PathBuf> {
    let settings = settings().read().expect("winapp settings lock poisoned");
    if let Some(custom) = &settings.custom_winapp2_path
        && custom.is_file()
    {
        return Some(custom.clone());
    }
    cleanup_dir()
        .map(|dir| dir.join("Winapp2.ini"))
        .filter(|p| p.is_file())
}

fn load_winapp2_content() -> String {
    if let Some(path) = resolve_winapp2_path()
        && let Ok(content) = std::fs::read_to_string(&path)
    {
        return content;
    }
    WINAPP2_BUNDLED.to_string()
}

fn load_winappx_content() -> String {
    if let Some(dir) = cleanup_dir() {
        let cached = dir.join("Winappx.ini");
        if cached.is_file()
            && let Ok(content) = std::fs::read_to_string(&cached)
        {
            return content;
        }
    }
    WINAPPX_BUNDLED.to_string()
}

pub fn winapp_db_status() -> WinappDbStatus {
    let settings = settings().read().expect("winapp settings lock poisoned");

    let custom_path = settings.custom_winapp2_path.clone();
    let cache_path = cleanup_dir().map(|dir| dir.join("Winapp2.ini"));

    let (source, last_updated) = if let Some(custom) = &custom_path {
        if custom.is_file() {
            let last_updated = file_mtime(custom);
            ("custom", last_updated)
        } else if let Some(ref cache) = cache_path {
            if cache.is_file() {
                let last_updated = file_mtime(cache);
                ("cache", last_updated)
            } else {
                ("bundled", None)
            }
        } else {
            ("bundled", None)
        }
    } else if let Some(ref cache) = cache_path {
        if cache.is_file() {
            let last_updated = file_mtime(cache);
            ("cache", last_updated)
        } else {
            ("bundled", None)
        }
    } else {
        ("bundled", None)
    };

    WinappDbStatus {
        source,
        last_updated,
        custom_path,
        cache_path,
    }
}

fn file_mtime(path: &std::path::Path) -> Option<u64> {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
}

pub fn set_custom_winapp2_path(path: Option<PathBuf>) -> Result<(), AppError> {
    let mut settings = settings().write().expect("winapp settings lock poisoned");
    settings.custom_winapp2_path = path;
    save_settings(&settings)
}

pub async fn download_winapp2() -> Result<DownloadReport, AppError> {
    let dir = cleanup_dir()
        .ok_or_else(|| AppError::message("cleanup data directory is not initialised"))?;
    std::fs::create_dir_all(dir)
        .map_err(|e| AppError::message(format!("failed to create cleanup cache dir: {e}")))?;

    let target = dir.join("Winapp2.ini");
    let content = download_text(WINAPP2_URL).await?;
    std::fs::write(&target, &content)
        .map_err(|e| AppError::message(format!("failed to write Winapp2.ini: {e}")))?;

    refresh_cache();
    Ok(DownloadReport {
        bytes: content.len(),
        path: target,
    })
}

pub async fn download_winappx() -> Result<DownloadReport, AppError> {
    let dir = cleanup_dir()
        .ok_or_else(|| AppError::message("cleanup data directory is not initialised"))?;
    std::fs::create_dir_all(dir)
        .map_err(|e| AppError::message(format!("failed to create cleanup cache dir: {e}")))?;

    let target = dir.join("Winappx.ini");
    let content = download_text(WINAPPX_URL).await?;
    std::fs::write(&target, &content)
        .map_err(|e| AppError::message(format!("failed to write Winappx.ini: {e}")))?;

    refresh_cache();
    Ok(DownloadReport {
        bytes: content.len(),
        path: target,
    })
}

async fn download_text(url: &str) -> Result<String, AppError> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| AppError::message(format!("failed to build HTTP client: {e}")))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| AppError::message(format!("failed to download Winapp2 database: {e}")))?;

    if !response.status().is_success() {
        return Err(AppError::message(format!(
            "Winapp2 download failed with HTTP {}",
            response.status()
        )));
    }

    response
        .text()
        .await
        .map_err(|e| AppError::message(format!("failed to read Winapp2 response body: {e}")))
}

pub(super) fn parse_ini(content: &str) -> Vec<IniEntry> {
    let mut entries = Vec::new();
    let mut current: Option<IniEntry> = None;

    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with(';') {
            continue;
        }

        if line.starts_with('[') && line.ends_with(']') {
            if let Some(entry) = current.take() {
                entries.push(entry);
            }
            current = Some(IniEntry {
                name: line
                    .trim_matches(['[', ']'])
                    .trim_end_matches(" *")
                    .to_string(),
                values: HashMap::new(),
            });
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if let Some(entry) = current.as_mut() {
            entry
                .values
                .entry(key.trim().to_string())
                .or_default()
                .push(value.trim().to_string());
        }
    }

    if let Some(entry) = current {
        entries.push(entry);
    }

    entries
}

fn exclude_rules_from_entry(entry: &IniEntry) -> Vec<ExcludeRule> {
    entry
        .values
        .iter()
        .filter(|(key, _)| key.starts_with("ExcludeKey"))
        .flat_map(|(_, values)| values)
        .filter_map(|value| parse_exclude_rule(value))
        .collect()
}

fn parse_exclude_rule(value: &str) -> Option<ExcludeRule> {
    let mut parts = value.split('|');
    let type_str = parts.next()?.trim().to_ascii_uppercase();
    let path = parts.next()?.trim().to_string();

    match type_str.as_str() {
        "FILE" => Some(ExcludeRule {
            rule_type: ExcludeRuleType::File,
            path,
            file_pattern: None,
        }),
        "PATH" => Some(ExcludeRule {
            rule_type: ExcludeRuleType::Path,
            path,
            file_pattern: parts
                .next()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
        }),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ini_handles_basic_entry() {
        let content = "[Test Entry *]\nFileKey1=%LocalAppData%\\Test|*.tmp\nDefault=False\n";
        let entries = parse_ini(content);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "Test Entry");
        assert!(entries[0].values.contains_key("FileKey1"));
        assert_eq!(
            entries[0].values.get("Default").and_then(|v| v.first()),
            Some(&"False".to_string())
        );
    }

    #[test]
    fn parse_ini_skips_comments_and_empty_lines() {
        let content = "; comment\n\n[Entry]\nFileKey1=path\n";
        let entries = parse_ini(content);
        assert_eq!(entries.len(), 1);
    }

    #[test]
    fn parse_exclude_rule_file_type() {
        let rule = parse_exclude_rule("FILE|%AppData%\\App\\important.dat");
        assert!(rule.is_some());
    }

    #[test]
    fn parse_exclude_rule_skips_reg_type() {
        let rule = parse_exclude_rule("REG|HKCU\\Software\\App");
        assert!(rule.is_none());
    }
}
