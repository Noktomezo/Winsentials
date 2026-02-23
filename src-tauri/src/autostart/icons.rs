use std::collections::HashMap;
use std::fs;
use std::io::{Cursor, Read, Write};
use std::path::PathBuf;
use std::sync::LazyLock;

use base64::Engine;
use parking_lot::RwLock;
use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Gdi::{
  BITMAP, BITMAPINFO, BITMAPINFOHEADER, DIB_RGB_COLORS, DeleteObject, GetDC,
  GetDIBits, GetObjectW, ReleaseDC,
};
use windows::Win32::Storage::FileSystem::FILE_ATTRIBUTE_NORMAL;
use windows::Win32::UI::Shell::{
  SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON, SHGetFileInfoW,
};
use windows::Win32::UI::WindowsAndMessaging::{
  DestroyIcon, GetIconInfo, HICON, ICONINFO,
};
use windows::core::PCWSTR;

static ICON_CACHE: LazyLock<RwLock<HashMap<String, String>>> =
  LazyLock::new(|| RwLock::new(HashMap::new()));

fn get_cache_dir() -> PathBuf {
  let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
  let cache_dir = home.join(".winsentials").join("icon_cache");
  let _ = fs::create_dir_all(&cache_dir);
  cache_dir
}

fn get_cache_path(key: &str) -> PathBuf {
  let safe_key: String = key
    .chars()
    .map(|c| {
      if c.is_alphanumeric() || c == '-' || c == '_' {
        c
      } else {
        '_'
      }
    })
    .collect();
  get_cache_dir().join(format!("{}.txt", safe_key))
}

fn load_from_cache(key: &str) -> Option<String> {
  if let Ok(mut file) = fs::File::open(get_cache_path(key)) {
    let mut content = String::new();
    if file.read_to_string(&mut content).is_ok() {
      return Some(content);
    }
  }
  None
}

fn save_to_cache(key: &str, data: &str) {
  if let Ok(mut file) = fs::File::create(get_cache_path(key)) {
    let _ = file.write_all(data.as_bytes());
  }
}

fn extract_icon_base64(path: &str) -> Option<String> {
  let cache_key = path.to_lowercase();

  if let Some(cached) = ICON_CACHE.read().get(&cache_key) {
    return Some(cached.clone());
  }

  if let Some(cached) = load_from_cache(&cache_key) {
    ICON_CACHE.write().insert(cache_key.clone(), cached.clone());
    return Some(cached);
  }

  let path_wide: Vec<u16> =
    path.encode_utf16().chain(std::iter::once(0)).collect();

  let mut shfi = SHFILEINFOW::default();
  let shfi_size = std::mem::size_of::<SHFILEINFOW>() as u32;

  let result = unsafe {
    SHGetFileInfoW(
      PCWSTR(path_wide.as_ptr()),
      FILE_ATTRIBUTE_NORMAL,
      Some(&mut shfi),
      shfi_size,
      SHGFI_ICON | SHGFI_LARGEICON,
    )
  };

  if result == 0 || shfi.hIcon.is_invalid() {
    return None;
  }

  let icon_data = unsafe { icon_to_base64(shfi.hIcon) };

  unsafe {
    let _ = DestroyIcon(shfi.hIcon);
  }

  if let Some(ref data) = icon_data {
    save_to_cache(&cache_key, data);
    ICON_CACHE.write().insert(cache_key, data.clone());
  }

  icon_data
}

#[deny(unsafe_op_in_unsafe_fn)]
unsafe fn icon_to_base64(icon: HICON) -> Option<String> {
  let mut icon_info = ICONINFO::default();
  if unsafe { GetIconInfo(icon, &mut icon_info) }.is_err() {
    return None;
  }

  let mut bitmap = BITMAP::default();
  unsafe {
    GetObjectW(
      icon_info.hbmColor,
      std::mem::size_of::<BITMAP>() as i32,
      Some(&mut bitmap as *mut _ as *mut _),
    );
  }

  let width = bitmap.bmWidth;
  let height = bitmap.bmHeight;

  if width <= 0 || height <= 0 {
    unsafe {
      let _ = DeleteObject(icon_info.hbmColor);
      let _ = DeleteObject(icon_info.hbmMask);
    };
    return None;
  }

  let size = match (width as usize).checked_mul(height as usize) {
    Some(s) => match s.checked_mul(4) {
      Some(s) => s,
      None => {
        unsafe {
          let _ = DeleteObject(icon_info.hbmColor);
          let _ = DeleteObject(icon_info.hbmMask);
        };
        return None;
      }
    },
    None => {
      unsafe {
        let _ = DeleteObject(icon_info.hbmColor);
        let _ = DeleteObject(icon_info.hbmMask);
      };
      return None;
    }
  };

  let mut pixels = vec![0u8; size];

  let mut bmi = BITMAPINFO {
    bmiHeader: BITMAPINFOHEADER {
      biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
      biWidth: width,
      biHeight: -height,
      biPlanes: 1,
      biBitCount: 32,
      biCompression: 0,
      ..Default::default()
    },
    ..Default::default()
  };

  let hdc = unsafe { GetDC(HWND::default()) };
  if hdc.is_invalid() {
    unsafe {
      let _ = DeleteObject(icon_info.hbmColor);
      let _ = DeleteObject(icon_info.hbmMask);
    };
    return None;
  }

  let dib_result = unsafe {
    GetDIBits(
      hdc,
      icon_info.hbmColor,
      0,
      height as u32,
      Some(pixels.as_mut_ptr() as *mut _),
      &mut bmi,
      DIB_RGB_COLORS,
    )
  };
  unsafe { ReleaseDC(HWND::default(), hdc) };

  if dib_result == 0 {
    unsafe {
      let _ = DeleteObject(icon_info.hbmColor);
      let _ = DeleteObject(icon_info.hbmMask);
    };
    return None;
  }

  unsafe {
    let _ = DeleteObject(icon_info.hbmColor);
  }
  unsafe {
    let _ = DeleteObject(icon_info.hbmMask);
  }

  for chunk in pixels.chunks_exact_mut(4) {
    chunk.swap(0, 2);
  }

  let mut png_data = Vec::new();
  {
    let mut encoder = png::Encoder::new(
      Cursor::new(&mut png_data),
      width as u32,
      height as u32,
    );
    encoder.set_color(png::ColorType::Rgba);
    let mut writer = encoder.write_header().ok()?;
    writer.write_image_data(&pixels).ok()?;
  }

  let b64 = base64::engine::general_purpose::STANDARD.encode(&png_data);
  Some(format!("data:image/png;base64,{}", b64))
}

pub fn get_icon(path: &str) -> Option<String> {
  extract_icon_base64(path)
}
