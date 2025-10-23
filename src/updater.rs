use log::info;
use serde::Deserialize;
use std::error::Error;
use std::fs;
use std::io::Write;

const REPO_OWNER: &str = "raghulj";
const REPO_NAME: &str = "lookout";

#[derive(Deserialize)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
}

#[derive(Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
}

/// Check if a new version is available on GitHub
pub async fn check_for_updates() -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
    let current_version = env!("CARGO_PKG_VERSION");

    info!("Current version: {}", current_version);
    info!("Checking for updates from GitHub...");

    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        REPO_OWNER, REPO_NAME
    );

    // Use async client
    let client = reqwest::Client::builder()
        .user_agent("lookout-update-checker")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| -> Box<dyn Error + Send + Sync> { Box::new(e) })?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| -> Box<dyn Error + Send + Sync> { Box::new(e) })?;

    if !response.status().is_success() {
        return Err(format!("GitHub API returned status: {}", response.status()).into());
    }

    let release: GitHubRelease = response
        .json()
        .await
        .map_err(|e| -> Box<dyn Error + Send + Sync> { Box::new(e) })?;
    let latest_version = release.tag_name.trim_start_matches('v');

    info!("Latest version available: {}", latest_version);

    if latest_version != current_version {
        info!(
            "New version available: {} -> {}",
            current_version, latest_version
        );
        return Ok(Some(latest_version.to_string()));
    }

    info!("Already on latest version");
    Ok(None)
}

/// Perform the self-update process
pub async fn perform_update() -> Result<String, Box<dyn Error + Send + Sync>> {
    info!("Starting self-update process...");

    let url = format!(
        "https://api.github.com/repos/{}/{}/releases/latest",
        REPO_OWNER, REPO_NAME
    );

    // Use async client
    let client = reqwest::Client::builder()
        .user_agent("lookout-update-checker")
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| -> Box<dyn Error + Send + Sync> { Box::new(e) })?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| -> Box<dyn Error + Send + Sync> { Box::new(e) })?;
    let release: GitHubRelease = response
        .json()
        .await
        .map_err(|e| -> Box<dyn Error + Send + Sync> { Box::new(e) })?;

    // Find the binary asset for Linux
    let asset = release
        .assets
        .iter()
        .find(|a| a.name == "lookout" || a.name == "lookout-linux")
        .ok_or("No suitable binary found in release")?;

    info!("Downloading update from: {}", asset.browser_download_url);

    // Download the new binary
    let binary_response = client
        .get(&asset.browser_download_url)
        .send()
        .await
        .map_err(|e| -> Box<dyn Error + Send + Sync> { Box::new(e) })?;
    let binary_data = binary_response
        .bytes()
        .await
        .map_err(|e| -> Box<dyn Error + Send + Sync> { Box::new(e) })?;

    // Get current executable path
    let current_exe = std::env::current_exe()?;
    let temp_new = current_exe.with_extension("new");

    // Write new binary to temp location
    let mut file = fs::File::create(&temp_new)?;
    file.write_all(&binary_data)?;
    drop(file);

    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&temp_new)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&temp_new, perms)?;
    }

    // Create update script that will run after we exit
    let script_path = current_exe.with_extension("update.sh");
    let script_content = format!(
        r#"#!/bin/bash
sleep 1
mv "{current}" "{current}.bak"
mv "{new}" "{current}"
chmod +x "{current}"
"{current}" &
rm -- "$0"
"#,
        current = current_exe.display(),
        new = temp_new.display()
    );

    let mut script_file = fs::File::create(&script_path)?;
    script_file.write_all(script_content.as_bytes())?;
    drop(script_file);

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&script_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&script_path, perms)?;
    }

    info!("Update prepared successfully");
    Ok(release.tag_name.trim_start_matches('v').to_string())
}

/// Check if the app can update itself (not installed via package manager)
pub fn can_self_update() -> bool {
    // Check if running from a system location that would require root
    if let Ok(exe_path) = std::env::current_exe() {
        let path_str = exe_path.to_string_lossy();

        // If installed via package manager locations, disable self-update
        if path_str.starts_with("/usr/bin")
            || path_str.starts_with("/usr/local/bin")
            || path_str.starts_with("/snap")
            || path_str.starts_with("/flatpak")
        {
            info!("Self-update disabled: installed via package manager");
            return false;
        }

        // Check if we have write permission to the executable
        if let Ok(metadata) = std::fs::metadata(&exe_path) {
            if metadata.permissions().readonly() {
                info!("Self-update disabled: no write permission");
                return false;
            }
        }
    }

    true
}

/// Restart the application after update
pub fn restart_application() -> Result<(), Box<dyn Error + Send + Sync>> {
    let exe_path = std::env::current_exe()?;
    let script_path = exe_path.with_extension("update.sh");

    info!("Executing update script and exiting: {:?}", script_path);

    // Launch the update script in the background
    std::process::Command::new("sh")
        .arg(&script_path)
        .spawn()?;

    // Exit current process so the script can replace the binary
    std::process::exit(0);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_self_update() {
        // This will vary depending on where tests are run from
        let can_update = can_self_update();
        println!("Can self-update: {}", can_update);
    }
}
