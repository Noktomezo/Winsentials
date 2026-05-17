fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    ensure_sidecar_placeholder();
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

fn ensure_sidecar_placeholder() {
    let target_triple = std::env::var("TAURI_ENV_TARGET_TRIPLE")
        .unwrap_or_else(|_| "x86_64-pc-windows-msvc".into());
    let sidecar_dir = std::path::Path::new("binaries");
    let sidecar_path = sidecar_dir.join(format!("winsentials_symlink_helper-{target_triple}.exe"));

    if sidecar_path.exists() {
        return;
    }

    let _ = std::fs::create_dir_all(sidecar_dir);
    let _ = std::fs::write(sidecar_path, []);
}
