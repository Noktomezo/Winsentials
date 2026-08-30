use std::fs;
use std::path::{Path, PathBuf};

use super::types::{StartupEntry, StartupScope, StartupSource, StartupStatus};
use super::vendor::{extract_clean_exe_path, get_file_publisher};

pub fn scan_tasks_startup() -> Vec<StartupEntry> {
    let mut entries = Vec::new();
    let tasks_dir = PathBuf::from(r"C:\Windows\System32\Tasks");
    if !tasks_dir.exists() {
        return entries;
    }

    scan_task_dir(&tasks_dir, &tasks_dir, &mut entries);
    entries
}

fn scan_task_dir(root: &Path, current_dir: &Path, entries: &mut Vec<StartupEntry>) {
    let Ok(read_dir) = fs::read_dir(current_dir) else {
        return;
    };

    for entry in read_dir.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Skip Microsoft internal system tasks
            let path_str = path.to_string_lossy();
            if path_str.ends_with(r"\Microsoft\Windows")
                || path_str.ends_with(r"\Microsoft\XblGameSave")
            {
                continue;
            }
            scan_task_dir(root, &path, entries);
        } else if path.is_file() {
            if let Some(task_entry) = parse_task_file(root, &path) {
                entries.push(task_entry);
            }
        }
    }
}

fn parse_task_file(root: &Path, file_path: &Path) -> Option<StartupEntry> {
    let content = fs::read_to_string(file_path).ok()?;
    if !content.contains("<Task") {
        return None;
    }

    // Extract Command
    let command = extract_xml_tag(&content, "Command")?;
    let trimmed_cmd = command.trim();
    if trimmed_cmd.is_empty() {
        return None;
    }

    let arguments = extract_xml_tag(&content, "Arguments");
    let author = extract_xml_tag(&content, "Author");

    let full_command = match arguments {
        Some(ref args) if !args.trim().is_empty() => format!("{command} {args}"),
        _ => command.clone(),
    };

    // Filter out core system binaries
    let lower_cmd = command.to_ascii_lowercase();
    if lower_cmd.contains(r"\windows\system32\") && !lower_cmd.contains("driver") {
        return None;
    }

    // Extract Enabled status
    let enabled_str = extract_xml_tag(&content, "Enabled").unwrap_or_else(|| "true".to_string());
    let status = if enabled_str.eq_ignore_ascii_case("false") {
        StartupStatus::Disabled
    } else {
        StartupStatus::Enabled
    };

    // Relative task path for schtasks command line
    let rel_path = file_path
        .strip_prefix(root)
        .ok()?
        .to_string_lossy()
        .to_string();
    let task_tn = if rel_path.starts_with('\\') {
        rel_path.clone()
    } else {
        format!(r"\{rel_path}")
    };

    let file_stem = file_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(&rel_path);

    let display_name = file_stem.to_string();
    let target_exe = extract_clean_exe_path(&command);
    let target_str = target_exe.as_ref().map(|p| p.to_string_lossy().to_string());

    let publisher = target_exe
        .as_deref()
        .and_then(get_file_publisher)
        .or_else(|| {
            author.and_then(|a| {
                let a_trimmed = a.trim();
                if !a_trimmed.is_empty() && !a_trimmed.eq_ignore_ascii_case("unknown") {
                    Some(a_trimmed.to_string())
                } else {
                    None
                }
            })
        });

    Some(StartupEntry {
        id: format!("task_{rel_path}"),
        name: file_stem.to_string(),
        display_name,
        publisher,
        source: StartupSource::ScheduledTask,
        scope: StartupScope::AllUsers,
        status,
        command: Some(full_command),
        target_path: target_str,
        location_label: "Task Scheduler".to_string(),
        raw_id: task_tn,
    })
}

fn extract_xml_tag(xml: &str, tag_name: &str) -> Option<String> {
    let open_tag = format!("<{tag_name}>");
    let close_tag = format!("</{tag_name}>");

    let start_idx = xml.find(&open_tag)? + open_tag.len();
    let end_idx = xml[start_idx..].find(&close_tag)? + start_idx;

    let val = xml[start_idx..end_idx].trim();
    if val.is_empty() {
        None
    } else {
        Some(val.to_string())
    }
}

pub fn toggle_task_entry(entry: &StartupEntry) -> bool {
    let task_name = &entry.raw_id;
    let action = if entry.status == StartupStatus::Enabled {
        "/disable"
    } else {
        "/enable"
    };

    #[cfg(target_os = "windows")]
    {
        let status = std::process::Command::new("schtasks")
            .args(["/change", "/tn", task_name, action])
            .status();
        matches!(status, Ok(s) if s.success())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = (task_name, action);
        true
    }
}

pub fn delete_task_entry(entry: &StartupEntry) -> bool {
    let task_name = &entry.raw_id;

    #[cfg(target_os = "windows")]
    {
        let status = std::process::Command::new("schtasks")
            .args(["/delete", "/tn", task_name, "/f"])
            .status();
        matches!(status, Ok(s) if s.success())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = task_name;
        true
    }
}
