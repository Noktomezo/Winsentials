use std::fs;
use std::path::PathBuf;

use lnk::ShellLink;

use crate::autostart::critical::get_critical_level;
use crate::autostart::file_info::get_file_version_info;
use crate::autostart::icons::get_icon;
use crate::autostart::types::{AutostartItem, AutostartSource};

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

  let common_startup = PathBuf::from(
    r"C:\ProgramData\Microsoft\Windows\Start Menu\Programs\Startup",
  );
  if common_startup.exists() {
    folders.push(("Common Startup".to_string(), common_startup));
  }

  folders
}

fn parse_lnk_file(path: &std::path::Path) -> Option<(String, String)> {
  let link = ShellLink::open(path).ok()?;

  let target = if let Some(info) = link.link_info() {
    if let Some(local) = info.local_base_path() {
      local.clone()
    } else if let Some(rel) = link.relative_path() {
      if let Some(parent) = path.parent() {
        parent.join(rel).to_string_lossy().to_string()
      } else {
        rel.to_string()
      }
    } else {
      path.to_string_lossy().to_string()
    }
  } else if let Some(rel) = link.relative_path() {
    if let Some(parent) = path.parent() {
      parent.join(rel).to_string_lossy().to_string()
    } else {
      rel.to_string()
    }
  } else {
    path.to_string_lossy().to_string()
  };

  let args: String = link
    .arguments().clone()
    .unwrap_or_default();

  let command = if args.is_empty() {
    target.clone()
  } else {
    format!("{} {}", target, args)
  };

  Some((target, command))
}

fn get_disabled_folder() -> PathBuf {
  let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
  home.join(".winsentials").join("startup_disabled")
}

pub fn get_folder_autostart_items() -> Vec<AutostartItem> {
  let mut items = Vec::new();
  let disabled_folder = get_disabled_folder();

  for (location_name, folder_path) in get_startup_folders() {
    if !folder_path.exists() {
      continue;
    }

    if let Ok(entries) = fs::read_dir(&folder_path) {
      for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();

        if path
          .extension()
          .map(|e| e.to_string_lossy().to_lowercase() == "lnk")
          .unwrap_or(false)
        {
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

          let is_disabled =
            disabled_folder.join(path.file_name().unwrap()).exists();

          let icon_base64 = get_icon(&target_path);

          let critical_level = get_critical_level(&name, &command);

          let publisher = get_file_version_info(&target_path)
            .ok()
            .and_then(|v| v.company_name)
            .unwrap_or_default();

          let id = format!(
            "folder|{}|{}",
            location_name.replace(' ', "_"),
            path.file_name().unwrap().to_string_lossy()
          );

          items.push(AutostartItem {
            id,
            name,
            publisher,
            command,
            location: location_name.clone(),
            source: AutostartSource::Folder,
            is_enabled: !is_disabled,
            is_delayed: false,
            icon_base64,
            critical_level,
            file_path: Some(target_path),
          });
        }
      }
    }
  }

  items
}

pub fn toggle_folder_item(id: &str, enable: bool) -> Result<(), String> {
  let parts: Vec<&str> = id.split('|').collect();
  if parts.len() != 3 || parts[0] != "folder" {
    return Err("Invalid folder item ID".to_string());
  }

  let location = parts[1];
  let filename = parts[2];

  let source_folders: Vec<(String, PathBuf)> = get_startup_folders();
  let source_path = source_folders
    .iter()
    .find(|(name, _)| name.replace(' ', "_") == location)
    .map(|(_, p)| p.clone())
    .ok_or("Source folder not found")?;

  let disabled_folder = get_disabled_folder();
  let _ = fs::create_dir_all(&disabled_folder);

  let source_file = source_path.join(filename);
  let disabled_file = disabled_folder.join(filename);

  if enable {
    if disabled_file.exists() {
      fs::rename(&disabled_file, &source_file)
        .map_err(|e| format!("Failed to restore file: {}", e))?;
    }
  } else if source_file.exists() {
    fs::rename(&source_file, &disabled_file)
      .map_err(|e| format!("Failed to disable file: {}", e))?;
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

  let source_folders: Vec<(String, PathBuf)> = get_startup_folders();
  let source_path = source_folders
    .iter()
    .find(|(name, _)| name.replace(' ', "_") == location)
    .map(|(_, p)| p.clone())
    .ok_or("Source folder not found")?;

  let disabled_folder = get_disabled_folder();

  let source_file = source_path.join(filename);
  let disabled_file = disabled_folder.join(filename);

  if source_file.exists() {
    fs::remove_file(&source_file)
      .map_err(|e| format!("Failed to delete file: {}", e))?;
  }

  if disabled_file.exists() {
    fs::remove_file(&disabled_file)
      .map_err(|e| format!("Failed to delete disabled file: {}", e))?;
  }

  Ok(())
}
