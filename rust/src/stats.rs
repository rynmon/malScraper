use crate::config::Paths;
use crate::printer::Printer;
use anyhow::{Context, Result};
use colored::Colorize;
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Default)]
pub struct ReportStats {
    pub file_name: String,
    pub exists: bool,
    pub line_count: usize,
    pub unique_count: usize,
    pub file_size: u64,
    pub last_modified: Option<String>,
}

pub struct Statistics;

impl Statistics {
    pub fn analyze_all_reports(paths: &Paths) -> Result<Vec<ReportStats>> {
        let mut stats = Vec::new();

        // Payload Report
        stats.push(Self::analyze_report(
            &paths.payload_report,
            "Payload Domains",
            Some(Self::count_unique_urls),
        )?);

        // AMP Report
        stats.push(Self::analyze_report(
            &paths.amp_report,
            "AMP Report",
            Some(Self::count_unique_lines),
        )?);

        // C2 Report
        stats.push(Self::analyze_report(
            &paths.c2_report,
            "C2 Servers",
            Some(Self::count_unique_lines),
        )?);

        // Top 100
        stats.push(Self::analyze_report(
            &paths.top_100,
            "Top 100",
            None,
        )?);

        // Hex Report
        stats.push(Self::analyze_report(
            &paths.hex_report,
            "Hex Report",
            Some(Self::count_unique_hashes),
        )?);

        // URLHaus Malware Downloads
        stats.push(Self::analyze_report(
            &paths.haus_mal_down,
            "URLHaus Malware Downloads",
            Some(Self::count_unique_csv_urls),
        )?);

        // PhishTank
        stats.push(Self::analyze_report(
            &paths.phish_tank,
            "PhishTank",
            Some(Self::count_unique_csv_urls),
        )?);

        Ok(stats)
    }

    fn analyze_report(
        path: &Path,
        name: &str,
        unique_counter: Option<fn(&Path) -> Result<usize>>,
    ) -> Result<ReportStats> {
        let exists = path.exists();
        let mut stats = ReportStats {
            file_name: name.to_string(),
            exists,
            ..Default::default()
        };

        if !exists {
            return Ok(stats);
        }

        // File size
        if let Ok(metadata) = fs::metadata(path) {
            stats.file_size = metadata.len();
        }

        // Last modified
        if let Ok(metadata) = fs::metadata(path) {
            if let Ok(modified) = metadata.modified() {
                if let Ok(datetime) = modified.duration_since(std::time::UNIX_EPOCH) {
                    let timestamp = datetime.as_secs() as i64;
                    if let Some(dt) = chrono::DateTime::<chrono::Utc>::from_timestamp_secs(timestamp) {
                        let local_dt: chrono::DateTime<chrono::Local> = dt.with_timezone(&chrono::Local);
                        stats.last_modified = Some(local_dt.format("%Y-%m-%d %H:%M:%S").to_string());
                    }
                }
            }
        }

        // Line count
        if let Ok(file) = fs::File::open(path) {
            let reader = BufReader::new(file);
            stats.line_count = reader.lines().count();
        }

        // Unique count
        if let Some(counter) = unique_counter {
            stats.unique_count = counter(path).unwrap_or(0);
        } else {
            stats.unique_count = stats.line_count; // If no unique counter, assume all are unique
        }

        Ok(stats)
    }

    fn count_unique_lines(path: &Path) -> Result<usize> {
        let file = fs::File::open(path).context("Failed to open file")?;
        let reader = BufReader::new(file);
        let mut unique = HashSet::new();

        for line in reader.lines() {
            let line = line.context("Failed to read line")?;
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                unique.insert(trimmed.to_string());
            }
        }

        Ok(unique.len())
    }

    fn count_unique_urls(path: &Path) -> Result<usize> {
        let file = fs::File::open(path).context("Failed to open file")?;
        let reader = BufReader::new(file);
        let mut unique = HashSet::new();
        let url_regex = Regex::new(r"https?://([^/\s]+)").unwrap();

        for line in reader.lines() {
            let line = line.context("Failed to read line")?;
            if let Some(captures) = url_regex.captures(&line) {
                if let Some(domain) = captures.get(1) {
                    let domain = domain.as_str().to_lowercase();
                    unique.insert(domain);
                }
            }
        }

        Ok(unique.len())
    }

    fn count_unique_hashes(path: &Path) -> Result<usize> {
        let file = fs::File::open(path).context("Failed to open file")?;
        let reader = BufReader::new(file);
        let mut unique = HashSet::new();
        // Hash patterns: MD5 (32 hex), SHA1 (40 hex), SHA256 (64 hex)
        let hash_regex = Regex::new(r"\b([a-fA-F0-9]{32}|[a-fA-F0-9]{40}|[a-fA-F0-9]{64})\b").unwrap();

        for line in reader.lines() {
            let line = line.context("Failed to read line")?;
            for cap in hash_regex.captures_iter(&line) {
                if let Some(hash) = cap.get(1) {
                    unique.insert(hash.as_str().to_lowercase());
                }
            }
        }

        Ok(unique.len())
    }

    fn count_unique_csv_urls(path: &Path) -> Result<usize> {
        let file = fs::File::open(path).context("Failed to open file")?;
        let reader = BufReader::new(file);
        let mut unique = HashSet::new();
        let url_regex = Regex::new(r"https?://([^/,\s]+)").unwrap();

        for line in reader.lines() {
            let line = line.context("Failed to read line")?;
            for cap in url_regex.captures_iter(&line) {
                if let Some(domain) = cap.get(1) {
                    let domain = domain.as_str().to_lowercase();
                    unique.insert(domain);
                }
            }
        }

        Ok(unique.len())
    }

    pub fn format_size(bytes: u64) -> String {
        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.2} KB", bytes as f64 / 1024.0)
        } else {
            format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
        }
    }

    pub fn display_dashboard(paths: &Paths) -> Result<()> {
        crate::utils::clear_screen();
        Printer::header("STATISTICS DASHBOARD", " :: ");
        println!();

        let stats = Self::analyze_all_reports(paths)?;

        // Calculate totals
        let total_lines: usize = stats.iter().map(|s| s.line_count).sum();
        let total_unique: usize = stats.iter().map(|s| s.unique_count).sum();
        let total_size: u64 = stats.iter().map(|s| s.file_size).sum();
        let existing_reports: usize = stats.iter().filter(|s| s.exists).count();

        // Summary section
        println!("{}", "SUMMARY".cyan().bold());
        println!("  Total Reports: {}", stats.len());
        println!("  Existing Reports: {}", existing_reports);
        println!("  Total Indicators: {}", total_lines.to_string().yellow());
        println!("  Unique Indicators: {}", total_unique.to_string().green());
        println!("  Total Size: {}", Self::format_size(total_size).yellow());
        println!();

        // Detailed report section
        println!("{}", "REPORT DETAILS".cyan().bold());
        println!();

        for stat in &stats {
            if stat.exists {
                Printer::success(&format!("{}", stat.file_name));
                println!(
                    "  {} Lines: {}",
                    "Total".magenta(),
                    stat.line_count.to_string().yellow()
                );
                if stat.unique_count != stat.line_count {
                    println!(
                        "  {} Unique: {}",
                        "Unique".magenta(),
                        stat.unique_count.to_string().green()
                    );
                }
                println!(
                    "  {} Size: {}",
                    "File".magenta(),
                    Self::format_size(stat.file_size).yellow()
                );
                if let Some(ref modified) = stat.last_modified {
                    println!("  {} Modified: {}", "Last".magenta(), modified.cyan());
                }
            } else {
                Printer::warning(&format!("{} - Not found", stat.file_name));
            }
            println!();
        }

        // Recommendations
        if existing_reports == 0 {
            Printer::warning("No reports found. Run FULL-SCAN or QUICK-SCAN to generate reports.");
        } else if existing_reports < stats.len() {
            Printer::info("Some reports are missing. Consider running FULL-SCAN to download all feeds.");
        }

        println!();
        Ok(())
    }
}

