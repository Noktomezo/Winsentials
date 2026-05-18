fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=TAURI_ENV_TARGET_TRIPLE");
    ensure_sidecar_placeholder().expect("failed to prepare sidecar placeholder");
    let windows = tauri_build::WindowsAttributes::new().app_manifest(
        r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <dependency>
    <dependentAssembly>
      <assemblyIdentity
        type="win32"
        name="Microsoft.Windows.Common-Controls"
        version="6.0.0.0"
        processorArchitecture="*"
        publicKeyToken="6595b64144ccf1df"
        language="*"
      />
    </dependentAssembly>
  </dependency>
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="requireAdministrator" uiAccess="false" />
      </requestedPrivileges>
    </security>
  </trustInfo>
</assembly>
"#,
    );

    tauri_build::try_build(tauri_build::Attributes::new().windows_attributes(windows))
        .expect("failed to run tauri build script");
}

fn ensure_sidecar_placeholder() -> Result<(), std::io::Error> {
    let target_triple = std::env::var("TAURI_ENV_TARGET_TRIPLE").unwrap_or_else(|_| {
        let arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_else(|_| "x86_64".into());
        let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_else(|_| "windows".into());
        let env = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_else(|_| "msvc".into());
        format!("{arch}-pc-{os}-{env}")
    });
    if !target_triple
        .chars()
        .all(|value| value.is_ascii_alphanumeric() || matches!(value, '.' | '_' | '-'))
        || target_triple.contains("..")
    {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("invalid TAURI_ENV_TARGET_TRIPLE `{target_triple}`"),
        ));
    }
    let sidecar_dir = std::path::Path::new("binaries");
    let sidecar_path = sidecar_dir.join(format!("winsentials_symlink_helper-{target_triple}.exe"));

    if sidecar_path.exists() {
        return Ok(());
    }

    std::fs::create_dir_all(sidecar_dir)?;
    std::fs::write(sidecar_path, [])?;
    Ok(())
}
