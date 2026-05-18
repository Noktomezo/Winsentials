#![windows_subsystem = "windows"]

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use windows::Win32::Foundation::{ERROR_CANCELLED, GetLastError};
use windows::Win32::Storage::FileSystem::{
    CreateSymbolicLinkW, SYMBOLIC_LINK_FLAG_ALLOW_UNPRIVILEGED_CREATE, SYMBOLIC_LINK_FLAG_DIRECTORY,
};
use windows::Win32::System::Com::{
    CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
    CoTaskMemFree, CoUninitialize,
};
use windows::Win32::UI::Shell::{
    FOS_FORCEFILESYSTEM, FOS_PICKFOLDERS, FileOpenDialog, IFileDialog, SIGDN_FILESYSPATH,
};
use windows::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MessageBoxW};
use windows::core::PCWSTR;

fn main() {
    if let Err(error) = run() {
        show_error(&error);
    }
}

fn run() -> Result<(), String> {
    let source = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or_else(|| "Не передан файл или папка для символической ссылки.".to_string())?;

    if !source.exists() {
        return Err(format!("Исходный путь не найден:\n{}", source.display()));
    }

    let Some(target_dir) = pick_target_directory()? else {
        return Ok(());
    };

    let link_path = available_link_path(&target_dir, &source)?;
    create_symbolic_link(&link_path, &source)?;

    Ok(())
}

fn pick_target_directory() -> Result<Option<PathBuf>, String> {
    let _com = ComApartment::init()?;

    let dialog: IFileDialog = unsafe {
        CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)
            .map_err(|error| format!("Не удалось открыть выбор папки: {error}"))?
    };

    unsafe {
        let options = dialog
            .GetOptions()
            .map_err(|error| format!("Не удалось прочитать настройки выбора папки: {error}"))?;
        dialog
            .SetOptions(options | FOS_PICKFOLDERS | FOS_FORCEFILESYSTEM)
            .map_err(|error| format!("Не удалось настроить выбор папки: {error}"))?;
        let title = wide("Выберите папку для символической ссылки");
        dialog
            .SetTitle(PCWSTR(title.as_ptr()))
            .map_err(|error| format!("Не удалось настроить заголовок окна: {error}"))?;

        if let Err(error) = dialog.Show(None) {
            if error.code() == windows::core::HRESULT::from_win32(ERROR_CANCELLED.0) {
                return Ok(None);
            }

            return Err(format!("Выбор папки не выполнен: {error}"));
        }

        let item = dialog
            .GetResult()
            .map_err(|error| format!("Не удалось получить выбранную папку: {error}"))?;
        let path_ptr = item
            .GetDisplayName(SIGDN_FILESYSPATH)
            .map_err(|error| format!("Не удалось прочитать путь выбранной папки: {error}"))?;
        let path = path_ptr
            .to_string()
            .map(PathBuf::from)
            .map_err(|error| format!("Не удалось декодировать путь выбранной папки: {error}"));

        if !path_ptr.is_null() {
            CoTaskMemFree(Some(path_ptr.0 as _));
        }

        path.map(Some)
    }
}

fn available_link_path(target_dir: &Path, source: &Path) -> Result<PathBuf, String> {
    let file_name = source
        .file_name()
        .ok_or_else(|| "Не удалось определить имя исходного файла или папки.".to_string())?;
    let candidate = target_dir.join(file_name);

    if !candidate.exists() {
        return Ok(candidate);
    }

    let stem = source
        .file_stem()
        .and_then(OsStr::to_str)
        .filter(|value| !value.is_empty())
        .unwrap_or("Link");
    let extension = source.extension().and_then(OsStr::to_str);

    for index in 1..1000 {
        let suffix = if index == 1 {
            " - symlink".to_string()
        } else {
            format!(" - symlink ({index})")
        };
        let name = match extension {
            Some(extension) => format!("{stem}{suffix}.{extension}"),
            None => format!("{stem}{suffix}"),
        };
        let candidate = target_dir.join(name);

        if !candidate.exists() {
            return Ok(candidate);
        }
    }

    Err("Не удалось подобрать свободное имя для символической ссылки.".to_string())
}

fn create_symbolic_link(link_path: &Path, source: &Path) -> Result<(), String> {
    let mut flags = SYMBOLIC_LINK_FLAG_ALLOW_UNPRIVILEGED_CREATE;
    if source.is_dir() {
        flags |= SYMBOLIC_LINK_FLAG_DIRECTORY;
    }

    let link_wide = wide_path(link_path);
    let source_wide = wide_path(source);
    let created = unsafe {
        CreateSymbolicLinkW(
            PCWSTR(link_wide.as_ptr()),
            PCWSTR(source_wide.as_ptr()),
            flags,
        )
    };

    if created {
        Ok(())
    } else {
        let error = unsafe { GetLastError() };
        Err(format!(
            "Не удалось создать символическую ссылку.\n\nИсточник:\n{}\n\nСсылка:\n{}\n\nОшибка Windows: {}.\n\nВключите Developer Mode или запустите действие с правами администратора.",
            source.display(),
            link_path.display(),
            error.0
        ))
    }
}

fn show_error(message: &str) {
    let title = wide("Winsentials");
    let message = wide(message);

    unsafe {
        MessageBoxW(
            None,
            PCWSTR(message.as_ptr()),
            PCWSTR(title.as_ptr()),
            MB_OK | MB_ICONERROR,
        );
    }
}

fn wide(value: &str) -> Vec<u16> {
    OsStr::new(value).encode_wide().chain([0]).collect()
}

fn wide_path(path: &Path) -> Vec<u16> {
    path.as_os_str().encode_wide().chain([0]).collect()
}

struct ComApartment;

impl ComApartment {
    fn init() -> Result<Self, String> {
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED)
                .ok()
                .map_err(|error| format!("Не удалось инициализировать COM: {error}"))?;
        }

        Ok(Self)
    }
}

impl Drop for ComApartment {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}
