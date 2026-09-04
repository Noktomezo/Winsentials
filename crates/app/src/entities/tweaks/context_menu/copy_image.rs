use std::path::{Path, PathBuf};

use crate::shared::shell_notify::notify_shell_change;

const REG_IMAGE_MENU: &str =
    r"Software\Classes\SystemFileAssociations\image\shell\Winsentials.CopyImage";
const REG_IMAGE_COMMAND: &str =
    r"Software\Classes\SystemFileAssociations\image\shell\Winsentials.CopyImage\command";

const SCRIPT: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../assets/scripts/copy-image.ps1"
));
const LAUNCHER: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../assets/scripts/copy-image.vbs"
));

fn shell_dir() -> Result<PathBuf, String> {
    std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .map(|path| path.join("Winsentials").join("shell"))
        .ok_or_else(|| "LOCALAPPDATA is unavailable".to_string())
}

fn command(launcher: &Path) -> String {
    format!(r#"wscript.exe "{}" "%1""#, launcher.display())
}

fn remove_legacy_files() {
    if let Some(dir) = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .map(|path| path.join("Winsentials").join("scripts"))
    {
        let _ = std::fs::remove_file(dir.join("copy-image.ps1"));
    }
}

#[must_use]
pub fn is_copy_image_applied() -> bool {
    #[cfg(target_os = "windows")]
    {
        shell_dir().is_ok_and(|dir| {
            let script = dir.join("copy-image.ps1");
            let launcher = dir.join("copy-image.vbs");
            script.exists()
                && launcher.exists()
                && windows_registry::CURRENT_USER
                    .open(REG_IMAGE_COMMAND)
                    .is_ok_and(|key| {
                        key.get_string("")
                            .is_ok_and(|value| value == command(&launcher))
                    })
        })
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_copy_image(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        let dir = shell_dir()?;
        let script = dir.join("copy-image.ps1");
        let launcher = dir.join("copy-image.vbs");

        if applied {
            std::fs::create_dir_all(&dir)
                .map_err(|error| format!("Failed to create shell directory: {error}"))?;
            std::fs::write(&script, SCRIPT)
                .map_err(|error| format!("Failed to write copy-image script: {error}"))?;
            std::fs::write(&launcher, LAUNCHER)
                .map_err(|error| format!("Failed to write copy-image launcher: {error}"))?;

            let menu = windows_registry::CURRENT_USER
                .create(REG_IMAGE_MENU)
                .map_err(|error| format!("Failed to create image menu item: {error}"))?;
            menu.set_string("MUIVerb", rust_i18n::t!("tweaks.copy_image_menu_item"))
                .and_then(|()| menu.set_string("Icon", "imageres.dll,-5302"))
                .map_err(|error| format!("Failed to configure image menu item: {error}"))?;

            windows_registry::CURRENT_USER
                .create(REG_IMAGE_COMMAND)
                .and_then(|key| key.set_string("", command(&launcher)))
                .map_err(|error| format!("Failed to configure image menu command: {error}"))?;

            remove_legacy_files();
        } else {
            let _ = windows_registry::CURRENT_USER.remove_tree(REG_IMAGE_MENU);
            for path in [&script, &launcher] {
                if path.exists() {
                    let _ = std::fs::remove_file(path);
                }
            }
            remove_legacy_files();
        }

        notify_shell_change();
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = applied;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_path_is_passed_to_vbs_launcher() {
        let value = command(Path::new(
            r"C:\Users\Test User\AppData\Local\Winsentials\shell\copy-image.vbs",
        ));

        assert_eq!(
            value,
            r#"wscript.exe "C:\Users\Test User\AppData\Local\Winsentials\shell\copy-image.vbs" "%1""#
        );
    }
}
