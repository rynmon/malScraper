use crate::config::Paths;
use crate::printer::Printer;
use anyhow::{Context, Result};
use colored::Colorize;
use regex::Regex;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::net::IpAddr;
use std::path::Path;
use trust_dns_resolver::config::{ResolverConfig, ResolverOpts};
use trust_dns_resolver::TokioAsyncResolver;

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub indicator: String,
    pub indicator_type: String,
    pub is_valid_format: bool,
    pub is_active: Option<bool>,
    #[allow(dead_code)]
    pub error: Option<String>,
}

pub struct Validator;

impl Validator {
    pub async fn validate_report(paths: &Paths, report_name: &str) -> Result<Vec<ValidationResult>> {
        let report_path = Self::get_report_path(paths, report_name)
            .ok_or_else(|| anyhow::anyhow!("Unknown report: {}", report_name))?;

        if !report_path.exists() {
            return Err(anyhow::anyhow!("Report file does not exist: {:?}", report_path));
        }

        let indicators = Self::read_indicators(report_path)?;
        let mut results = Vec::new();

        Printer::info(&format!("Validating {} indicators...", indicators.len()));

        // Create DNS resolver
        let resolver = TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());

        for (idx, indicator) in indicators.iter().enumerate() {
            if (idx + 1) % 100 == 0 {
                Printer::info(&format!("Validating {}/{}...", idx + 1, indicators.len()));
            }

            let result = Self::validate_indicator(indicator, &resolver).await;
            results.push(result);
        }

        Ok(results)
    }

    fn get_report_path<'a>(paths: &'a Paths, report_name: &str) -> Option<&'a Path> {
        match report_name.to_lowercase().as_str() {
            "payload" | "payloads" | "payload-domains" => Some(&paths.payload_report),
            "amp" => Some(&paths.amp_report),
            "c2" | "c2servers" | "c2-servers" => Some(&paths.c2_report),
            "hex" | "hashes" => Some(&paths.hex_report),
            "haus" | "urlhaus" | "url-haus" => Some(&paths.haus_mal_down),
            "phish" | "phishtank" | "phish-tank" => Some(&paths.phish_tank),
            "top100" | "top" | "top-100" => Some(&paths.top_100),
            _ => None,
        }
    }

    fn read_indicators(path: &Path) -> Result<Vec<String>> {
        let file = File::open(path).context("Failed to open report file")?;
        let reader = BufReader::new(file);
        let mut indicators = Vec::new();
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
                let url_regex = Regex::new(r"https?://([^/,\s]+)").unwrap();
                let ip_regex = Regex::new(r"\b(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})\b").unwrap();
                
                for cap in url_regex.captures_iter(&trimmed) {
                    if let Some(domain) = cap.get(1) {
                        indicators.push(domain.as_str().to_lowercase());
                    }
                }
                
                for cap in ip_regex.captures_iter(&trimmed) {
                    if let Some(ip) = cap.get(1) {
                        indicators.push(ip.as_str().to_string());
                    }
                }
                
                // If no URL/IP found, use first field
                if !indicators.iter().any(|i| trimmed.contains(i)) {
                    let first_field = trimmed.split(',').next().unwrap_or("").trim();
                    if !first_field.is_empty() {
                        indicators.push(first_field.to_string());
                    }
                }
            } else {
                indicators.push(trimmed.to_string());
            }
        }

        Ok(indicators)
    }

    async fn validate_indicator(
        indicator: &str,
        resolver: &TokioAsyncResolver,
    ) -> ValidationResult {
        let trimmed = indicator.trim();
        
        // Determine indicator type
        let indicator_type = if Self::is_ip_address(trimmed) {
            "IP Address"
        } else if Self::is_url(trimmed) {
            "URL"
        } else if Self::is_hash(trimmed) {
            "Hash"
        } else {
            "Domain"
        };

        // Validate format
        let is_valid_format = match indicator_type {
            "IP Address" => Self::validate_ip_format(trimmed),
            "Domain" => Self::validate_domain_format(trimmed),
            "URL" => true, // URLs are complex, just check if it starts with http
            "Hash" => true, // Hashes are already validated by is_hash
            _ => false,
        };

        // Check if active (only for domains and IPs)
        let is_active = if indicator_type == "Domain" && is_valid_format {
            Some(Self::check_domain_active(trimmed, resolver).await)
        } else if indicator_type == "IP Address" && is_valid_format {
            Some(Self::check_ip_active(trimmed).await)
        } else {
            None
        };

        ValidationResult {
            indicator: trimmed.to_string(),
            indicator_type: indicator_type.to_string(),
            is_valid_format,
            is_active,
            error: None,
        }
    }

    fn is_ip_address(s: &str) -> bool {
        s.parse::<IpAddr>().is_ok()
    }

    fn is_url(s: &str) -> bool {
        s.starts_with("http://") || s.starts_with("https://")
    }

    fn is_hash(s: &str) -> bool {
        matches!(s.len(), 32 | 40 | 64) && s.chars().all(|c| c.is_ascii_hexdigit())
    }

    fn validate_ip_format(ip: &str) -> bool {
        if let Ok(addr) = ip.parse::<IpAddr>() {
            // Check if it's a valid IPv4 (not IPv6 for now)
            matches!(addr, IpAddr::V4(_))
        } else {
            false
        }
    }

    fn validate_domain_format(domain: &str) -> bool {
        // Basic domain validation
        if domain.is_empty() || domain.len() > 253 {
            return false;
        }

        // Check for valid characters
        if !domain.chars().all(|c| {
            c.is_ascii_alphanumeric() || c == '-' || c == '.' || c == '_'
        }) {
            return false;
        }

        // Check structure (at least one dot for TLD)
        let parts: Vec<&str> = domain.split('.').collect();
        if parts.len() < 2 {
            return false;
        }

        // Each part should be 1-63 characters
        parts.iter().all(|part| !part.is_empty() && part.len() <= 63)
    }

    async fn check_domain_active(domain: &str, resolver: &TokioAsyncResolver) -> bool {
        // Try to resolve the domain
        match resolver.lookup_ip(domain).await {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    async fn check_ip_active(_ip: &str) -> bool {
        // For IP addresses, we could ping them, but that's more complex
        // For now, just return true if format is valid
        // In the future, could add ICMP ping or TCP connection test
        true
    }

    pub fn display_results(results: &[ValidationResult], report_name: &str) {
        crate::utils::clear_screen();
        Printer::header(&format!("VALIDATION RESULTS :: {}", report_name), " :: ");
        println!();

        // Group by type
        let mut by_type: HashMap<String, Vec<&ValidationResult>> = HashMap::new();
        for result in results {
            by_type
                .entry(result.indicator_type.clone())
                .or_insert_with(Vec::new)
                .push(result);
        }

        // Statistics
        let total = results.len();
        let valid_format = results.iter().filter(|r| r.is_valid_format).count();
        let invalid_format = total - valid_format;
        let active = results.iter().filter(|r| r.is_active == Some(true)).count();
        let inactive = results.iter().filter(|r| r.is_active == Some(false)).count();
        let not_checked = results.iter().filter(|r| r.is_active.is_none()).count();

        println!("{}", "SUMMARY".cyan().bold());
        println!("  Total Indicators: {}", total.to_string().yellow());
        println!("  Valid Format: {}", valid_format.to_string().green());
        println!("  Invalid Format: {}", invalid_format.to_string().red());
        if active > 0 || inactive > 0 {
            println!("  Active: {}", active.to_string().green());
            println!("  Inactive: {}", inactive.to_string().red());
        }
        if not_checked > 0 {
            println!("  Not Checked: {}", not_checked.to_string().yellow());
        }
        println!();

        // Detailed results by type
        println!("{}", "DETAILED RESULTS".cyan().bold());
        println!();

        for (indicator_type, type_results) in &by_type {
            println!("{} ({})", indicator_type.cyan().bold(), type_results.len());
            
            let valid = type_results.iter().filter(|r| r.is_valid_format).count();
            let invalid = type_results.len() - valid;
            
            println!("  Valid Format: {}", valid.to_string().green());
            if invalid > 0 {
                println!("  Invalid Format: {}", invalid.to_string().red());
                println!();
                println!("  {} Invalid Indicators:", "Invalid".red().bold());
                for result in type_results.iter().filter(|r| !r.is_valid_format).take(20) {
                    println!("    {}", result.indicator);
                }
                if invalid > 20 {
                    println!("    ... and {} more", invalid - 20);
                }
            }

            // Show active/inactive for domains and IPs
            if indicator_type == "Domain" || indicator_type == "IP Address" {
                let active_count = type_results.iter().filter(|r| r.is_active == Some(true)).count();
                let inactive_count = type_results.iter().filter(|r| r.is_active == Some(false)).count();
                
                if active_count > 0 || inactive_count > 0 {
                    println!();
                    if active_count > 0 {
                        println!("  {} Active Indicators:", "Active".green().bold());
                        for result in type_results.iter().filter(|r| r.is_active == Some(true)).take(10) {
                            println!("    {}", result.indicator);
                        }
                        if active_count > 10 {
                            println!("    ... and {} more", active_count - 10);
                        }
                    }
                    
                    if inactive_count > 0 {
                        println!();
                        println!("  {} Inactive Indicators:", "Inactive".red().bold());
                        for result in type_results.iter().filter(|r| r.is_active == Some(false)).take(10) {
                            println!("    {}", result.indicator);
                        }
                        if inactive_count > 10 {
                            println!("    ... and {} more", inactive_count - 10);
                        }
                    }
                }
            }
            
            println!();
        }
    }
}

