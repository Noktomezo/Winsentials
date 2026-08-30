use std::fs;
use std::path::{Path, PathBuf};

use super::types::{StartupEntry, StartupScope, StartupSource, StartupStatus};
use super::vendor::{clean_display_name, extract_clean_exe_path, get_file_metadata};

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
            let path_str = path.to_string_lossy();
            // Skip Microsoft Windows core OS scheduled tasks
            if path_str.ends_with(r"\Microsoft\Windows")
                || path_str.ends_with(r"\Microsoft\XblGameSave")
                || path_str.ends_with(r"\Microsoft\Windows Defender")
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

fn read_task_file_content(file_path: &Path) -> Option<String> {
    let bytes = fs::read(file_path).ok()?;
    if bytes.is_empty() {
        return None;
    }

    // UTF-16 LE with BOM (0xFF, 0xFE)
    if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE {
        let u16_slice: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();
        return Some(String::from_utf16_lossy(&u16_slice));
    }

    // UTF-16 BE with BOM (0xFE, 0xFF)
    if bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] == 0xFF {
        let u16_slice: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
            .collect();
        return Some(String::from_utf16_lossy(&u16_slice));
    }

    // Try UTF-8
    if let Ok(s) = String::from_utf8(bytes.clone()) {
        return Some(s);
    }

    // Fallback: UTF-16 LE without BOM
    let u16_slice: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();
    let s = String::from_utf16_lossy(&u16_slice);
    if s.contains("<Task") { Some(s) } else { None }
}

fn parse_task_file(root: &Path, file_path: &Path) -> Option<StartupEntry> {
    let content = read_task_file_content(file_path)?;
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

    // Extract target executable or target dll
    let mut target_exe = extract_clean_exe_path(&command);
    if let Some(ref args) = arguments {
        let lower_cmd = command.to_ascii_lowercase();
        if lower_cmd.contains("rundll32")
            || lower_cmd.contains("powershell")
            || lower_cmd.contains("cmd.exe")
        {
            if let Some(arg_target) = extract_clean_exe_path(args) {
                target_exe = Some(arg_target);
            }
        }
    }

    let target_str = target_exe.as_ref().map(|p| p.to_string_lossy().to_string());

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

    let display_name = clean_display_name(file_stem, target_exe.as_deref());

    let (pe_publisher, _) = target_exe
        .as_deref()
        .map_or((None, None), get_file_metadata);

    let publisher = pe_publisher.or_else(|| {
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
        icon_path: None,
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
