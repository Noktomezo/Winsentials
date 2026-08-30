use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
};

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "xtask",
    about = "Winsentials developer automation, packaging and patch manager"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Apply all .patch files from patches/ to cargo git checkouts
    Patch,
    /// Revert all patches and restore cargo git checkouts to clean state
    Unpatch,
    /// Export current changes from cargo git checkout to patches/0001-gpui-custom.patch
    Diff,
    /// Run full completion gate (fmt-check, check, clippy, test) with patches applied
    Gate,
    /// Run cargo check
    Check,
    /// Run cargo clippy with -D warnings
    Clippy,
    /// Format all workspace code with cargo fmt
    Fmt,
    /// Check workspace formatting without modifying files
    FmtCheck,
    /// Run all unit and integration tests
    Test,
    /// Build release executable, UPX compress, package portable ZIP and build NSIS installer
    Build,
    /// Run development server with watchexec auto-reload
    Dev {
        /// Optional arguments passed to the application
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
    /// Run the winsentials application
    Run {
        /// Optional arguments passed to the application
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Patch => apply_all_patches(),
        Commands::Unpatch => revert_all_patches(),
        Commands::Diff => export_patch_diff(),
        Commands::Gate => {
            let _ = apply_all_patches();
            run_gate()
        }
        Commands::Check => {
            let _ = apply_all_patches();
            run_cargo(&["check"])
        }
        Commands::Clippy => {
            let _ = apply_all_patches();
            run_cargo(&["clippy", "--", "-D", "warnings"])
        }
        Commands::Fmt => run_cargo(&["fmt", "--all"]),
        Commands::FmtCheck => run_cargo(&["fmt", "--all", "--check"]),
        Commands::Test => {
            let _ = apply_all_patches();
            run_cargo(&["test"])
        }
        Commands::Build => {
            let _ = apply_all_patches();
            run_build()
        }
        Commands::Dev { args } => {
            let _ = apply_all_patches();
            let mut watch_args = vec![
                "-r",
                "-e",
                "rs,hlsl,toml,json",
                "--",
                "cargo",
                "run",
                "--package",
                "winsentials",
                "--",
            ];
            let rest_refs: Vec<&str> = args.iter().map(String::as_str).collect();
            watch_args.extend(rest_refs);
            run_external("watchexec", &watch_args)
        }
        Commands::Run { args } => {
            let _ = apply_all_patches();
            let mut run_args = vec!["run", "--package", "winsentials", "--"];
            let rest_refs: Vec<&str> = args.iter().map(String::as_str).collect();
            run_args.extend(rest_refs);
            run_cargo(&run_args)
        }
    };

    if let Err(err) = result {
        eprintln!("xtask error: {err}");
        std::process::exit(1);
    }
}

fn cargo_home() -> PathBuf {
    env::var("CARGO_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            let home = env::var("USERPROFILE")
                .or_else(|_| env::var("HOME"))
                .unwrap_or_else(|_| ".".to_string());
            Path::new(&home).join(".cargo")
        })
}

fn find_zed_checkouts() -> Vec<PathBuf> {
    let checkouts_dir = cargo_home().join("git").join("checkouts");
    let mut dirs = Vec::new();
    if let Ok(entries) = fs::read_dir(&checkouts_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir()
                && path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with("zed-"))
            {
                if let Ok(sub_entries) = fs::read_dir(&path) {
                    for sub in sub_entries.flatten() {
                        let sub_path = sub.path();
                        if sub_path.is_dir() && sub_path.join("crates").join("gpui").exists() {
                            dirs.push(sub_path);
                        }
                    }
                }
            }
        }
    }
    dirs
}

fn get_patches() -> Vec<PathBuf> {
    let patches_dir = Path::new("patches");
    let mut patches = Vec::new();
    if let Ok(entries) = fs::read_dir(patches_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("patch") {
                patches.push(path);
            }
        }
    }
    patches.sort();
    patches
}

fn apply_all_patches() -> Result<(), String> {
    let patches = get_patches();
    if patches.is_empty() {
        return Ok(());
    }

    let checkouts = find_zed_checkouts();
    if checkouts.is_empty() {
        println!("No active zed git checkout found, running cargo fetch...");
        run_cargo(&["fetch"])?;
    }

    let checkouts = find_zed_checkouts();
    if checkouts.is_empty() {
        return Err("Could not locate zed checkout in cargo cache.".to_string());
    }

    for checkout in &checkouts {
        for patch in &patches {
            let current = env::current_dir().map_err(|e| e.to_string())?;
            let abs_patch = current.join(patch);
            let patch_str = abs_patch.to_string_lossy();

            // Check if already applied (reverse check passes)
            let already_applied = Command::new("git")
                .args([
                    "-C",
                    &checkout.to_string_lossy(),
                    "apply",
                    "--reverse",
                    "--check",
                    "--ignore-space-change",
                    &patch_str,
                ])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            if already_applied {
                println!(
                    "Patch {} already applied to {:?}",
                    patch.display(),
                    checkout.file_name().unwrap_or_default()
                );
                continue;
            }

            let can_apply = Command::new("git")
                .args([
                    "-C",
                    &checkout.to_string_lossy(),
                    "apply",
                    "--check",
                    "--ignore-space-change",
                    &patch_str,
                ])
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);

            if !can_apply {
                continue;
            }

            println!(
                "Applying patch {} to {:?}",
                patch.display(),
                checkout.file_name().unwrap_or_default()
            );

            let status = Command::new("git")
                .args([
                    "-C",
                    &checkout.to_string_lossy(),
                    "apply",
                    "--whitespace=nowarn",
                    "--ignore-space-change",
                    &patch_str,
                ])
                .status()
                .map_err(|e| format!("failed to run git apply: {e}"))?;

            if !status.success() {
                eprintln!(
                    "Warning: Failed to apply patch {} to {:?}",
                    patch.display(),
                    checkout
                );
            }
        }
    }

    Ok(())
}

fn revert_all_patches() -> Result<(), String> {
    let checkouts = find_zed_checkouts();
    if checkouts.is_empty() {
        println!("No zed git checkouts found.");
        return Ok(());
    }

    for checkout in &checkouts {
        println!("Reverting patches in {:?}", checkout);
        let status = Command::new("git")
            .args(["-C", &checkout.to_string_lossy(), "checkout", "."])
            .status()
            .map_err(|e| format!("failed to run git checkout: {e}"))?;

        if !status.success() {
            return Err(format!("failed to revert in {:?}", checkout));
        }

        let _ = Command::new("git")
            .args(["-C", &checkout.to_string_lossy(), "clean", "-fd"])
            .status();
    }

    println!("All patches reverted successfully.");
    Ok(())
}

fn export_patch_diff() -> Result<(), String> {
    let checkouts = find_zed_checkouts();
    let Some(checkout) = checkouts.first() else {
        return Err("No zed git checkout found to generate diff from.".to_string());
    };

    let output = Command::new("git")
        .args(["-C", &checkout.to_string_lossy(), "diff"])
        .output()
        .map_err(|e| format!("failed to run git diff: {e}"))?;

    if !output.status.success() {
        return Err("git diff failed".to_string());
    }

    if output.stdout.is_empty() {
        println!("No modifications found in {:?}.", checkout);
        return Ok(());
    }

    let patch_path = Path::new("patches").join("0001-gpui-custom.patch");
    fs::write(&patch_path, &output.stdout)
        .map_err(|e| format!("failed to write patch file: {e}"))?;

    println!("Exported diff to {}", patch_path.display());
    Ok(())
}

fn run_gate() -> Result<(), String> {
    println!("=== [1/4] Running fmt check ===");
    run_cargo(&["fmt", "--all", "--check"])?;

    println!("=== [2/4] Running cargo check ===");
    run_cargo(&["check"])?;

    println!("=== [3/4] Running cargo clippy ===");
    run_cargo(&["clippy", "--", "-D", "warnings"])?;

    println!("=== [4/4] Running cargo test ===");
    run_cargo(&["test"])?;

    println!("=== All checks passed successfully! ===");
    Ok(())
}

fn package_portable_zip(binary_path: &Path, zip_path: &Path) -> Result<(), String> {
    let file = fs::File::create(zip_path).map_err(|e| format!("Failed to create zip file: {e}"))?;
    let mut zip = zip::ZipWriter::new(file);

    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    zip.start_file("Winsentials.exe", options)
        .map_err(|e| format!("Failed to start file in zip: {e}"))?;

    let binary_bytes = fs::read(binary_path).map_err(|e| format!("Failed to read binary: {e}"))?;
    std::io::Write::write_all(&mut zip, &binary_bytes)
        .map_err(|e| format!("Failed to write binary to zip: {e}"))?;

    zip.finish()
        .map_err(|e| format!("Failed to finish zip: {e}"))?;
    Ok(())
}

fn convert_png_to_bmp_on_the_fly(
    src_png: &Path,
    dst_bmp: &Path,
    bg_color: [u8; 3],
) -> Result<(), String> {
    if !src_png.exists() {
        return Ok(());
    }

    let img = image::open(src_png)
        .map_err(|e| format!("Failed to open PNG {}: {e}", src_png.display()))?;
    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    let mut rgb_img = image::RgbImage::new(width, height);

    for (x, y, pixel) in rgba.enumerate_pixels() {
        let [r, g, b, a] = pixel.0;
        let alpha = f32::from(a) / 255.0;
        let inv_alpha = 1.0 - alpha;

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let final_r = (f32::from(r) * alpha + f32::from(bg_color[0]) * inv_alpha).round() as u8;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let final_g = (f32::from(g) * alpha + f32::from(bg_color[1]) * inv_alpha).round() as u8;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let final_b = (f32::from(b) * alpha + f32::from(bg_color[2]) * inv_alpha).round() as u8;

        rgb_img.put_pixel(x, y, image::Rgb([final_r, final_g, final_b]));
    }

    rgb_img
        .save_with_format(dst_bmp, image::ImageFormat::Bmp)
        .map_err(|e| format!("Failed to save BMP {}: {e}", dst_bmp.display()))?;

    Ok(())
}

fn convert_png_to_bmp_resized(
    src_png: &Path,
    dst_bmp: &Path,
    target_width: u32,
    target_height: u32,
    bg_color: [u8; 3],
) -> Result<(), String> {
    if !src_png.exists() {
        return Ok(());
    }

    let img = image::open(src_png)
        .map_err(|e| format!("Failed to open PNG {}: {e}", src_png.display()))?;
    let resized = img.resize_exact(
        target_width,
        target_height,
        image::imageops::FilterType::Lanczos3,
    );
    let rgba = resized.to_rgba8();
    let (width, height) = rgba.dimensions();
    let mut rgb_img = image::RgbImage::new(width, height);

    for (x, y, pixel) in rgba.enumerate_pixels() {
        let [r, g, b, a] = pixel.0;
        let alpha = f32::from(a) / 255.0;
        let inv_alpha = 1.0 - alpha;

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let final_r = (f32::from(r) * alpha + f32::from(bg_color[0]) * inv_alpha).round() as u8;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let final_g = (f32::from(g) * alpha + f32::from(bg_color[1]) * inv_alpha).round() as u8;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let final_b = (f32::from(b) * alpha + f32::from(bg_color[2]) * inv_alpha).round() as u8;

        rgb_img.put_pixel(x, y, image::Rgb([final_r, final_g, final_b]));
    }

    rgb_img
        .save_with_format(dst_bmp, image::ImageFormat::Bmp)
        .map_err(|e| format!("Failed to save BMP {}: {e}", dst_bmp.display()))?;

    Ok(())
}

fn prepare_installer_bmps() -> Result<(), String> {
    let assets = Path::new("assets");
    let logo_png = assets.join("app-logo.png");
    let small_bmp = assets.join("app-installer-small.bmp");
    let header_png = assets.join("app-installer-header.png");
    let header_bmp = assets.join("app-installer-header.bmp");
    let sidebar_png = assets.join("app-installer-sidebar.png");
    let sidebar_bmp = assets.join("app-installer-sidebar.bmp");

    // 1. WizardSmallImage (58x58 crisp square app logo)
    convert_png_to_bmp_resized(&logo_png, &small_bmp, 58, 58, [15, 21, 26])?;
    // 2. Header banner for NSIS
    convert_png_to_bmp_on_the_fly(&header_png, &header_bmp, [15, 21, 26])?;
    // 3. WizardImage sidebar
    convert_png_to_bmp_on_the_fly(&sidebar_png, &sidebar_bmp, [15, 21, 26])?;

    Ok(())
}

fn build_installer(version: &str, output_path: &Path) -> Result<(), String> {
    prepare_installer_bmps()?;

    let iss_script = Path::new("tooling").join("packaging").join("installer.iss");
    if iss_script.exists() {
        let version_flag = format!("/DMyAppVersion={version}");
        let out_dir_flag = format!(
            "/DMyOutputDir=..\\..\\{}",
            output_path
                .parent()
                .unwrap_or(Path::new("."))
                .to_string_lossy()
        );
        let file_stem = output_path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy();
        let out_name_flag = format!("/DMyOutputBaseFilename={file_stem}");

        println!("=== Building Inno Setup installer ===");
        if let Ok(()) = run_external(
            "iscc",
            &[
                &version_flag,
                &out_dir_flag,
                &out_name_flag,
                "/Q",
                &iss_script.to_string_lossy(),
            ],
        ) {
            return Ok(());
        }
    }

    let nsi_script = Path::new("tooling").join("packaging").join("installer.nsi");
    if nsi_script.exists() {
        let version_flag = format!("-DVERSION={version}");
        let out_flag = format!("-DOUTFILE=..\\..\\{}", output_path.to_string_lossy());

        println!("=== Building NSIS installer (fallback) ===");
        let makensis = "makensis";
        run_external(
            makensis,
            &[&version_flag, &out_flag, &nsi_script.to_string_lossy()],
        )?;
    }
    Ok(())
}

fn run_build() -> Result<(), String> {
    println!("=== Building Winsentials release binary ===");
    run_cargo(&["build", "--release", "--package", "winsentials"])?;

    let release_dir = Path::new("target").join("release");
    let binary = release_dir.join(format!("Winsentials{}", env::consts::EXE_SUFFIX));
    if !binary.exists() {
        return Err(format!("Binary not found at {}", binary.display()));
    }

    println!("=== Compressing binary with UPX ===");
    if let Err(e) = run_external("upx", &["--best", "--lzma", &binary.to_string_lossy()]) {
        eprintln!("Warning: UPX compression skipped or failed: {e}");
    }

    let version = "0.1.0";

    println!("=== Packaging portable ZIP ===");
    let portable_zip_path = release_dir.join("winsentials-win-x64-portable.zip");
    package_portable_zip(&binary, &portable_zip_path)?;

    let setup_exe_path = release_dir.join("winsentials-win-x64-setup.exe");
    build_installer(version, &setup_exe_path)?;

    println!("\n=== Release distribution artifacts ready in target/release/ ===");
    let artifacts = [
        ("Winsentials.exe", &binary),
        ("winsentials-win-x64-portable.zip", &portable_zip_path),
        ("winsentials-win-x64-setup.exe", &setup_exe_path),
    ];

    for (name, path) in &artifacts {
        if let Ok(meta) = fs::metadata(path) {
            let size_mb = meta.len() as f64 / (1024.0 * 1024.0);
            println!("  - {:<36} ({:.2} MB)", name, size_mb);
        }
    }

    Ok(())
}

fn run_cargo(args: &[&str]) -> Result<(), String> {
    let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".to_string());
    run_external(&cargo, args)
}

fn run_external(program: &str, args: &[&str]) -> Result<(), String> {
    let status: ExitStatus = Command::new(program)
        .args(args)
        .status()
        .map_err(|e| format!("failed to execute {program}: {e}"))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "{program} command exited with non-zero status: {:?}",
            status.code()
        ))
    }
}
