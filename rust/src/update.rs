use crate::config::{RELEASE_URL, UPDATE_CHECK_TIMEOUT_SECS, Paths, UpdateInfo};
use crate::printer::Printer;
use crate::utils::is_newer_version;
use anyhow::{Context, Result};
use colored::Colorize;
use serde_json::Value;
use std::fs;
use std::time::{Duration, SystemTime};
use tokio::time::timeout;

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

        // Get download URL
        let empty_vec: Vec<Value> = vec![];
        let assets = release_data
            .get("assets")
            .and_then(|a| a.as_array())
            .unwrap_or(&empty_vec);

        let mut download_url = None;
        let mut _checksum = None;

        for asset in assets {
            let name = asset
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("");
            if name.ends_with(".zip") || name.ends_with(".tar.gz") {
                download_url = asset
                    .get("browser_download_url")
                    .and_then(|u| u.as_str())
                    .map(|s| s.to_string());

                // Look for checksum file
                let base_name = name.rsplit('.').nth(1).unwrap_or(name);
                for asset2 in assets {
                    let name2 = asset2
                        .get("name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("");
                    if name2.contains(base_name)
                        && (name2.ends_with(".sha256") || name2.contains("checksum"))
                    {
                        if let Some(checksum_url) = asset2
                            .get("browser_download_url")
                            .and_then(|u| u.as_str())
                        {
                            if let Ok(resp) = self.client.get(checksum_url).send().await {
                                if let Ok(text) = resp.text().await {
                                    _checksum = text.split_whitespace().next().map(|s| s.to_string());
                                }
                            }
                        }
                        break;
                    }
                }
                break;
            }
        }

        // Fallback to zipball
        if download_url.is_none() {
            download_url = release_data
                .get("zipball_url")
                .and_then(|u| u.as_str())
                .map(|s| s.to_string());
        }

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
            if let Some(_url) = download_url {
                return Ok(Some(UpdateInfo {
                    new_script_path: String::new(), // Will be set during download
                    temp_dir: String::new(),
                    backup_path: String::new(),
                    version: latest_version.to_string(),
                }));
            }
        } else {
            self.update_check_timestamp();
            println!("Continuing with current version...");
        }

        Ok(None)
    }
}

