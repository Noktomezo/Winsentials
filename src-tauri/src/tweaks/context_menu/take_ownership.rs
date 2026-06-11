use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";
const CUSTOM_VALUE: &str = "custom";

const FILE_MENU_KEY: RegKey = RegKey {
    hive: Hive::CurrentUser,
    path: r"Software\Classes\*\shell\runas",
};

const FILE_COMMAND_KEY: RegKey = RegKey {
    hive: Hive::CurrentUser,
    path: r"Software\Classes\*\shell\runas\command",
};

const DIRECTORY_MENU_KEY: RegKey = RegKey {
    hive: Hive::CurrentUser,
    path: r"Software\Classes\Directory\shell\runas",
};

const DIRECTORY_COMMAND_KEY: RegKey = RegKey {
    hive: Hive::CurrentUser,
    path: r"Software\Classes\Directory\shell\runas\command",
};

pub struct TakeOwnershipTweak {
    meta: TweakMeta,
}

impl Default for TakeOwnershipTweak {
    fn default() -> Self {
        Self::new()
    }
}

impl TakeOwnershipTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "take_ownership_context_menu".into(),
                category: "context_menu".into(),
                name: "contextMenu.tweaks.takeOwnership.name".into(),
                short_description: "contextMenu.tweaks.takeOwnership.shortDescription".into(),
                detail_description: "contextMenu.tweaks.takeOwnership.detailDescription".into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: ENABLED_VALUE.into(),
                risk: RiskLevel::Medium,
                risk_description: Some("contextMenu.tweaks.takeOwnership.riskDescription".into()),
                conflicts: None,
                requires_action: RequiresAction::None,
                min_os_build: Some(10240),
                min_os_ubr: None,
                min_required_memory_gb: None,
            },
        }
    }

    fn get_menu_label() -> String {
        let is_russian = RegKey {
            hive: Hive::CurrentUser,
            path: r"Control Panel\International",
        }
        .get_string("LocaleName")
        .map(|val| val.to_lowercase().starts_with("ru"))
        .unwrap_or(false);

        if is_russian {
            "Получить полный доступ".to_string()
        } else {
            "Take Ownership".to_string()
        }
    }

    fn file_command() -> &'static str {
        r#"cmd.exe /c takeown /f "%1" && icacls "%1" /grant *S-1-5-32-544:F"#
    }

    fn directory_command() -> &'static str {
        r#"cmd.exe /c takeown /f "%1" /r /d y && icacls "%1" /grant *S-1-5-32-544:F /t /c"#
    }

    fn write_menu(
        menu_key: &RegKey,
        command_key: &RegKey,
        label: &str,
        command: &str,
    ) -> Result<(), AppError> {
        menu_key.set_string("", label)?;
        menu_key.set_string("HasLUAShield", "")?;
        menu_key.set_string("NoWorkingDirectory", "")?;
        command_key.set_string("", command)?;
        command_key.set_string("IsolatedCommand", command)
    }

    fn snapshot_menu(menu_key: &RegKey, command_key: &RegKey) -> Result<MenuSnapshot, AppError> {
        Ok(MenuSnapshot {
            menu_existed: menu_key.key_exists()?,
            command_existed: command_key.key_exists()?,
            label: read_string_or_missing(menu_key, "")?,
            has_lua_shield: read_string_or_missing(menu_key, "HasLUAShield")?,
            no_working_directory: read_string_or_missing(menu_key, "NoWorkingDirectory")?,
            command: read_string_or_missing(command_key, "")?,
            isolated_command: read_string_or_missing(command_key, "IsolatedCommand")?,
        })
    }

    fn rollback_menu(menu_key: &RegKey, command_key: &RegKey, snapshot: &MenuSnapshot) {
        if snapshot.command_existed {
            restore_string_value(command_key, "", snapshot.command.as_deref());
            restore_string_value(
                command_key,
                "IsolatedCommand",
                snapshot.isolated_command.as_deref(),
            );
        } else {
            let _ = command_key.delete_subkey_tree();
        }

        if snapshot.menu_existed {
            restore_string_value(menu_key, "", snapshot.label.as_deref());
            restore_string_value(menu_key, "HasLUAShield", snapshot.has_lua_shield.as_deref());
            restore_string_value(
                menu_key,
                "NoWorkingDirectory",
                snapshot.no_working_directory.as_deref(),
            );
        } else {
            let _ = menu_key.delete_subkey_tree();
        }
    }

    fn menu_matches(
        menu_key: &RegKey,
        command_key: &RegKey,
        label: &str,
        command: &str,
    ) -> Result<bool, AppError> {
        Ok(
            read_string_or_missing(menu_key, "")?.as_deref() == Some(label)
                && read_string_or_missing(command_key, "")?.as_deref() == Some(command),
        )
    }

    fn is_enabled(&self) -> Result<bool, AppError> {
        let label = Self::get_menu_label();
        Ok(Self::menu_matches(
            &FILE_MENU_KEY,
            &FILE_COMMAND_KEY,
            &label,
            Self::file_command(),
        )? && Self::menu_matches(
            &DIRECTORY_MENU_KEY,
            &DIRECTORY_COMMAND_KEY,
            &label,
            Self::directory_command(),
        )?)
    }

    fn is_default(&self) -> Result<bool, AppError> {
        Ok(!FILE_MENU_KEY.key_exists()? && !DIRECTORY_MENU_KEY.key_exists()?)
    }

    fn enable() -> Result<(), AppError> {
        let label = Self::get_menu_label();
        let file_snapshot = Self::snapshot_menu(&FILE_MENU_KEY, &FILE_COMMAND_KEY)?;
        let directory_snapshot = Self::snapshot_menu(&DIRECTORY_MENU_KEY, &DIRECTORY_COMMAND_KEY)?;

        if let Err(error) = Self::write_menu(
            &FILE_MENU_KEY,
            &FILE_COMMAND_KEY,
            &label,
            Self::file_command(),
        ) {
            Self::rollback_menu(&FILE_MENU_KEY, &FILE_COMMAND_KEY, &file_snapshot);
            return Err(error);
        }

        if let Err(error) = Self::write_menu(
            &DIRECTORY_MENU_KEY,
            &DIRECTORY_COMMAND_KEY,
            &label,
            Self::directory_command(),
        ) {
            Self::rollback_menu(
                &DIRECTORY_MENU_KEY,
                &DIRECTORY_COMMAND_KEY,
                &directory_snapshot,
            );
            Self::rollback_menu(&FILE_MENU_KEY, &FILE_COMMAND_KEY, &file_snapshot);
            return Err(error);
        }

        Ok(())
    }
}

impl Tweak for TakeOwnershipTweak {
    fn id(&self) -> &str {
        &self.meta.id
    }

    fn meta(&self) -> &TweakMeta {
        &self.meta
    }

    fn apply(&self, value: &str) -> Result<(), AppError> {
        match value {
            ENABLED_VALUE => Self::enable(),
            DISABLED_VALUE => self.reset(),
            _ => Err(AppError::message(format!(
                "unsupported value `{value}` for {}",
                self.id()
            ))),
        }
    }

    fn reset(&self) -> Result<(), AppError> {
        FILE_MENU_KEY.delete_subkey_tree()?;
        DIRECTORY_MENU_KEY.delete_subkey_tree()
    }

    fn get_status(&self) -> Result<TweakStatus, AppError> {
        let is_enabled = self.is_enabled()?;
        let is_default = self.is_default()?;

        Ok(TweakStatus {
            current_value: if is_enabled {
                ENABLED_VALUE.into()
            } else if is_default {
                DISABLED_VALUE.into()
            } else {
                CUSTOM_VALUE.into()
            },
            is_default,
        })
    }
}

struct MenuSnapshot {
    menu_existed: bool,
    command_existed: bool,
    label: Option<String>,
    has_lua_shield: Option<String>,
    no_working_directory: Option<String>,
    command: Option<String>,
    isolated_command: Option<String>,
}

fn restore_string_value(key: &RegKey, name: &str, value: Option<&str>) {
    if let Some(value) = value {
        let _ = key.set_string(name, value);
    } else {
        let _ = key.delete_value(name);
    }
}

fn read_string_or_missing(key: &RegKey, name: &str) -> Result<Option<String>, AppError> {
    match key.get_string(name) {
        Ok(value) => Ok(Some(value)),
        Err(AppError::Io(error)) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}
