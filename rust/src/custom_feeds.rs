use crate::config::Paths;
use crate::printer::Printer;
use anyhow::{Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomFeed {
    pub name: String,
    pub url: String,
    pub added_date: i64,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomFeeds {
    pub feeds: Vec<CustomFeed>,
}

pub struct CustomFeedManager;

impl CustomFeedManager {
    fn get_feeds_path(paths: &Paths) -> std::path::PathBuf {
        paths.base_dir.join("custom_feeds.json")
    }

    pub fn load_feeds(paths: &Paths) -> Result<CustomFeeds> {
        let feeds_path = Self::get_feeds_path(paths);
        
        if !feeds_path.exists() {
            return Ok(CustomFeeds {
                feeds: Vec::new(),
            });
        }

        let content = fs::read_to_string(&feeds_path)
            .context("Failed to read custom feeds file")?;
        
        let feeds: CustomFeeds = serde_json::from_str(&content)
            .context("Failed to parse custom feeds file")?;
        
        Ok(feeds)
    }

    fn save_feeds(paths: &Paths, feeds: &CustomFeeds) -> Result<()> {
        let feeds_path = Self::get_feeds_path(paths);
        let json = serde_json::to_string_pretty(feeds)
            .context("Failed to serialize custom feeds")?;
        fs::write(&feeds_path, json)
            .context("Failed to write custom feeds file")?;
        Ok(())
    }

    pub fn add_feed(paths: &Paths, url: &str, name: Option<&str>, description: Option<String>) -> Result<()> {
        let mut feeds = Self::load_feeds(paths)?;

        // Validate URL
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(anyhow::anyhow!("URL must start with http:// or https://"));
        }

        // Check if feed already exists
        if feeds.feeds.iter().any(|f| f.url == url) {
            return Err(anyhow::anyhow!("Feed with this URL already exists"));
        }

        // Generate name from URL if not provided
        let feed_name = if let Some(n) = name {
            n.to_string()
        } else {
            // Extract domain from URL as default name
            url.replace("https://", "")
                .replace("http://", "")
                .split('/')
                .next()
                .unwrap_or("Custom Feed")
                .to_string()
        };

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let feed = CustomFeed {
            name: feed_name,
            url: url.to_string(),
            added_date: timestamp,
            description,
        };

        feeds.feeds.push(feed);
        Self::save_feeds(paths, &feeds)?;

        Ok(())
    }

    pub fn list_feeds(paths: &Paths) -> Result<()> {
        let feeds = Self::load_feeds(paths)?;

        crate::utils::clear_screen();
        Printer::header("CUSTOM FEEDS", " :: ");
        println!();

        if feeds.feeds.is_empty() {
            Printer::warning("No custom feeds configured.");
            Printer::info("Use 'FEEDS ADD <url>' to add a custom feed.");
            println!();
            return Ok(());
        }

        println!("{}", "FEEDS".cyan().bold());
        println!();

        for (idx, feed) in feeds.feeds.iter().enumerate() {
            println!("  {}. {}", (idx + 1).to_string().cyan().bold(), feed.name.cyan());
            println!("     URL: {}", feed.url.yellow());
            if let Some(ref desc) = feed.description {
                println!("     Description: {}", desc);
            }
            
            let dt = chrono::DateTime::<chrono::Utc>::from_timestamp_secs(feed.added_date)
                .unwrap_or_else(|| chrono::Utc::now());
            let date_str = dt.format("%Y-%m-%d %H:%M:%S UTC").to_string();
            println!("     Added: {}", date_str.cyan());
            println!();
        }

        println!("{}", "SUMMARY".cyan().bold());
        println!("  Total Feeds: {}", feeds.feeds.len().to_string().yellow());
        println!();

        Ok(())
    }

    pub fn remove_feed(paths: &Paths, name_or_url: &str) -> Result<()> {
        let mut feeds = Self::load_feeds(paths)?;

        let initial_len = feeds.feeds.len();
        feeds.feeds.retain(|f| f.name != name_or_url && f.url != name_or_url);

        if feeds.feeds.len() == initial_len {
            return Err(anyhow::anyhow!("Feed not found: {}", name_or_url));
        }

        Self::save_feeds(paths, &feeds)?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_all_feeds(paths: &Paths) -> Result<HashMap<String, String>> {
        let feeds = Self::load_feeds(paths)?;
        let mut result = HashMap::new();

        for feed in feeds.feeds {
            result.insert(feed.name.clone(), feed.url);
        }

        Ok(result)
    }
}

