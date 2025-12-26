use crate::config::Paths;
use crate::printer::Printer;
use anyhow::{Context, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSnapshot {
    pub report_name: String,
    pub indicators: HashSet<String>,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanHistory {
    pub snapshots: Vec<ReportSnapshot>,
    pub last_scan_time: i64,
}

pub struct History;

impl History {
    fn get_history_path(paths: &Paths) -> std::path::PathBuf {
        paths.base_dir.join("scan_history.json")
    }

    pub fn load_history(paths: &Paths) -> Result<ScanHistory> {
        let history_path = Self::get_history_path(paths);
        
        if !history_path.exists() {
            return Ok(ScanHistory {
                snapshots: Vec::new(),
                last_scan_time: 0,
            });
        }

        let content = std::fs::read_to_string(&history_path)
            .context("Failed to read history file")?;
        
        let history: ScanHistory = serde_json::from_str(&content)
            .context("Failed to parse history file")?;
        
        Ok(history)
    }

    fn save_history(paths: &Paths, history: &ScanHistory) -> Result<()> {
        let history_path = Self::get_history_path(paths);
        let json = serde_json::to_string_pretty(history)
            .context("Failed to serialize history")?;
        std::fs::write(&history_path, json)
            .context("Failed to write history file")?;
        Ok(())
    }

    fn collect_current_indicators(paths: &Paths) -> Result<Vec<(String, HashSet<String>)>> {
        let mut results = Vec::new();

        let reports = vec![
            ("Payload Domains", &paths.payload_report),
            ("AMP Report", &paths.amp_report),
            ("C2 Servers", &paths.c2_report),
            ("Top 100", &paths.top_100),
            ("Hex Report", &paths.hex_report),
            ("URLHaus Malware Downloads", &paths.haus_mal_down),
            ("PhishTank", &paths.phish_tank),
        ];

        for (name, path) in &reports {
            if path.exists() {
                match Self::read_indicators_from_file(path) {
                    Ok(indicators) => {
                        results.push((name.to_string(), indicators));
                    }
                    Err(e) => {
                        Printer::warning(&format!("Failed to read {}: {}", name, e));
                    }
                }
            }
        }

        Ok(results)
    }

    fn read_indicators_from_file(path: &Path) -> Result<HashSet<String>> {
        let file = File::open(path).context("Failed to open report file")?;
        let reader = BufReader::new(file);
        let mut indicators = HashSet::new();
        let is_csv = path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("csv"))
            .unwrap_or(false);

        for line_result in reader.lines() {
            let line = match line_result {
                Ok(l) => l,
                Err(_) => continue, // Skip invalid UTF-8 lines
            };
            
            let trimmed = line.trim();
            
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if is_csv {
                // Extract URLs and IPs from CSV
                use regex::Regex;
                let url_regex = Regex::new(r"https?://([^/,\s]+)").unwrap();
                let ip_regex = Regex::new(r"\b(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})\b").unwrap();
                
                for cap in url_regex.captures_iter(&trimmed) {
                    if let Some(domain) = cap.get(1) {
                        indicators.insert(domain.as_str().to_lowercase());
                    }
                }
                
                for cap in ip_regex.captures_iter(&trimmed) {
                    if let Some(ip) = cap.get(1) {
                        indicators.insert(ip.as_str().to_string());
                    }
                }
                
                // If no URL/IP found, use first field
                if indicators.is_empty() {
                    let first_field = trimmed.split(',').next().unwrap_or("").trim();
                    if !first_field.is_empty() {
                        indicators.insert(first_field.to_string());
                    }
                }
            } else {
                indicators.insert(trimmed.to_string());
            }
        }

        Ok(indicators)
    }

    pub fn compare_with_history(paths: &Paths) -> Result<()> {
        let history = Self::load_history(paths)?;
        let current = Self::collect_current_indicators(paths)?;
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        if history.snapshots.is_empty() {
            Printer::info("No previous scan history found.");
            Printer::info("This will be saved as the baseline for future comparisons.");
            
            // Save current scan as baseline
            let snapshots: Vec<ReportSnapshot> = current
                .into_iter()
                .map(|(name, indicators)| ReportSnapshot {
                    report_name: name,
                    indicators,
                    timestamp: current_time,
                })
                .collect();
            
            let new_history = ScanHistory {
                snapshots,
                last_scan_time: current_time,
            };
            
            Self::save_history(paths, &new_history)?;
            Printer::success("Baseline scan saved successfully!");
            return Ok(());
        }

        // Compare current with history
        crate::utils::clear_screen();
        Printer::header("SCAN DIFFERENCES", " :: ");
        println!();

        let mut total_new = 0;
        let mut total_removed = 0;
        let mut total_unchanged = 0;

        for (report_name, current_indicators) in &current {
            // Find matching snapshot in history
            let previous = history.snapshots.iter()
                .find(|s| s.report_name == *report_name);

            if let Some(prev_snapshot) = previous {
                let new_indicators: Vec<&String> = current_indicators
                    .difference(&prev_snapshot.indicators)
                    .collect();
                let removed_indicators: Vec<&String> = prev_snapshot.indicators
                    .difference(current_indicators)
                    .collect();
                let unchanged_count = current_indicators
                    .intersection(&prev_snapshot.indicators)
                    .count();

                total_new += new_indicators.len();
                total_removed += removed_indicators.len();
                total_unchanged += unchanged_count;

                println!("{}", report_name.cyan().bold());
                println!("  Current: {}", current_indicators.len().to_string().yellow());
                println!("  Previous: {}", prev_snapshot.indicators.len().to_string().yellow());
                println!("  New: {}", new_indicators.len().to_string().green());
                println!("  Removed: {}", removed_indicators.len().to_string().red());
                println!("  Unchanged: {}", unchanged_count.to_string().cyan());
                
                if !new_indicators.is_empty() {
                    println!();
                    println!("  {} New Indicators:", "New".green().bold());
                    for indicator in new_indicators.iter().take(20) {
                        println!("    {}", indicator);
                    }
                    if new_indicators.len() > 20 {
                        println!("    ... and {} more", new_indicators.len() - 20);
                    }
                }
                
                if !removed_indicators.is_empty() {
                    println!();
                    println!("  {} Removed Indicators:", "Removed".red().bold());
                    for indicator in removed_indicators.iter().take(20) {
                        println!("    {}", indicator);
                    }
                    if removed_indicators.len() > 20 {
                        println!("    ... and {} more", removed_indicators.len() - 20);
                    }
                }
                println!();
            } else {
                // New report that wasn't in history
                total_new += current_indicators.len();
                println!("{}", report_name.cyan().bold());
                println!("  {} New Report (not in previous scan)", "New".green().bold());
                println!("  Indicators: {}", current_indicators.len().to_string().yellow());
                println!();
            }
        }

        // Check for reports that were in history but not in current scan
        for snapshot in &history.snapshots {
            if !current.iter().any(|(name, _)| name == &snapshot.report_name) {
                total_removed += snapshot.indicators.len();
                println!("{}", snapshot.report_name.cyan().bold());
                println!("  {} Report no longer exists", "Removed".red().bold());
                println!("  Previous indicators: {}", snapshot.indicators.len().to_string().yellow());
                println!();
            }
        }

        // Summary
        println!("{}", "SUMMARY".cyan().bold());
        println!("  Total New Indicators: {}", total_new.to_string().green());
        println!("  Total Removed Indicators: {}", total_removed.to_string().red());
        println!("  Total Unchanged Indicators: {}", total_unchanged.to_string().cyan());
        
        if let Some(last_scan) = history.snapshots.first() {
            let last_scan_dt = chrono::DateTime::<chrono::Utc>::from_timestamp_secs(last_scan.timestamp)
                .unwrap_or_else(|| chrono::Utc::now());
            let last_scan_str = last_scan_dt.format("%Y-%m-%d %H:%M:%S UTC").to_string();
            println!("  Last Scan: {}", last_scan_str.cyan());
        }
        println!();

        // Update history with current scan
        let snapshots: Vec<ReportSnapshot> = current
            .into_iter()
            .map(|(name, indicators)| ReportSnapshot {
                report_name: name,
                indicators,
                timestamp: current_time,
            })
            .collect();
        
        let new_history = ScanHistory {
            snapshots,
            last_scan_time: current_time,
        };
        
        Self::save_history(paths, &new_history)?;
        Printer::info("History updated with current scan.");

        Ok(())
    }

    #[allow(dead_code)]
    pub fn show_history_summary(paths: &Paths) -> Result<()> {
        let history = Self::load_history(paths)?;

        crate::utils::clear_screen();
        Printer::header("SCAN HISTORY SUMMARY", " :: ");
        println!();

        if history.snapshots.is_empty() {
            Printer::warning("No scan history found.");
            Printer::info("Run DIFF or CHANGES after a scan to create history.");
            println!();
            return Ok(());
        }

        println!("{}", "HISTORY SNAPSHOTS".cyan().bold());
        println!();

        for snapshot in &history.snapshots {
            let dt = chrono::DateTime::<chrono::Utc>::from_timestamp_secs(snapshot.timestamp)
                .unwrap_or_else(|| chrono::Utc::now());
            let time_str = dt.format("%Y-%m-%d %H:%M:%S UTC").to_string();
            
            println!("  {}", snapshot.report_name.cyan());
            println!("    Indicators: {}", snapshot.indicators.len().to_string().yellow());
            println!("    Timestamp: {}", time_str.cyan());
            println!();
        }

        let total_indicators: usize = history.snapshots.iter()
            .map(|s| s.indicators.len())
            .sum();

        println!("{}", "SUMMARY".cyan().bold());
        println!("  Total Reports: {}", history.snapshots.len());
        println!("  Total Indicators: {}", total_indicators.to_string().yellow());
        
        if history.last_scan_time > 0 {
            let last_scan_dt = chrono::DateTime::<chrono::Utc>::from_timestamp_secs(history.last_scan_time)
                .unwrap_or_else(|| chrono::Utc::now());
            let last_scan_str = last_scan_dt.format("%Y-%m-%d %H:%M:%S UTC").to_string();
            println!("  Last Scan: {}", last_scan_str.cyan());
        }
        println!();

        Ok(())
    }
}

