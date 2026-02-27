use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Component, Path, PathBuf};

use lnk::ShellLink;
use lnk::encoding::WINDOWS_1252;
use rayon::prelude::*;

use crate::autostart::critical::get_critical_level;
use crate::autostart::file_info::get_file_version_info;
use crate::autostart::icons::get_icon;
use crate::autostart::types::{AutostartItem, AutostartSource};

fn validate_filename(filename: &str) -> Result<(), String> {
  if filename.is_empty() {
    return Err("Filename is empty".to_string());
  }

  let path = Path::new(filename);

  if path.is_absolute() {
    return Err("Filename must not be absolute".to_string());
  }

  let components: Vec<_> = path.components().collect();
  if components.len() != 1 {
    return Err("Filename must contain exactly one path component".to_string());
  }

  match &components[0] {
    Component::Normal(_) => Ok(()),
    _ => Err("Filename contains invalid path components".to_string()),
  }
}

fn get_startup_folders() -> Vec<(String, PathBuf)> {
  let mut folders = Vec::new();

  if let Some(appdata) = dirs::data_dir() {
    let user_startup = appdata
      .join("Microsoft")
      .join("Windows")
      .join("Start Menu")
      .join("Programs")
      .join("Startup");
    folders.push(("User Startup".to_string(), user_startup));
  }

  let program_data = env::var_os("PROGRAMDATA")
    .map(PathBuf::from)
    .unwrap_or_else(|| PathBuf::from("C:\\ProgramData"));
  let common_startup = program_data
    .join("Microsoft")
    .join("Windows")
    .join("Start Menu")
    .join("Programs")
    .join("Startup");
  if common_startup.exists() {
    folders.push(("Common Startup".to_string(), common_startup));
  }

  folders
}

fn parse_lnk_file(path: &std::path::Path) -> Option<(String, String)> {
  let link = ShellLink::open(path, WINDOWS_1252).ok()?;

  let target = if let Some(info) = link.link_info() {
    if let Some(local) = info.local_base_path() {
      local.to_string()
    } else if let Some(rel) = link.string_data().relative_path() {
      if let Some(parent) = path.parent() {
        parent.join(rel).to_string_lossy().to_string()
      } else {
        rel.clone()
      }
    } else {
      path.to_string_lossy().to_string()
    }
  } else if let Some(rel) = link.string_data().relative_path() {
    if let Some(parent) = path.parent() {
      parent.join(rel).to_string_lossy().to_string()
    } else {
      rel.clone()
    }
  } else {
    path.to_string_lossy().to_string()
  };

  let args: String = link
    .string_data()
    .command_line_arguments()
    .clone()
    .unwrap_or_default();

  let command = if args.is_empty() {
    target.clone()
  } else {
    format!("{target} {args}")
  };

  Some((target, command))
}

fn get_disabled_folder(location: &str) -> Option<PathBuf> {
  let home = dirs::home_dir()?;
  Some(
    home
      .join(".winsentials")
      .join("startup_disabled")
      .join(location.replace(' ', "_")),
  )
}

struct RawFolderItem {
  name: String,
  target_path: String,
  command: String,
  location_name: String,
  filename: String,
  is_enabled: bool,
}

fn collect_raw_lnk_items(
  dir: &Path,
  location_name: &str,
  is_enabled: bool,
  seen_files: &mut HashSet<String>,
  items: &mut Vec<RawFolderItem>,
) {
  if let Ok(entries) = fs::read_dir(dir) {
    for entry in entries.filter_map(|e| e.ok()) {
      let path = entry.path();

      if path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase() == "lnk")
        .unwrap_or(false)
      {
        let filename = match path.file_name() {
          Some(n) => n.to_string_lossy().to_string(),
          None => continue,
        };

        if filename.is_empty() {
          continue;
        }

        if seen_files.contains(&filename) {
          continue;
        }
        seen_files.insert(filename.clone());

        let name = path
          .file_stem()
          .map(|s| s.to_string_lossy().to_string())
          .unwrap_or_else(|| "Unknown".to_string());

        let (target_path, command) =
          parse_lnk_file(&path).unwrap_or_else(|| {
            (
              path.to_string_lossy().to_string(),
              path.to_string_lossy().to_string(),
            )
          });

        items.push(RawFolderItem {
          name,
          target_path,
          command,
          location_name: location_name.to_string(),
          filename,
          is_enabled,
        });
      }
    }
  }
}

fn enrich_folder_item(raw: RawFolderItem) -> AutostartItem {
  let icon_base64 = get_icon(&raw.target_path);

  let critical_level = get_critical_level(&raw.name, &raw.command);

  let publisher = get_file_version_info(&raw.target_path)
    .ok()
    .and_then(|v| v.company_name)
    .unwrap_or_default();

  let id = format!(
    "folder|{}|{}",
    raw.location_name.replace(' ', "_"),
    raw.filename
  );

  AutostartItem {
    id,
    name: raw.name,
    publisher,
    command: raw.command,
    location: raw.location_name,
    source: AutostartSource::Folder,
    is_enabled: raw.is_enabled,
    is_delayed: false,
    icon_base64,
    critical_level,
    file_path: Some(raw.target_path),
    start_type: None,
  }
}

fn collect_all_raw_folder_items() -> Vec<RawFolderItem> {
  let mut raw_items = Vec::new();

  for (location_name, folder_path) in get_startup_folders() {
    if !folder_path.exists() {
      continue;
    }

    let disabled_folder = get_disabled_folder(&location_name);
    let mut seen_files: HashSet<String> = HashSet::new();

    collect_raw_lnk_items(
      &folder_path,
      &location_name,
      true,
      &mut seen_files,
      &mut raw_items,
    );

    if let Some(disabled_path) = &disabled_folder
      && disabled_path.exists()
    {
      collect_raw_lnk_items(
        disabled_path,
        &location_name,
        false,
        &mut seen_files,
        &mut raw_items,
      );
    }
  }

  raw_items
}

pub fn get_folder_autostart_items() -> Vec<AutostartItem> {
  collect_all_raw_folder_items()
    .into_par_iter()
    .map(enrich_folder_item)
    .collect()
}

pub fn get_folder_items_fast() -> Vec<AutostartItem> {
  collect_all_raw_folder_items()
    .into_iter()
    .map(|raw| {
      let critical_level = get_critical_level(&raw.name, &raw.command);

      let id = format!(
        "folder|{}|{}",
        raw.location_name.replace(' ', "_"),
        raw.filename
      );

      AutostartItem {
        id,
        name: raw.name,
        publisher: String::new(),
        command: raw.command,
        location: raw.location_name,
        source: AutostartSource::Folder,
        is_enabled: raw.is_enabled,
        is_delayed: false,
        icon_base64: None,
        critical_level,
        file_path: Some(raw.target_path),
        start_type: None,
      }
    })
    .collect()
}

pub fn toggle_folder_item(id: &str, enable: bool) -> Result<(), String> {
  let parts: Vec<&str> = id.split('|').collect();
  if parts.len() != 3 || parts[0] != "folder" {
    return Err("Invalid folder item ID".to_string());
  }

  let location = parts[1];
  let filename = parts[2];

  validate_filename(filename)?;

  let source_folders: Vec<(String, PathBuf)> = get_startup_folders();
  let source_path = source_folders
    .iter()
    .find(|(name, _)| name.replace(' ', "_") == location)
    .map(|(_, p)| p.clone())
    .ok_or("Source folder not found")?;

  let disabled_folder = get_disabled_folder(location)
    .ok_or("Cannot determine disabled folder path")?;

  let source_file = source_path.join(filename);
  let disabled_file = disabled_folder.join(filename);

  if enable {
    if disabled_file.exists() {
      fs::rename(&disabled_file, &source_file)
        .map_err(|e| format!("Failed to restore file: {e}"))?;
    }
  } else {
    fs::create_dir_all(&disabled_folder)
      .map_err(|e| format!("Failed to create disabled folder: {e}"))?;
    if source_file.exists() {
      fs::rename(&source_file, &disabled_file)
        .map_err(|e| format!("Failed to disable file: {e}"))?;
    }
  }

  Ok(())
}

pub fn delete_folder_item(id: &str) -> Result<(), String> {
  let parts: Vec<&str> = id.split('|').collect();
  if parts.len() != 3 || parts[0] != "folder" {
    return Err("Invalid folder item ID".to_string());
  }

  let location = parts[1];
  let filename = parts[2];

  validate_filename(filename)?;

  let source_folders: Vec<(String, PathBuf)> = get_startup_folders();
  let source_path = source_folders
    .iter()
    .find(|(name, _)| name.replace(' ', "_") == location)
    .map(|(_, p)| p.clone())
    .ok_or("Source folder not found")?;

  let disabled_folder = get_disabled_folder(location)
    .ok_or("Cannot determine disabled folder path")?;

  let source_file = source_path.join(filename);
  let disabled_file = disabled_folder.join(filename);

  if source_file.exists() {
    fs::remove_file(&source_file)
      .map_err(|e| format!("Failed to delete file: {e}"))?;
  }

  if disabled_file.exists() {
    fs::remove_file(&disabled_file)
      .map_err(|e| format!("Failed to delete disabled file: {e}"))?;
  }

  Ok(())
}
