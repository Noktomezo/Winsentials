use crate::error::AppError;
use crate::registry::{Hive, RegKey};
use crate::tweaks::{RequiresAction, RiskLevel, Tweak, TweakControlType, TweakMeta, TweakStatus};

const ENABLED_VALUE: &str = "enabled";
const DISABLED_VALUE: &str = "disabled";
const CUSTOM_VALUE: &str = "custom";

const MENU_LABEL: &str = "Create symbolic link";
const MENU_ICON: &str = "shell32.dll,147";

const FILE_MENU_KEY: RegKey = RegKey {
    hive: Hive::CurrentUser,
    path: r"Software\Classes\*\shell\Winsentials.CreateSymbolicLink",
};

const FILE_COMMAND_KEY: RegKey = RegKey {
    hive: Hive::CurrentUser,
    path: r"Software\Classes\*\shell\Winsentials.CreateSymbolicLink\command",
};

const DIRECTORY_MENU_KEY: RegKey = RegKey {
    hive: Hive::CurrentUser,
    path: r"Software\Classes\Directory\shell\Winsentials.CreateSymbolicLink",
};

const DIRECTORY_COMMAND_KEY: RegKey = RegKey {
    hive: Hive::CurrentUser,
    path: r"Software\Classes\Directory\shell\Winsentials.CreateSymbolicLink\command",
};

pub struct CreateSymbolicLinkContextMenuTweak {
    meta: TweakMeta,
}

impl Default for CreateSymbolicLinkContextMenuTweak {
    fn default() -> Self {
        Self::new()
    }
}
impl CreateSymbolicLinkContextMenuTweak {
    pub fn new() -> Self {
        Self {
            meta: TweakMeta {
                id: "create_symbolic_link_context_menu".into(),
                category: "context_menu".into(),
                name: "contextMenu.tweaks.createSymbolicLink.name".into(),
                short_description: "contextMenu.tweaks.createSymbolicLink.shortDescription".into(),
                detail_description: "contextMenu.tweaks.createSymbolicLink.detailDescription"
                    .into(),
                control: TweakControlType::Toggle,
                current_value: DISABLED_VALUE.into(),
                default_value: DISABLED_VALUE.into(),
                recommended_value: ENABLED_VALUE.into(),
                risk: RiskLevel::Low,
                risk_description: Some(
                    "contextMenu.tweaks.createSymbolicLink.riskDescription".into(),
                ),
                conflicts: None,
                requires_action: RequiresAction::None,
                min_os_build: Some(10240),
                min_os_ubr: None,
                min_required_memory_gb: None,
            },
        }
    }

    fn command_value() -> String {
        r#"cmd /c echo @echo off>"%temp%\Symlinker_%random%.cmd" & echo set "myCommand="(new-object -COM 'Shell.Application')^^.BrowseForFolder(0,'',0).self.path"">>"%temp%\Symlinker_%random%.cmd" & echo for /f "usebackq delims=" %%%%I in (`powershell %%myCommand%%`) do set "dir=%%%%I">>"%temp%\Symlinker_%random%.cmd" & echo if not defined dir (del /f /q "%%~f0" ^& exit)>>"%temp%\Symlinker_%random%.cmd" & echo setlocal enabledelayedexpansion>>"%temp%\Symlinker_%random%.cmd" & echo mklink "!dir!\%%~nx1" %%1>>"%temp%\Symlinker_%random%.cmd" & echo del /f /q "%%~f0">>"%temp%\Symlinker_%random%.cmd" & start /min "" "%temp%\Symlinker_%random%.cmd" "%1" ^& exit"#.into()
    }

    fn write_menu(menu_key: &RegKey, command_key: &RegKey, command: &str) -> Result<(), AppError> {
        menu_key.set_string("", MENU_LABEL)?;
        menu_key.set_string("Icon", MENU_ICON)?;
        menu_key.set_string("HasLUAShield", "")?;
        command_key.set_string("", command)
    }

    fn snapshot_menu(menu_key: &RegKey, command_key: &RegKey) -> Result<MenuSnapshot, AppError> {
        Ok(MenuSnapshot {
            menu_existed: menu_key.key_exists()?,
            command_existed: command_key.key_exists()?,
            label: read_string_or_missing(menu_key, "")?,
            icon: read_string_or_missing(menu_key, "Icon")?,
            has_lua_shield: read_string_or_missing(menu_key, "HasLUAShield")?,
            command: read_string_or_missing(command_key, "")?,
        })
    }

    fn rollback_menu(menu_key: &RegKey, command_key: &RegKey, snapshot: &MenuSnapshot) {
        if snapshot.command_existed {
            restore_string_value(command_key, "", snapshot.command.as_deref());
        } else {
            let _ = command_key.delete_subkey_tree();
        }

        if snapshot.menu_existed {
            restore_string_value(menu_key, "", snapshot.label.as_deref());
            restore_string_value(menu_key, "Icon", snapshot.icon.as_deref());
            restore_string_value(menu_key, "HasLUAShield", snapshot.has_lua_shield.as_deref());
        } else {
            let _ = menu_key.delete_subkey_tree();
        }
    }

    fn menu_matches(
        menu_key: &RegKey,
        command_key: &RegKey,
        command: &str,
    ) -> Result<bool, AppError> {
        Ok(
            read_string_or_missing(menu_key, "")?.as_deref() == Some(MENU_LABEL)
                && read_string_or_missing(menu_key, "Icon")?.as_deref() == Some(MENU_ICON)
                && read_string_or_missing(menu_key, "HasLUAShield")?.is_some()
                && read_string_or_missing(command_key, "")?.as_deref() == Some(command),
        )
    }

    fn is_enabled(&self) -> Result<bool, AppError> {
        let command = Self::command_value();

        Ok(
            Self::menu_matches(&FILE_MENU_KEY, &FILE_COMMAND_KEY, &command)?
                && Self::menu_matches(&DIRECTORY_MENU_KEY, &DIRECTORY_COMMAND_KEY, &command)?,
        )
    }

    fn is_default(&self) -> Result<bool, AppError> {
        Ok(!FILE_MENU_KEY.key_exists()? && !DIRECTORY_MENU_KEY.key_exists()?)
    }

    fn enable() -> Result<(), AppError> {
        let command = Self::command_value();
        let file_snapshot = Self::snapshot_menu(&FILE_MENU_KEY, &FILE_COMMAND_KEY)?;
        let directory_snapshot = Self::snapshot_menu(&DIRECTORY_MENU_KEY, &DIRECTORY_COMMAND_KEY)?;

        if let Err(error) = Self::write_menu(&FILE_MENU_KEY, &FILE_COMMAND_KEY, &command) {
            Self::rollback_menu(&FILE_MENU_KEY, &FILE_COMMAND_KEY, &file_snapshot);
            return Err(error);
        }

        if let Err(error) = Self::write_menu(&DIRECTORY_MENU_KEY, &DIRECTORY_COMMAND_KEY, &command)
        {
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

impl Tweak for CreateSymbolicLinkContextMenuTweak {
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
    icon: Option<String>,
    has_lua_shield: Option<String>,
    command: Option<String>,
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
