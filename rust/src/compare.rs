use crate::config::Paths;
use crate::printer::Printer;
use anyhow::{Context, Result};
use colored::Colorize;
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct Compare;

#[derive(Debug)]
pub struct ComparisonResult {
    pub report1_name: String,
    pub report2_name: String,
    pub report1_total: usize,
    pub report2_total: usize,
    pub common: usize,
    pub only_in_report1: Vec<String>,
    pub only_in_report2: Vec<String>,
}

impl Compare {
    /// Get the path for a report by name
    fn get_report_path<'a>(paths: &'a Paths, report_name: &str) -> Option<&'a Path> {
        match report_name.to_lowercase().as_str() {
            "payload" | "payloadreport" => Some(&paths.payload_report),
            "amp" | "ampreport" => Some(&paths.amp_report),
            "c2" | "c2report" => Some(&paths.c2_report),
            "top100" | "top" => Some(&paths.top_100),
            "hex" | "hexreport" => Some(&paths.hex_report),
            "haus" | "hausmaldown" => Some(&paths.haus_mal_down),
            "phish" | "phishtank" => Some(&paths.phish_tank),
            _ => None,
        }
    }

    /// Read indicators from a report file
    fn read_indicators_from_file(path: &Path) -> Result<HashSet<String>> {
        let file = File::open(path)
            .with_context(|| format!("Failed to open file: {}", path.display()))?;
        let reader = BufReader::new(file);
        let mut indicators = HashSet::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line.with_context(|| {
                format!("Failed to read line {} from {}", line_num + 1, path.display())
            })?;
            let trimmed = line.trim();
            
            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Handle CSV files (extract URL or IP from CSV)
            if path.extension().and_then(|s| s.to_str()) == Some("csv") {
                // Try to parse CSV: look for URL or IP in common CSV formats
                let parts: Vec<&str> = trimmed.split(',').collect();
                if let Some(indicator) = parts.iter()
                    .find(|s| {
                        let s = s.trim();
                        s.starts_with("http://") || s.starts_with("https://") || 
                        s.matches('.').count() >= 3 && s.chars().all(|c| c.is_ascii_digit() || c == '.')
                    })
                    .map(|s| s.trim().to_string())
                {
                    indicators.insert(indicator);
                } else {
                    // If no URL/IP found, use first non-empty field
                    if let Some(first_field) = parts.first().map(|s| s.trim()) {
                        if !first_field.is_empty() {
                            indicators.insert(first_field.to_string());
                        }
                    }
                }
            } else {
                // For text files, use the line as-is (trimmed)
                indicators.insert(trimmed.to_string());
            }
        }

        Ok(indicators)
    }

    /// Compare two reports
    pub fn compare_reports(
        paths: &Paths,
        report1_name: &str,
        report2_name: &str,
    ) -> Result<ComparisonResult> {
        let report1_path = Self::get_report_path(paths, report1_name)
            .ok_or_else(|| anyhow::anyhow!("Unknown report: {}", report1_name))?;
        let report2_path = Self::get_report_path(paths, report2_name)
            .ok_or_else(|| anyhow::anyhow!("Unknown report: {}", report2_name))?;

        if !report1_path.exists() {
            return Err(anyhow::anyhow!("Report not found: {}", report1_path.display()));
        }
        if !report2_path.exists() {
            return Err(anyhow::anyhow!("Report not found: {}", report2_path.display()));
        }

        let indicators1 = Self::read_indicators_from_file(report1_path)
            .with_context(|| format!("Failed to read indicators from {}", report1_path.display()))?;
        let indicators2 = Self::read_indicators_from_file(report2_path)
            .with_context(|| format!("Failed to read indicators from {}", report2_path.display()))?;

        let report1_total = indicators1.len();
        let report2_total = indicators2.len();

        // Find common indicators
        let common: HashSet<_> = indicators1.intersection(&indicators2).cloned().collect();
        let common_count = common.len();

        // Find indicators only in report1
        let only_in_report1: Vec<String> = indicators1
            .difference(&indicators2)
            .cloned()
            .collect();

        // Find indicators only in report2
        let only_in_report2: Vec<String> = indicators2
            .difference(&indicators1)
            .cloned()
            .collect();

        Ok(ComparisonResult {
            report1_name: report1_name.to_string(),
            report2_name: report2_name.to_string(),
            report1_total,
            report2_total,
            common: common_count,
            only_in_report1,
            only_in_report2,
        })
    }

    /// Display comparison results
    pub fn display_results(result: &ComparisonResult) {
        Printer::info(&format!(
            "Comparing '{}' and '{}'",
            result.report1_name, result.report2_name
        ));
        println!();

        // Summary statistics
        println!("{}", "Comparison Summary:".bold().cyan());
        println!(
            "  {}: {} indicators",
            result.report1_name.bold(),
            result.report1_total.to_string().yellow()
        );
        println!(
            "  {}: {} indicators",
            result.report2_name.bold(),
            result.report2_total.to_string().yellow()
        );
        println!(
            "  {}: {} indicators",
            "Common".bold().green(),
            result.common.to_string().green()
        );
        println!(
            "  {}: {} indicators",
            format!("Only in {}", result.report1_name).bold().red(),
            result.only_in_report1.len().to_string().red()
        );
        println!(
            "  {}: {} indicators",
            format!("Only in {}", result.report2_name).bold().red(),
            result.only_in_report2.len().to_string().red()
        );
        println!();

        // Calculate percentage differences
        let report1_diff_pct = if result.report1_total > 0 {
            (result.only_in_report1.len() as f64 / result.report1_total as f64) * 100.0
        } else {
            0.0
        };
        let report2_diff_pct = if result.report2_total > 0 {
            (result.only_in_report2.len() as f64 / result.report2_total as f64) * 100.0
        } else {
            0.0
        };

        println!("{}", "Difference Analysis:".bold().cyan());
        println!(
            "  {} of {} ({:.1}%) are unique to {}",
            result.only_in_report1.len().to_string().red(),
            result.report1_total,
            report1_diff_pct,
            result.report1_name
        );
        println!(
            "  {} of {} ({:.1}%) are unique to {}",
            result.only_in_report2.len().to_string().red(),
            result.report2_total,
            report2_diff_pct,
            result.report2_name
        );
        println!(
            "  {} indicators ({:.1}%) are common to both",
            result.common.to_string().green(),
            if result.report1_total + result.report2_total > 0 {
                (result.common as f64 / ((result.report1_total + result.report2_total) as f64 / 2.0)) * 100.0
            } else {
                0.0
            }
        );
        println!();

        // Show unique indicators (limited to first 20 of each)
        if !result.only_in_report1.is_empty() {
            println!(
                "{} (showing first {}):",
                format!("Only in {}", result.report1_name).bold().red(),
                result.only_in_report1.len().min(20)
            );
            for indicator in result.only_in_report1.iter().take(20) {
                println!("  - {}", indicator.red());
            }
            if result.only_in_report1.len() > 20 {
                println!(
                    "  ... and {} more",
                    (result.only_in_report1.len() - 20).to_string().yellow()
                );
            }
            println!();
        }

        if !result.only_in_report2.is_empty() {
            println!(
                "{} (showing first {}):",
                format!("Only in {}", result.report2_name).bold().red(),
                result.only_in_report2.len().min(20)
            );
            for indicator in result.only_in_report2.iter().take(20) {
                println!("  - {}", indicator.red());
            }
            if result.only_in_report2.len() > 20 {
                println!(
                    "  ... and {} more",
                    (result.only_in_report2.len() - 20).to_string().yellow()
                );
            }
            println!();
        }

        if result.only_in_report1.is_empty() && result.only_in_report2.is_empty() {
            Printer::success("Both reports contain identical indicators!");
        }
    }
}

