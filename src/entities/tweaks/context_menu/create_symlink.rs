const REG_STAR_SHELL: &str = r"Software\Classes\*\shell\CreateSymlink";
const REG_STAR_CMD: &str = r"Software\Classes\*\shell\CreateSymlink\command";
const REG_DIR_SHELL: &str = r"Software\Classes\Directory\shell\CreateSymlink";
const REG_DIR_CMD: &str = r"Software\Classes\Directory\shell\CreateSymlink\command";

const SYMLINK_VBS_PATH: &str = r"C:\Windows\Symlink.vbs";
const SYMLINK_VBS_CONTENT: &str = r#"Option Explicit
Dim objArgs, objShell, objFSO, srcPath, objFolder, dstFolder, srcName, cmd, isDir, dstPath
Set objArgs = WScript.Arguments
If objArgs.Count = 0 Then WScript.Quit
srcPath = objArgs(0)
Set objFSO = CreateObject("Scripting.FileSystemObject")
Set objShell = CreateObject("Shell.Application")
If Not objFSO.FileExists(srcPath) And Not objFSO.FolderExists(srcPath) Then WScript.Quit
Set objFolder = objShell.BrowseForFolder(0, "Выберите папку для создания символической ссылки:", &H0001 + &H0040 + &H0010)
If objFolder Is Nothing Then WScript.Quit
dstFolder = objFolder.Self.Path
If dstFolder = "" Then WScript.Quit
srcName = objFSO.GetFileName(srcPath)
isDir = objFSO.FolderExists(srcPath)
dstPath = objFSO.BuildPath(dstFolder, srcName)
If isDir Then
    cmd = "/c mklink /d """ & dstPath & """ """ & srcPath & """"
Else
    cmd = "/c mklink """ & dstPath & """ """ & srcPath & """"
End If
objShell.ShellExecute "cmd.exe", cmd, "", "", 0
"#;

#[must_use]
pub fn is_create_symlink_applied() -> bool {
    #[cfg(target_os = "windows")]
    {
        windows_registry::CURRENT_USER.open(REG_STAR_CMD).is_ok()
            && windows_registry::CURRENT_USER.open(REG_DIR_CMD).is_ok()
            && std::path::Path::new(SYMLINK_VBS_PATH).exists()
    }
    #[cfg(not(target_os = "windows"))]
    {
        false
    }
}

pub fn set_create_symlink(applied: bool) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        if applied {
            let mut vbs_bytes = vec![0xFF, 0xFE];
            for u in SYMLINK_VBS_CONTENT.encode_utf16() {
                vbs_bytes.extend_from_slice(&u.to_le_bytes());
            }
            let _ = std::fs::write(SYMLINK_VBS_PATH, vbs_bytes);

            let label = rust_i18n::t!("tweaks.create_symlink_menu_item").to_string();
            let cmd_str = format!(r#"wscript.exe "{SYMLINK_VBS_PATH}" "%1""#);

            // 1. Files (*)
            let star_key = windows_registry::CURRENT_USER
                .create(REG_STAR_SHELL)
                .map_err(|e| format!("Failed to create * shell key: {e}"))?;
            let _ = star_key.set_string("", &label);
            let _ = star_key.set_string("Icon", "shell32.dll,264");

            let star_cmd_key = windows_registry::CURRENT_USER
                .create(REG_STAR_CMD)
                .map_err(|e| format!("Failed to create * command key: {e}"))?;
            let _ = star_cmd_key.set_string("", &cmd_str);

            // 2. Directories (Directory)
            let dir_key = windows_registry::CURRENT_USER
                .create(REG_DIR_SHELL)
                .map_err(|e| format!("Failed to create Directory shell key: {e}"))?;
            let _ = dir_key.set_string("", &label);
            let _ = dir_key.set_string("Icon", "shell32.dll,264");

            let dir_cmd_key = windows_registry::CURRENT_USER
                .create(REG_DIR_CMD)
                .map_err(|e| format!("Failed to create Directory command key: {e}"))?;
            let _ = dir_cmd_key.set_string("", &cmd_str);
        } else {
            let _ = windows_registry::CURRENT_USER.remove_tree(REG_STAR_SHELL);
            let _ = windows_registry::CURRENT_USER.remove_tree(REG_DIR_SHELL);
            let _ = std::fs::remove_file(SYMLINK_VBS_PATH);
        }

        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = applied;
        Ok(())
    }
}
