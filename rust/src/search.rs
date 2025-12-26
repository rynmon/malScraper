use crate::config::Paths;
use crate::printer::Printer;
use anyhow::{Context, Result};
use colored::Colorize;
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub file_name: String,
    pub line_number: usize,
    pub content: String,
    pub matched_term: String,
}

pub struct Search;

impl Search {
    pub fn search_all_reports(paths: &Paths, term: &str, case_sensitive: bool) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();
        let pattern = if case_sensitive {
            Regex::new(&regex::escape(term))
        } else {
            Regex::new(&format!("(?i){}", regex::escape(term)))
        }
        .context("Invalid search pattern")?;

        // Search in all reports
        let reports = vec![
            ("Payload Domains", &paths.payload_report),
            ("AMP Report", &paths.amp_report),
            ("C2 Servers", &paths.c2_report),
            ("Top 100", &paths.top_100),
            ("Hex Report", &paths.hex_report),
            ("URLHaus Malware Downloads", &paths.haus_mal_down),
            ("PhishTank", &paths.phish_tank),
        ];

        for (name, path) in reports {
            if path.exists() {
                if let Ok(file_results) = Self::search_file(path, name, &pattern, term) {
                    results.extend(file_results);
                }
            }
        }

        Ok(results)
    }

    fn search_file(
        path: &Path,
        file_name: &str,
        pattern: &Regex,
        original_term: &str,
    ) -> Result<Vec<SearchResult>> {
        let file = File::open(path).context("Failed to open file")?;
        let reader = BufReader::new(file);
        let mut results = Vec::new();

        for (line_num, line) in reader.lines().enumerate() {
            let line = line.context("Failed to read line")?;
            if pattern.is_match(&line) {
                // Find the matched portion
                let matched = if let Some(mat) = pattern.find(&line) {
                    mat.as_str().to_string()
                } else {
                    original_term.to_string()
                };

                results.push(SearchResult {
                    file_name: file_name.to_string(),
                    line_number: line_num + 1,
                    content: line.trim().to_string(),
                    matched_term: matched,
                });
            }
        }

        Ok(results)
    }

    pub fn display_results(results: &[SearchResult], term: &str) {
        crate::utils::clear_screen();
        Printer::header(&format!("SEARCH RESULTS :: \"{}\"", term), " :: ");
        println!();

        if results.is_empty() {
            Printer::warning(&format!("No matches found for \"{}\"", term));
            println!();
            return;
        }

        // Group results by file
        use std::collections::HashMap;
        let mut by_file: HashMap<String, Vec<&SearchResult>> = HashMap::new();
        for result in results {
            by_file
                .entry(result.file_name.clone())
                .or_insert_with(Vec::new)
                .push(result);
        }

        // Display summary
        println!("{}", "SUMMARY".cyan().bold());
        println!("  Total Matches: {}", results.len().to_string().yellow());
        println!("  Files with Matches: {}", by_file.len().to_string().green());
        println!();

        // Display results grouped by file
        println!("{}", "RESULTS".cyan().bold());
        println!();

        for (file_name, file_results) in &by_file {
            Printer::success(&format!("{} ({})", file_name, file_results.len()));
            for result in file_results.iter().take(20) {
                // Highlight the matched term in the line
                let highlighted = result.content.replace(
                    &result.matched_term,
                    &result.matched_term.yellow().bold().to_string(),
                );
                println!("  {}: {}", format!("Line {}", result.line_number).cyan(), highlighted);
            }
            if file_results.len() > 20 {
                Printer::info(&format!("  ... and {} more matches in this file", file_results.len() - 20));
            }
            println!();
        }

        if results.len() > 100 {
            Printer::warning(&format!("Showing first 100 results. Total: {} matches", results.len()));
        }
    }

    pub fn filter_reports(
        paths: &Paths,
        feed_type: Option<&str>,
        pattern: Option<&str>,
    ) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();

        // Map feed types to paths
        let reports: Vec<(&str, &Path)> = match feed_type {
            Some("payload") | Some("payloads") => vec![("Payload Domains", &paths.payload_report)],
            Some("amp") => vec![("AMP Report", &paths.amp_report)],
            Some("c2") | Some("c2servers") | Some("c2-servers") => vec![("C2 Servers", &paths.c2_report)],
            Some("hex") | Some("hashes") => vec![("Hex Report", &paths.hex_report)],
            Some("haus") | Some("urlhaus") | Some("url-haus") => vec![("URLHaus Malware Downloads", &paths.haus_mal_down)],
            Some("phish") | Some("phishtank") | Some("phish-tank") => vec![("PhishTank", &paths.phish_tank)],
            Some("top100") | Some("top") | Some("top-100") => vec![("Top 100", &paths.top_100)],
            None => vec![
                ("Payload Domains", &paths.payload_report),
                ("AMP Report", &paths.amp_report),
                ("C2 Servers", &paths.c2_report),
                ("Top 100", &paths.top_100),
                ("Hex Report", &paths.hex_report),
                ("URLHaus Malware Downloads", &paths.haus_mal_down),
                ("PhishTank", &paths.phish_tank),
            ],
            _ => {
                return Err(anyhow::anyhow!(
                    "Unknown feed type. Valid types: payload, amp, c2, hex, haus, phish, top100"
                ));
            }
        };

        // Create pattern if provided
        let regex_pattern = if let Some(pat) = pattern {
            Some(Regex::new(&format!("(?i){}", regex::escape(pat))).context("Invalid filter pattern")?)
        } else {
            None
        };

        // Filter reports
        for (name, path) in reports {
            if !path.exists() {
                continue;
            }

            if let Some(ref pattern) = regex_pattern {
                if let Ok(file_results) = Self::search_file(path, name, pattern, pattern.as_str()) {
                    results.extend(file_results);
                }
            } else {
                // No pattern, just list all lines from selected feed
                if let Ok(file) = File::open(path) {
                    let reader = BufReader::new(file);
                    for (line_num, line) in reader.lines().enumerate() {
                        if let Ok(line) = line {
                            if !line.trim().is_empty() {
                                results.push(SearchResult {
                                    file_name: name.to_string(),
                                    line_number: line_num + 1,
                                    content: line.trim().to_string(),
                                    matched_term: String::new(),
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    pub fn display_filter_results(results: &[SearchResult], feed_type: Option<&str>, pattern: Option<&str>) {
        crate::utils::clear_screen();
        
        let title = if let Some(feed) = feed_type {
            if let Some(pat) = pattern {
                format!("FILTER RESULTS :: Feed: {} | Pattern: \"{}\"", feed, pat)
            } else {
                format!("FILTER RESULTS :: Feed: {}", feed)
            }
        } else if let Some(pat) = pattern {
            format!("FILTER RESULTS :: Pattern: \"{}\"", pat)
        } else {
            "FILTER RESULTS".to_string()
        };
        
        Printer::header(&title, " :: ");
        println!();

        if results.is_empty() {
            Printer::warning("No results found matching the filter criteria.");
            println!();
            return;
        }

        // Group by file
        use std::collections::HashMap;
        let mut by_file: HashMap<String, Vec<&SearchResult>> = HashMap::new();
        for result in results {
            by_file
                .entry(result.file_name.clone())
                .or_insert_with(Vec::new)
                .push(result);
        }

        println!("{}", "SUMMARY".cyan().bold());
        println!("  Total Results: {}", results.len().to_string().yellow());
        println!("  Files: {}", by_file.len().to_string().green());
        println!();

        println!("{}", "RESULTS".cyan().bold());
        println!();

        for (file_name, file_results) in &by_file {
            Printer::success(&format!("{} ({})", file_name, file_results.len()));
            for result in file_results.iter().take(50) {
                if result.matched_term.is_empty() {
                    println!("  {}: {}", format!("Line {}", result.line_number).cyan(), result.content);
                } else {
                    let highlighted = result.content.replace(
                        &result.matched_term,
                        &result.matched_term.yellow().bold().to_string(),
                    );
                    println!("  {}: {}", format!("Line {}", result.line_number).cyan(), highlighted);
                }
            }
            if file_results.len() > 50 {
                Printer::info(&format!("  ... and {} more results in this file", file_results.len() - 50));
            }
            println!();
        }
    }
}

