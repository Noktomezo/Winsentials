use std::path::Path;

fn create_multi_res_ico(
    source_png_path: &Path,
    target_ico_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let img = image::open(source_png_path)?;
    let sizes = [16u32, 24, 32, 48, 64, 128, 256];
    let mut png_buffers = Vec::new();

    for &size in &sizes {
        let resized = img.resize_exact(size, size, image::imageops::FilterType::Lanczos3);
        let mut buf = std::io::Cursor::new(Vec::new());
        resized.write_to(&mut buf, image::ImageFormat::Png)?;
        png_buffers.push((size, buf.into_inner()));
    }

    let count = u16::try_from(png_buffers.len())?;
    let mut ico_data = Vec::new();

    // 1. ICO Header (6 bytes)
    ico_data.extend_from_slice(&[0, 0]); // Reserved
    ico_data.extend_from_slice(&[1, 0]); // Type 1 = Icon
    ico_data.extend_from_slice(&count.to_le_bytes()); // Image count

    // Header size = 6 + count * 16
    let mut offset = 6 + u32::from(count) * 16;

    // 2. Directory Entries
    for (size, buf) in &png_buffers {
        let width_byte = if *size >= 256 {
            0u8
        } else {
            u8::try_from(*size).unwrap_or(0)
        };
        let height_byte = if *size >= 256 {
            0u8
        } else {
            u8::try_from(*size).unwrap_or(0)
        };
        let data_size = u32::try_from(buf.len())?;

        ico_data.push(width_byte);
        ico_data.push(height_byte);
        ico_data.push(0); // Color count
        ico_data.push(0); // Reserved
        ico_data.extend_from_slice(&1u16.to_le_bytes()); // Color planes
        ico_data.extend_from_slice(&32u16.to_le_bytes()); // Bit depth
        ico_data.extend_from_slice(&data_size.to_le_bytes());
        ico_data.extend_from_slice(&offset.to_le_bytes());

        offset += data_size;
    }

    // 3. Image Payloads
    for (_, buf) in png_buffers {
        ico_data.extend_from_slice(&buf);
    }

    std::fs::write(target_ico_path, ico_data)?;
    Ok(())
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/app-logo.png");
    println!("cargo:rerun-if-changed=assets/app-logo-dev.png");
    println!("cargo:rerun-if-env-changed=PROFILE");

    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let is_debug = profile == "debug";

    let (source_png, target_ico, app_name) = if is_debug {
        (
            Path::new("assets/app-logo-dev.png"),
            Path::new("assets/app-logo-dev.ico"),
            "Winsentials (Dev)",
        )
    } else {
        (
            Path::new("assets/app-logo.png"),
            Path::new("assets/app-logo.ico"),
            "Winsentials",
        )
    };

    if source_png.exists() {
        if let Err(e) = create_multi_res_ico(source_png, target_ico) {
            eprintln!("Warning: failed to generate multi-resolution ICO: {e}");
        }
    }

    #[cfg(windows)]
    {
        let mut res = winres::WindowsResource::new();
        if target_ico.exists() {
            if let Some(ico_str) = target_ico.to_str() {
                res.set_icon(ico_str);
            }
        }
        res.set("FileDescription", app_name);
        res.set("ProductName", app_name);
        res.set("OriginalFilename", "Winsentials.exe");
        res.set("InternalName", "Winsentials");
        res.set("CompanyName", "Noktomezo");
        res.set("LegalCopyright", "Copyright (C) 2026 Noktomezo");
        res.set("Comments", "https://github.com/Noktomezo/Winsentials");
        res.set_language(0x0409); // U.S. English (standard for Windows PE metadata)

        if let Err(e) = res.compile() {
            eprintln!("Warning: failed to compile Windows resources: {e}");
        }
    }
}
