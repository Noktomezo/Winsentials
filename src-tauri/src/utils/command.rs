#[cfg(windows)]
use std::os::windows::process::CommandExt;
use std::process::Command;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Creates a Command with hidden console window on Windows
#[cfg(windows)]
pub fn hidden_command(program: &str) -> Command {
  let mut cmd = Command::new(program);
  cmd.creation_flags(CREATE_NO_WINDOW);
  cmd
}

#[cfg(not(windows))]
pub fn hidden_command(program: &str) -> Command {
  Command::new(program)
}
