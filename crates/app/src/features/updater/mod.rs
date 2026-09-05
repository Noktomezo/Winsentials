use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use futures_util::StreamExt;
use reqwest::header::USER_AGENT;
#[allow(unused_imports)]
use semver::Version;
use serde::{Deserialize, Serialize};

pub const MINISIGN_PUBLIC_KEY: &str = "RWSlOeLuyQXBGSoaYq+TP8gRloPlIBZ2wDXx9yQvkzie9zUBalLlZFGG";
pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const GITHUB_OWNER: &str = "Noktomezo";
pub const GITHUB_REPO: &str = "Winsentials";

pub const LATEST_JSON_URL: &str =
    "https://github.com/Noktomezo/Winsentials/releases/latest/download/latest.json";
pub const LATEST_RELEASE_WEB: &str = "https://github.com/Noktomezo/Winsentials/releases/latest";
pub const LATEST_RELEASE_API: &str =
    "https://api.github.com/repos/Noktomezo/Winsentials/releases/latest";

pub const UPDATE_POLL_INTERVAL: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub version: String,
    pub download_url: String,
    pub release_url: String,
    pub signature: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum UpdateState {
    #[default]
    Idle,
    Checking,
    UpToDate,
    UpdateAvailable(UpdateInfo),
    Downloading {
        version: String,
        progress: f32,
    },
    Installing {
        version: String,
    },
    Error(String),
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct PlatformTarget {
    signature: String,
    url: String,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct UpdateManifest {
    version: String,
    notes: Option<String>,
    platforms: HashMap<String, PlatformTarget>,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct GitHubReleaseAsset {
    name: String,
    browser_download_url: String,
}

#[allow(dead_code)]
#[derive(Deserialize)]
struct GitHubReleaseResponse {
    tag_name: String,
    html_url: String,
    assets: Vec<GitHubReleaseAsset>,
}

/// Verifies a downloaded binary payload against a Minisign signature string using the embedded public key.
pub fn verify_installer_signature(bytes: &[u8], sig_str: &str) -> Result<(), String> {
    let pk = minisign_verify::PublicKey::from_base64(MINISIGN_PUBLIC_KEY)
        .map_err(|e| format!("Invalid public key: {e}"))?;
    let sig = minisign_verify::Signature::decode(sig_str.trim())
        .map_err(|e| format!("Invalid signature format: {e}"))?;
    pk.verify(bytes, &sig, false)
        .map_err(|e| format!("Minisign signature verification failed: {e}"))?;
    Ok(())
}

/// Checks for new versions against GitHub Releases using rate-limit immune strategies:
/// 1. Direct download of `latest.json` from release assets (bypasses GitHub REST API completely).
/// 2. Web release page HTTP redirect inspection (browser-grade redirect, immune to API rate limits).
/// 3. Fallback to GitHub REST API with graceful HTTP 403 / 429 handling.
#[allow(clippy::too_many_lines)]
pub async fn check_for_update(
    client: &reqwest::Client,
    current_version_str: &str,
) -> Result<Option<UpdateInfo>, String> {
    #[cfg(all(debug_assertions, not(test)))]
    {
        let _ = (client, current_version_str);
        tokio::time::sleep(Duration::from_millis(600)).await;
        Ok(Some(UpdateInfo {
            version: "99.0.0".to_string(),
            download_url: "mock://winsentials/update/99.0.0".to_string(),
            release_url: "https://github.com/Noktomezo/Winsentials/releases/tag/v99.0.0"
                .to_string(),
            signature: None,
            notes: Some("Dev mock update 99.0.0".to_string()),
        }))
    }

    #[cfg(any(not(debug_assertions), test))]
    {
        let current_semver = Version::parse(current_version_str.trim_start_matches('v'))
            .map_err(|e| format!("Failed to parse current version '{current_version_str}': {e}"))?;

        // Strategy 1: Direct latest.json asset download (completely immune to api.github.com rate limits)
        if let Ok(resp) = client
            .get(LATEST_JSON_URL)
            .header(USER_AGENT, "Winsentials-Updater")
            .send()
            .await
        {
            if resp.status().is_success() {
                if let Ok(manifest) = resp.json::<UpdateManifest>().await {
                    if let Ok(remote_semver) =
                        Version::parse(manifest.version.trim_start_matches('v'))
                    {
                        if remote_semver > current_semver {
                            let target = manifest
                                .platforms
                                .get("windows-x86_64")
                                .or_else(|| manifest.platforms.values().next());

                            if let Some(target) = target {
                                return Ok(Some(UpdateInfo {
                                    version: manifest.version.clone(),
                                    download_url: target.url.clone(),
                                    release_url: format!(
                                        "https://github.com/{GITHUB_OWNER}/{GITHUB_REPO}/releases/tag/v{}",
                                        manifest.version
                                    ),
                                    signature: Some(target.signature.clone()),
                                    notes: manifest.notes,
                                }));
                            }
                        } else {
                            return Ok(None);
                        }
                    }
                }
            }
        }

        // Strategy 2: Web releases redirect (immune to GitHub REST API 60 req/hr limits and WARP 403)
        if let Ok(response) = client
        .get(LATEST_RELEASE_WEB)
        .header(
            USER_AGENT,
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36",
        )
        .send()
        .await
    {
        if response.status().is_success() {
            let final_url = response.url().to_string();
            if let Some(tag_part) = final_url.split("/releases/tag/").nth(1) {
                let tag = tag_part
                    .split('?')
                    .next()
                    .unwrap_or(tag_part)
                    .trim()
                    .to_string();
                let remote_tag = tag.trim_start_matches('v');
                if let Ok(remote_semver) = Version::parse(remote_tag) {
                    if remote_semver > current_semver {
                        let download_url = format!(
                            "https://github.com/{GITHUB_OWNER}/{GITHUB_REPO}/releases/download/{tag}/winsentials-win-x64-setup.exe"
                        );
                        return Ok(Some(new_version_with_tag(
                            &tag,
                            download_url,
                            final_url,
                        )));
                    }
                    return Ok(None);
                }
            }
        }
    }

        // Strategy 3: GitHub REST API (safely handling 403/429 rate limit backoff)
        let api_resp = client
            .get(LATEST_RELEASE_API)
            .header(USER_AGENT, format!("Winsentials-Updater/{CURRENT_VERSION}"))
            .send()
            .await;

        match api_resp {
            Ok(resp) => {
                if resp.status() == reqwest::StatusCode::FORBIDDEN
                    || resp.status() == reqwest::StatusCode::TOO_MANY_REQUESTS
                {
                    // Rate limit reached on GitHub REST API, safely treat as up-to-date without alarming user
                    return Ok(None);
                }
                if !resp.status().is_success() {
                    return Err(format!("GitHub API returned HTTP {}", resp.status()));
                }

                let release: GitHubReleaseResponse = resp
                    .json()
                    .await
                    .map_err(|e| format!("Failed to parse GitHub release JSON: {e}"))?;
                let remote_tag = release.tag_name.trim().trim_start_matches('v');
                if let Ok(remote_semver) = Version::parse(remote_tag) {
                    if remote_semver > current_semver {
                        let exe_asset = release.assets.iter().find(|a| {
                            a.name.ends_with("-setup.exe")
                                || std::path::Path::new(&a.name)
                                    .extension()
                                    .is_some_and(|ext| ext.eq_ignore_ascii_case("exe"))
                                || a.name.contains("installer")
                        });

                        let download_url = exe_asset.map_or_else(
                        || {
                            format!(
                                "https://github.com/{GITHUB_OWNER}/{GITHUB_REPO}/releases/download/{}/winsentials-win-x64-setup.exe",
                                release.tag_name
                            )
                        },
                        |a| a.browser_download_url.clone(),
                    );

                        return Ok(Some(UpdateInfo {
                            version: release.tag_name,
                            download_url,
                            release_url: release.html_url,
                            signature: None,
                            notes: None,
                        }));
                    }
                }
                Ok(None)
            }
            Err(err) => Err(format!("Network error checking updates: {err}")),
        }
    }
}

#[allow(dead_code)]
fn new_version_with_tag(tag: &str, download_url: String, release_url: String) -> UpdateInfo {
    UpdateInfo {
        version: tag.to_string(),
        download_url,
        release_url,
        signature: None,
        notes: None,
    }
}

/// Streams and downloads the update installer executable, validates its MZ header,
/// verifies its Minisign signature if provided, and launches the Inno Setup installer.
pub async fn download_and_install_update<F>(
    client: &reqwest::Client,
    info: &UpdateInfo,
    mut on_progress: F,
) -> Result<(), String>
where
    F: FnMut(f32) + Send + 'static,
{
    if info.download_url.starts_with("mock://") || (cfg!(debug_assertions) && !cfg!(test)) {
        let steps = 40;
        for i in 1..=steps {
            tokio::time::sleep(Duration::from_millis(60)).await;
            #[allow(clippy::cast_precision_loss)]
            let progress = i as f32 / steps as f32;
            on_progress(progress);
        }
        return Ok(());
    }

    let response = client
        .get(&info.download_url)
        .header(USER_AGENT, "Winsentials-Updater")
        .send()
        .await
        .map_err(|e| format!("Failed to request update download: {e}"))?;

    if !response.status().is_success() {
        return Err(format!(
            "HTTP {} while downloading update from {}",
            response.status(),
            info.download_url
        ));
    }

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;
    #[allow(clippy::cast_possible_truncation)]
    let mut payload = Vec::with_capacity(total_size as usize);
    let mut stream = response.bytes_stream();

    while let Some(chunk_res) = stream.next().await {
        let chunk = chunk_res.map_err(|e| format!("Download stream error: {e}"))?;
        payload.extend_from_slice(&chunk);
        downloaded += chunk.len() as u64;

        if total_size > 0 {
            #[allow(clippy::cast_precision_loss)]
            let progress = (downloaded as f32 / total_size as f32).clamp(0.0, 1.0);
            on_progress(progress);
        }
    }

    if payload.len() < 2 || &payload[..2] != b"MZ" {
        return Err("Downloaded update file is not a valid Windows executable".to_string());
    }

    // Verify Minisign signature if provided in latest.json
    if let Some(ref sig_str) = info.signature {
        verify_installer_signature(&payload, sig_str)?;
    }

    let temp_dir = std::env::temp_dir();
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis());
    let installer_path: PathBuf = temp_dir.join(format!("winsentials-setup-{timestamp}.exe"));

    std::fs::write(&installer_path, &payload)
        .map_err(|e| format!("Failed to write temp installer: {e}"))?;

    #[cfg(windows)]
    {
        // Launch Inno Setup installer
        std::process::Command::new(&installer_path).args(["/SILENT", "/CLOSEAPPLICATIONS", "/RESTARTAPPLICATIONS"])
            .spawn()
            .map_err(|e| format!("Failed to launch installer: {e}"))?;

        // Exit cleanly so Inno Setup can replace Winsentials.exe
        std::process::exit(0);
    }

    #[cfg(not(windows))]
    {
        drop(installer_path);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        let v0_9_0 = Version::parse("0.9.0").unwrap();
        let v0_9_1 = Version::parse("0.9.1").unwrap();
        let v0_10_0 = Version::parse("0.10.0").unwrap();
        assert!(v0_9_1 > v0_9_0);
        assert!(v0_10_0 > v0_9_1);
        assert!(!(v0_9_0 > v0_9_0));
    }

    #[test]
    fn test_signature_verification_format() {
        let dummy_data = b"Winsentials Test Payload";
        let invalid_sig = "untrusted comment: test\nWRONG_SIGNATURE\n";
        assert!(verify_installer_signature(dummy_data, invalid_sig).is_err());
    }
}
