use std::path::Path;

pub const SHELL_HOSTS: &[&str] = &[
    "powershell.exe",
    "pwsh.exe",
    "cmd.exe",
    "wscript.exe",
    "cscript.exe",
    "mshta.exe",
];

pub const POWERSHELL_HOSTS: &[&str] = &["powershell.exe", "pwsh.exe"];
pub const COMMAND_HOSTS: &[&str] = &["cmd.exe"];
pub const SCRIPT_HOSTS: &[&str] = &["wscript.exe", "cscript.exe", "mshta.exe"];

pub fn is_shell_host_name(name: &str) -> bool {
    SHELL_HOSTS
        .iter()
        .any(|host| host.eq_ignore_ascii_case(name))
}

pub fn is_shell_host_path(path: &str) -> bool {
    Path::new(path)
        .file_name()
        .and_then(|value| value.to_str())
        .map(is_shell_host_name)
        .unwrap_or(false)
}
