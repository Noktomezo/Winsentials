use crate::error::AppError;
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, RegType::REG_BINARY};
use winreg::{RegKey as WinRegKey, RegValue};

pub enum Hive {
    CurrentUser,
    LocalMachine,
}

pub struct RegKey {
    pub hive: Hive,
    pub path: &'static str,
}

impl RegKey {
    pub fn open_read(&self) -> Result<WinRegKey, AppError> {
        self.root_handle()
            .open_subkey_with_flags(self.path, KEY_READ)
            .map_err(AppError::from)
    }

    pub fn open_write(&self) -> Result<WinRegKey, AppError> {
        self.root_handle()
            .create_subkey(self.path)
            .map(|(key, _)| key)
            .map_err(AppError::from)
    }

    pub fn get_dword(&self, name: &str) -> Result<u32, AppError> {
        let key = self.open_read()?;
        key.get_value(name).map_err(AppError::from)
    }

    pub fn set_dword(&self, name: &str, value: u32) -> Result<(), AppError> {
        let key = self.open_write()?;
        key.set_value(name, &value).map_err(AppError::from)
    }

    pub fn get_string(&self, name: &str) -> Result<String, AppError> {
        let key = self.open_read()?;
        key.get_value(name).map_err(AppError::from)
    }

    pub fn get_binary(&self, name: &str) -> Result<Vec<u8>, AppError> {
        let key = self.open_read()?;
        key.get_raw_value(name)
            .map(|value| value.bytes.into_owned())
            .map_err(AppError::from)
    }

    pub fn set_string(&self, name: &str, value: &str) -> Result<(), AppError> {
        let key = self.open_write()?;
        key.set_value(name, &value).map_err(AppError::from)
    }

    pub fn set_binary(&self, name: &str, value: &[u8]) -> Result<(), AppError> {
        let key = self.open_write()?;
        key.set_raw_value(
            name,
            &RegValue {
                bytes: value.to_vec().into(),
                vtype: REG_BINARY,
            },
        )
        .map_err(AppError::from)
    }

    pub fn key_exists(&self) -> Result<bool, AppError> {
        match self.open_read() {
            Ok(_) => Ok(true),
            Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error),
        }
    }

    pub fn delete_value(&self, name: &str) -> Result<(), AppError> {
        let key = self.open_write()?;

        match key.delete_value(name) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(AppError::from(error)),
        }
    }

    pub fn delete_subkey_tree(&self) -> Result<(), AppError> {
        match self.root_handle().delete_subkey_all(self.path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(AppError::from(error)),
        }
    }

    fn root_handle(&self) -> WinRegKey {
        match self.hive {
            Hive::CurrentUser => WinRegKey::predef(HKEY_CURRENT_USER),
            Hive::LocalMachine => WinRegKey::predef(HKEY_LOCAL_MACHINE),
        }
    }
}
