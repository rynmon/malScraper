use crate::config::{RELEASE_URL, UPDATE_CHECK_TIMEOUT_SECS, Paths, UpdateInfo};
use crate::printer::Printer;
use crate::utils::is_newer_version;
use anyhow::{Context, Result};
use colored::Colorize;
use serde_json::Value;
use std::fs;
use std::io::Write;
use std::time::{Duration, SystemTime};
use tokio::time::timeout;
use futures::StreamExt;
use tokio::io::AsyncWriteExt;

pub struct UpdateChecker {
    client: reqwest::Client,
    paths: Paths,
}

impl UpdateChecker {
    pub fn new(paths: Paths) -> Self {
        // GitHub API requires User-Agent header
        let user_agent = format!("malScraper/{} (https://github.com/rynmon/malScraper)", 
            crate::config::CURRENT_VERSION);
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(UPDATE_CHECK_TIMEOUT_SECS))
            .user_agent(&user_agent)
            .build()
            .expect("Failed to create HTTP client");

        Self { client, paths }
    }

    pub fn should_check_for_updates(&self) -> bool {
        let last_check_file = self.paths.updates_dir.join(".last_update_check");

        if !last_check_file.exists() {
            return true;
        }

        if let Ok(metadata) = fs::metadata(&last_check_file) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(elapsed) = SystemTime::now().duration_since(modified) {
                    // Check once per day
                    return elapsed.as_secs() >= 86400;
                }
            }
        }

        true
    }

    fn update_check_timestamp(&self) {
        let last_check_file = self.paths.updates_dir.join(".last_update_check");
        let _ = fs::File::create(last_check_file);
    }

    pub async fn check_for_updates(
        &self,
        current_version: &str,
        force: bool,
    ) -> Result<Option<UpdateInfo>> {
        if !force && !self.should_check_for_updates() {
            return Ok(None);
        }

        Printer::info("Checking for updates...");

        let response = timeout(
            Duration::from_secs(UPDATE_CHECK_TIMEOUT_SECS),
            self.client.get(RELEASE_URL).send(),
        )
        .await
        .context("Update check timed out")?
        .context("Failed to send request")?;

        // Check HTTP status
        let status = response.status();
        if !status.is_success() {
            // Try to get error message from response body
            let error_text = response.text().await.unwrap_or_else(|_| "No error details".to_string());
            let error_msg = format!(
                "GitHub API returned error {} {}: {}",
                status.as_u16(),
                status.canonical_reason().unwrap_or("Unknown"),
                error_text
            );
            return Err(anyhow::anyhow!(error_msg));
        }

        // Get response text first for better error messages
        let response_text = response.text().await.context("Failed to read response body")?;
        
        let release_data: Value = serde_json::from_str(&response_text)
            .context(format!("Failed to parse release data as JSON. Response: {}", 
                if response_text.len() > 200 { 
                    format!("{}...", &response_text[..200]) 
                } else { 
                    response_text.clone() 
                }))?;

        let latest_version = release_data
            .get("tag_name")
            .and_then(|v| v.as_str())
            .map(|s| s.trim_start_matches('v').to_string())
            .unwrap_or_else(|| "0.0.0".to_string());

        if !is_newer_version(current_version, &latest_version) {
            Printer::success(&format!("Running latest version: {}", current_version));
            self.update_check_timestamp();
            return Ok(None);
        }

        println!("\n{}", "New version available!".yellow().bold());
        Printer::info(&format!("Current version: {}", current_version));
        Printer::info(&format!("Latest version: {}", latest_version));

        // Get assets for platform-specific binary detection
        let empty_vec: Vec<Value> = vec![];
        let assets = release_data
            .get("assets")
            .and_then(|a| a.as_array())
            .unwrap_or(&empty_vec);

        // Display release notes
        if let Some(body) = release_data.get("body").and_then(|b| b.as_str()) {
            Printer::info("\nRelease notes:");
            for line in body.lines().take(5) {
                println!("  {}", line);
            }
            if body.lines().count() > 5 {
                Printer::warning("  ...and more");
            }
        }

        // Prompt user
        use dialoguer::Confirm;
        if Confirm::new()
            .with_prompt("Would you like to update now?")
            .default(true)
            .interact()
            .unwrap_or(false)
        {
            self.update_check_timestamp();
            // Find the correct binary for this platform
            let platform_binary = self.find_platform_binary(&assets)?;
            if let Some(binary_url) = platform_binary {
                return Ok(Some(UpdateInfo {
                    new_script_path: String::new(), // Will be set during download
                    temp_dir: String::new(),
                    backup_path: String::new(),
                    version: latest_version.to_string(),
                    download_url: binary_url,
                }));
            } else {
                Printer::warning("Could not find a binary for your platform. Please download manually from:");
                Printer::info("https://github.com/rynmon/malScraper/releases");
                return Ok(None);
            }
        } else {
            self.update_check_timestamp();
            println!("Continuing with current version...");
        }

        Ok(None)
    }

    fn find_platform_binary(&self, assets: &[Value]) -> Result<Option<String>> {
        let (target_os, target_arch) = self.detect_platform();
        
        Printer::info(&format!("Detected platform: {} {}", target_arch, target_os));
        
        // Build possible binary name patterns
        let patterns = match (target_os.as_str(), target_arch.as_str()) {
            ("windows", "x86_64") => vec!["x86_64-pc-windows-msvc.exe", "windows-x86_64.exe"],
            ("linux", "x86_64") => vec!["x86_64-unknown-linux-gnu", "linux-x86_64"],
            ("linux", "aarch64") => vec!["aarch64-unknown-linux-gnu", "linux-aarch64"],
            ("macos", "x86_64") | ("darwin", "x86_64") => vec!["x86_64-apple-darwin", "macos-x86_64"],
            ("macos", "aarch64") | ("darwin", "aarch64") => vec!["aarch64-apple-darwin", "macos-aarch64"],
            _ => vec![],
        };
        
        for asset in assets {
            if let Some(name) = asset.get("name").and_then(|n| n.as_str()) {
                for pattern in &patterns {
                    if name.contains(pattern) && !name.ends_with(".sha256") && !name.contains("checksum") {
                        if let Some(url) = asset.get("browser_download_url").and_then(|u| u.as_str()) {
                            Printer::info(&format!("Found binary: {}", name));
                            return Ok(Some(url.to_string()));
                        }
                    }
                }
            }
        }
        
        Ok(None)
    }
    
    fn detect_platform(&self) -> (String, String) {
        let os = if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") || cfg!(target_os = "ios") {
            "macos"
        } else if cfg!(target_os = "linux") {
            "linux"
        } else {
            "unknown"
        };
        
        let arch = if cfg!(target_arch = "x86_64") {
            "x86_64"
        } else if cfg!(target_arch = "aarch64") {
            "aarch64"
        } else {
            "unknown"
        };
        
        (os.to_string(), arch.to_string())
    }

    pub async fn install_update(&self, update_info: UpdateInfo) -> Result<()> {
        Printer::info("Downloading update...");
        
        // Get current executable path
        let current_exe = std::env::current_exe()
            .context("Failed to get current executable path")?;
        
        // Create backup
        let backup_path = current_exe.with_extension(format!("{}.backup", 
            crate::config::CURRENT_VERSION));
        Printer::info(&format!("Creating backup: {}", backup_path.display()));
        std::fs::copy(&current_exe, &backup_path)
            .context("Failed to create backup")?;
        
        // Create temp directory
        let temp_dir = self.paths.updates_dir.join(format!("update-{}", update_info.version));
        std::fs::create_dir_all(&temp_dir)
            .context("Failed to create temp directory")?;
        
        // Download the binary
        let download_path = temp_dir.join(if cfg!(target_os = "windows") {
            "malscraper-new.exe"
        } else {
            "malscraper-new"
        });
        
        let response = self.client.get(&update_info.download_url).send().await
            .context("Failed to download update")?;
        
        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Download failed with status: {}", response.status()));
        }
        
        let total_size = response.content_length().unwrap_or(0);
        let mut file = tokio::fs::File::create(&download_path).await
            .context("Failed to create download file")?;
        
        let mut downloaded = 0u64;
        let mut stream = response.bytes_stream();
        
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Failed to read chunk")?;
            file.write_all(&chunk).await
                .context("Failed to write chunk")?;
            downloaded += chunk.len() as u64;
            
            if total_size > 0 {
                let percent = (downloaded * 100) / total_size;
                print!("\rDownloading: {}% ({}/{})", percent, downloaded, total_size);
                std::io::stdout().flush().ok();
            }
        }
        println!();
        
        // Make executable on Unix systems
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&download_path)
                .context("Failed to get file metadata")?
                .permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&download_path, perms)
                .context("Failed to set executable permissions")?;
        }
        
        Printer::success("Download complete!");
        
        // Replace the current binary
        Printer::info("Installing update...");
        
        // On Windows, we need to rename the old file first
        #[cfg(windows)]
        {
            let old_exe = current_exe.with_extension("exe.old");
            if old_exe.exists() {
                std::fs::remove_file(&old_exe).ok();
            }
            std::fs::rename(&current_exe, &old_exe)
                .context("Failed to rename current executable")?;
            std::fs::copy(&download_path, &current_exe)
                .context("Failed to copy new executable")?;
            std::fs::remove_file(&old_exe).ok();
        }
        
        #[cfg(unix)]
        {
            std::fs::copy(&download_path, &current_exe)
                .context("Failed to copy new executable")?;
        }
        
        // Clean up temp directory
        std::fs::remove_dir_all(&temp_dir).ok();
        
        Printer::success(&format!("Update installed successfully! Version {}", update_info.version));
        Printer::info("Please restart the application to use the new version.");
        
        Ok(())
    }
}

