    use super::*;

    #[test]
    #[cfg(target_os = "windows")]
    fn test_extract_explorer_icon() {
        let icon = resolve_entry_icon(Some("C:\\Windows\\explorer.exe"), None);
        assert!(icon.is_some());
        let path = icon.unwrap();
        assert!(Path::new(&path).exists());
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_extract_powershell_script_icon() {
        let icon = resolve_entry_icon(
            Some(r"C:\ProgramData\Winhance\OpenWebSearch\OpenWebSearchRepair.ps1"),
            Some(
                r#"powershell.exe -ExecutionPolicy Bypass -NoProfile -Command "iex([IO.File]::ReadAllText('C:\ProgramData\Winhance\OpenWebSearch\OpenWebSearchRepair.ps1'))""#,
            ),
        );
        assert!(icon.is_some());
        let path = icon.unwrap();
        assert!(Path::new(&path).exists());
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_extract_amnezia_service_icon() {
        let amnezia_path = r"C:\Program Files\AmneziaVPN\AmneziaVPN-service.exe";
        if Path::new(amnezia_path).exists() {
            let icon = resolve_entry_icon(Some(amnezia_path), None);
            assert!(icon.is_some());
        }
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_extract_cncmd_sibling_icon() {
        let cncmd_path = r"C:\Program Files\AMD\CNext\CNext\cncmd.exe";
        if Path::new(cncmd_path).exists() {
            let icon = resolve_entry_icon(Some(cncmd_path), None);
            assert!(icon.is_some());
        }
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_resolve_shortcut_sharex() {
        let appdata = std::env::var("APPDATA").unwrap();
        let lnk = PathBuf::from(appdata)
            .join(r"Microsoft\Windows\Start Menu\Programs\Startup\ShareX.lnk");
        if lnk.exists() {
            let target = resolve_shortcut(&lnk);
            assert!(target.is_some());
            let t = target.unwrap();
            assert!(t.to_string_lossy().to_lowercase().contains("sharex.exe"));
        }
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_resolve_shortcut_waves() {
        let progdata = std::env::var("PROGRAMDATA").unwrap();
        let lnk = PathBuf::from(progdata)
            .join(r"Microsoft\Windows\Start Menu\Programs\Startup\WavesLocalServer.lnk");
        if lnk.exists() {
            let target = resolve_shortcut(&lnk);
            assert!(target.is_some());
            let t = target.unwrap();
            assert!(
                t.to_string_lossy()
                    .to_lowercase()
                    .contains("waveslocalserver.exe")
            );
        }
    }
