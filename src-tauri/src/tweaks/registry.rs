use std::io;
use std::process::Command;
use winreg::enums::*;
use winreg::{RegKey, RegValue, HKEY};

pub fn read_reg_string(hive: HKEY, path: &str, name: &str) -> Option<String> {
  let root = RegKey::predef(hive);
  root.open_subkey(path).ok()?.get_value(name).ok()
}

pub fn read_reg_u32(hive: HKEY, path: &str, name: &str) -> Option<u32> {
  let root = RegKey::predef(hive);
  root.open_subkey(path).ok()?.get_value(name).ok()
}

pub fn read_reg_binary(hive: HKEY, path: &str, name: &str) -> Option<Vec<u8>> {
  let root = RegKey::predef(hive);
  let key = root.open_subkey(path).ok()?;
  let value: RegValue = key.get_raw_value(name).ok()?;
  Some(value.bytes)
}

pub fn write_reg_string(
  hive: HKEY,
  path: &str,
  name: &str,
  value: &str,
) -> io::Result<()> {
  let root = RegKey::predef(hive);
  let (key, _) = root.create_subkey_with_flags(path, KEY_WRITE)?;
  key.set_value(name, &value)
}

pub fn write_reg_u32(
  hive: HKEY,
  path: &str,
  name: &str,
  value: u32,
) -> io::Result<()> {
  let root = RegKey::predef(hive);
  let (key, _) = root.create_subkey_with_flags(path, KEY_WRITE)?;
  key.set_value(name, &value)
}

pub fn write_reg_binary(
  hive: HKEY,
  path: &str,
  name: &str,
  value: &[u8],
) -> io::Result<()> {
  let root = RegKey::predef(hive);
  let (key, _) = root.create_subkey_with_flags(path, KEY_WRITE)?;
  let reg_value = RegValue {
    vtype: REG_BINARY,
    bytes: value.to_vec(),
  };
  key.set_raw_value(name, &reg_value)
}

pub fn delete_reg_value(hive: HKEY, path: &str, name: &str) -> io::Result<()> {
  let root = RegKey::predef(hive);
  let key = root.open_subkey_with_flags(path, KEY_WRITE)?;
  key.delete_value(name)
}

pub fn delete_reg_key(hive: HKEY, path: &str) -> io::Result<()> {
  let root = RegKey::predef(hive);
  let (parent_path, key_name) = path.rsplit_once('\\').unwrap_or((path, ""));
  if key_name.is_empty() {
    return Err(io::Error::new(io::ErrorKind::InvalidInput, "Invalid path"));
  }
  let parent = root.open_subkey_with_flags(parent_path, KEY_WRITE)?;
  parent.delete_subkey_all(key_name)
}

pub fn restart_explorer() {
  let _ = Command::new("taskkill")
    .args(["/f", "/im", "explorer.exe"])
    .status();
  std::thread::sleep(std::time::Duration::from_millis(700));
  let _ = Command::new("explorer.exe").spawn();
}
