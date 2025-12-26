use crate::config::Paths;
use crate::printer::Printer;
use anyhow::{Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhitelistEntry {
    pub indicator: String,
    pub indicator_type: String, // "domain", "ip", "url", "hash"
    pub added_date: String,
    pub reason: Option<String>,
}

pub struct WhitelistManager;

impl WhitelistManager {
    fn get_whitelist_path(paths: &Paths) -> PathBuf {
        paths.base_dir.join("whitelist.json")
    }

    /// Load whitelist from file
    pub fn load_whitelist(paths: &Paths) -> Result<Vec<WhitelistEntry>> {
        let whitelist_path = Self::get_whitelist_path(paths);
        
        if !whitelist_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&whitelist_path)
            .with_context(|| format!("Failed to read whitelist file: {}", whitelist_path.display()))?;
        
        let entries: Vec<WhitelistEntry> = serde_json::from_str(&content)
            .with_context(|| "Failed to parse whitelist file")?;
        
        Ok(entries)
    }

    /// Save whitelist to file
    fn save_whitelist(paths: &Paths, entries: &[WhitelistEntry]) -> Result<()> {
        let whitelist_path = Self::get_whitelist_path(paths);
        
        // Ensure directory exists
        if let Some(parent) = whitelist_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
        }

        let content = serde_json::to_string_pretty(entries)
            .context("Failed to serialize whitelist")?;
        
        fs::write(&whitelist_path, content)
            .with_context(|| format!("Failed to write whitelist file: {}", whitelist_path.display()))?;
        
        Ok(())
    }

    /// Detect indicator type (domain, IP, URL, hash)
    fn detect_indicator_type(indicator: &str) -> String {
        let trimmed = indicator.trim().to_lowercase();
        
        // Check for URL
        if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            return "url".to_string();
        }
        
        // Check for IP address (simple check)
        if trimmed.matches('.').count() == 3 {
            let parts: Vec<&str> = trimmed.split('.').collect();
            if parts.len() == 4 && parts.iter().all(|p| p.parse::<u8>().is_ok()) {
                return "ip".to_string();
            }
        }
        
        // Check for hash (MD5, SHA1, SHA256)
        if trimmed.len() == 32 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
            return "hash".to_string();
        }
        if trimmed.len() == 40 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
            return "hash".to_string();
        }
        if trimmed.len() == 64 && trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
            return "hash".to_string();
        }
        
        // Default to domain
        "domain".to_string()
    }

    /// Add an indicator to the whitelist
    pub fn add_indicator(
        paths: &Paths,
        indicator: &str,
        reason: Option<String>,
    ) -> Result<()> {
        let mut entries = Self::load_whitelist(paths)?;
        
        let trimmed_indicator = indicator.trim().to_lowercase();
        
        // Check if already whitelisted
        if entries.iter().any(|e| e.indicator.to_lowercase() == trimmed_indicator) {
            return Err(anyhow::anyhow!("Indicator '{}' is already whitelisted", indicator));
        }
        
        let indicator_type = Self::detect_indicator_type(&trimmed_indicator);
        let added_date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        
        let entry = WhitelistEntry {
            indicator: trimmed_indicator.clone(),
            indicator_type,
            added_date,
            reason,
        };
        
        entries.push(entry);
        Self::save_whitelist(paths, &entries)?;
        
        Printer::success(&format!("Added '{}' to whitelist", indicator));
        Ok(())
    }

    /// Remove an indicator from the whitelist
    pub fn remove_indicator(paths: &Paths, indicator: &str) -> Result<()> {
        let mut entries = Self::load_whitelist(paths)?;
        
        let trimmed_indicator = indicator.trim().to_lowercase();
        let initial_len = entries.len();
        
        entries.retain(|e| e.indicator.to_lowercase() != trimmed_indicator);
        
        if entries.len() == initial_len {
            return Err(anyhow::anyhow!("Indicator '{}' not found in whitelist", indicator));
        }
        
        Self::save_whitelist(paths, &entries)?;
        
        Printer::success(&format!("Removed '{}' from whitelist", indicator));
        Ok(())
    }

    /// List all whitelisted indicators
    pub fn list_indicators(paths: &Paths) -> Result<()> {
        let entries = Self::load_whitelist(paths)?;
        
        if entries.is_empty() {
            Printer::info("Whitelist is empty");
            return Ok(());
        }
        
        println!("\n{}", "Whitelisted Indicators:".bold().cyan());
        println!("{}", "=".repeat(80).cyan());
        
        for (idx, entry) in entries.iter().enumerate() {
            println!("\n{}. {}", (idx + 1).to_string().yellow().bold(), entry.indicator.bold());
            println!("   Type: {}", entry.indicator_type.cyan());
            println!("   Added: {}", entry.added_date.yellow());
            if let Some(ref reason) = entry.reason {
                println!("   Reason: {}", reason.magenta());
            }
        }
        
        println!("\n{}", "=".repeat(80).cyan());
        println!("Total: {} indicator(s)", entries.len().to_string().green().bold());
        println!();
        
        Ok(())
    }

    /// Get all whitelisted indicators as a HashSet for fast lookup
    /// Reserved for future use: integrating whitelist filtering into report generation
    #[allow(dead_code)]
    pub fn get_whitelist_set(paths: &Paths) -> Result<HashSet<String>> {
        let entries = Self::load_whitelist(paths)?;
        Ok(entries.iter().map(|e| e.indicator.to_lowercase()).collect())
    }

    /// Check if an indicator is whitelisted
    /// Reserved for future use: integrating whitelist filtering into report generation
    #[allow(dead_code)]
    pub fn is_whitelisted(paths: &Paths, indicator: &str) -> Result<bool> {
        let whitelist = Self::get_whitelist_set(paths)?;
        Ok(whitelist.contains(&indicator.trim().to_lowercase()))
    }

    /// Filter indicators by removing whitelisted ones
    /// Reserved for future use: integrating whitelist filtering into report generation
    #[allow(dead_code)]
    pub fn filter_indicators(paths: &Paths, indicators: &[String]) -> Result<Vec<String>> {
        let whitelist = Self::get_whitelist_set(paths)?;
        Ok(indicators
            .iter()
            .filter(|indicator| !whitelist.contains(&indicator.trim().to_lowercase()))
            .cloned()
            .collect())
    }
}

