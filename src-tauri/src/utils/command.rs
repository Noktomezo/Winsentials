use std::os::windows::process::CommandExt;
use std::process::Command;

const CREATE_NO_WINDOW: u32 = 0x08000000;

/// Creates a Command with hidden console window on Windows
pub fn hidden_command(program: &str) -> Command {
  let mut cmd = Command::new(program);
  cmd.creation_flags(CREATE_NO_WINDOW);
  cmd
}
