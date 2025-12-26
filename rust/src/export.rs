use crate::config::Paths;
use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub enum ExportFormat {
    Iptables,
    WindowsFirewall,
    PfSense,
    Json,
    Csv,
    Stix,
    Taxii,
}

impl ExportFormat {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "iptables" | "iptable" => Some(Self::Iptables),
            "windows" | "win" | "windows-firewall" | "winfw" => Some(Self::WindowsFirewall),
            "pfsense" | "pf" => Some(Self::PfSense),
            "json" => Some(Self::Json),
            "csv" => Some(Self::Csv),
            "stix" => Some(Self::Stix),
            "taxii" => Some(Self::Taxii),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Iptables => "iptables",
            Self::WindowsFirewall => "windows-firewall",
            Self::PfSense => "pfsense",
            Self::Json => "json",
            Self::Csv => "csv",
            Self::Stix => "stix",
            Self::Taxii => "taxii",
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Self::Iptables => "sh",
            Self::WindowsFirewall => "ps1",
            Self::PfSense => "txt",
            Self::Json => "json",
            Self::Csv => "csv",
            Self::Stix => "json",
            Self::Taxii => "json",
        }
    }
}

pub struct Exporter;

impl Exporter {
    pub fn get_report_path<'a>(paths: &'a Paths, report_name: &str) -> Option<&'a Path> {
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

    pub fn get_report_display_name(report_name: &str) -> String {
        match report_name.to_lowercase().as_str() {
            "payload" | "payloads" | "payload-domains" => "Payload Domains".to_string(),
            "amp" => "AMP Report".to_string(),
            "c2" | "c2servers" | "c2-servers" => "C2 Servers".to_string(),
            "hex" | "hashes" => "Hex Report".to_string(),
            "haus" | "urlhaus" | "url-haus" => "URLHaus Malware Downloads".to_string(),
            "phish" | "phishtank" | "phish-tank" => "PhishTank".to_string(),
            "top100" | "top" | "top-100" => "Top 100".to_string(),
            _ => report_name.to_string(),
        }
    }

    pub fn export(
        paths: &Paths,
        format: ExportFormat,
        report_name: &str,
        output_path: Option<&str>,
    ) -> Result<String> {
        let report_path = Self::get_report_path(paths, report_name)
            .ok_or_else(|| anyhow::anyhow!("Unknown report: {}", report_name))?;

        if !report_path.exists() {
            return Err(anyhow::anyhow!("Report file does not exist: {:?}", report_path));
        }

        let report_display = Self::get_report_display_name(report_name);
        let default_filename = format!(
            "{}_{}.{}",
            report_name.replace(" ", "_").to_lowercase(),
            format.as_str(),
            format.extension()
        );
        let output_file = output_path
            .map(|p| p.to_string())
            .unwrap_or_else(|| default_filename);

        match format {
            ExportFormat::Iptables => Self::export_iptables(report_path, &output_file, &report_display),
            ExportFormat::WindowsFirewall => {
                Self::export_windows_firewall(report_path, &output_file, &report_display)
            }
            ExportFormat::PfSense => Self::export_pfsense(report_path, &output_file, &report_display),
            ExportFormat::Json => Self::export_json(paths, report_path, &output_file, &report_display),
            ExportFormat::Csv => Self::export_csv(report_path, &output_file, &report_display),
            ExportFormat::Stix => Self::export_stix(report_path, &output_file, &report_display),
            ExportFormat::Taxii => Self::export_taxii(report_path, &output_file, &report_display),
        }?;

        Ok(output_file)
    }

    fn export_iptables(path: &Path, output: &str, report_name: &str) -> Result<()> {
        let file = File::open(path).context("Failed to open report file")?;
        let reader = BufReader::new(file);
        let mut output_file = File::create(output).context("Failed to create output file")?;

        writeln!(output_file, "#!/bin/bash")?;
        writeln!(output_file, "# iptables rules generated from {} report", report_name)?;
        writeln!(output_file, "# Generated by malScraper")?;
        writeln!(output_file, "#")?;
        writeln!(output_file, "# Flush existing rules")?;
        writeln!(output_file, "iptables -F")?;
        writeln!(output_file, "iptables -X")?;
        writeln!(output_file, "#")?;
        writeln!(output_file, "# Block malicious domains/IPs")?;
        writeln!(output_file)?;

        for line in reader.lines() {
            let line = line.context("Failed to read line")?;
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            // Try to determine if it's an IP or domain
            if Self::is_ip_address(trimmed) {
                writeln!(output_file, "iptables -A INPUT -s {} -j DROP", trimmed)?;
                writeln!(output_file, "iptables -A OUTPUT -d {} -j DROP", trimmed)?;
            } else {
                // Domain - note: iptables doesn't directly support domain blocking
                // This would require dnsmasq or similar
                writeln!(output_file, "# Domain: {} (requires DNS filtering)", trimmed)?;
            }
        }

        writeln!(output_file)?;
        writeln!(output_file, "# Save rules")?;
        writeln!(output_file, "iptables-save > /etc/iptables/rules.v4")?;

        Ok(())
    }

    fn export_windows_firewall(path: &Path, output: &str, report_name: &str) -> Result<()> {
        let file = File::open(path).context("Failed to open report file")?;
        let reader = BufReader::new(file);
        let mut output_file = File::create(output).context("Failed to create output file")?;

        writeln!(output_file, "# Windows Firewall rules generated from {} report", report_name)?;
        writeln!(output_file, "# Generated by malScraper")?;
        writeln!(output_file, "# Run this script as Administrator")?;
        writeln!(output_file)?;
        writeln!(output_file, "$ErrorActionPreference = 'Stop'")?;
        writeln!(output_file)?;
        writeln!(output_file, "# Create firewall rule group")?;
        writeln!(output_file, "$RuleGroup = 'malScraper Blocked Indicators'")?;
        writeln!(output_file)?;
        writeln!(output_file, "# Block malicious domains/IPs")?;
        writeln!(output_file)?;

        let mut rule_num = 1;
        for line in reader.lines() {
            let line = line.context("Failed to read line")?;
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if Self::is_ip_address(trimmed) {
                writeln!(
                    output_file,
                    "New-NetFirewallRule -DisplayName \"Block IP {} - {}\" -Direction Outbound -RemoteAddress {} -Action Block -Group $RuleGroup -Description \"Blocked by malScraper from {} report\"",
                    rule_num, trimmed, trimmed, report_name
                )?;
            } else {
                // For domains, we can create a rule but Windows Firewall doesn't directly block domains
                // This would require DNS filtering or host file modification
                writeln!(
                    output_file,
                    "# Domain: {} (Windows Firewall cannot directly block domains - consider using hosts file or DNS filtering)",
                    trimmed
                )?;
            }
            rule_num += 1;
        }

        Ok(())
    }

    fn export_pfsense(path: &Path, output: &str, report_name: &str) -> Result<()> {
        let file = File::open(path).context("Failed to open report file")?;
        let reader = BufReader::new(file);
        let mut output_file = File::create(output).context("Failed to create output file")?;

        writeln!(output_file, "# pfSense alias rules generated from {} report", report_name)?;
        writeln!(output_file, "# Generated by malScraper")?;
        writeln!(output_file, "# Import these aliases in pfSense: Firewall > Aliases")?;
        writeln!(output_file)?;
        writeln!(output_file, "# Format: Name,Type,Content")?;
        writeln!(output_file)?;

        let mut entries = Vec::new();
        for line in reader.lines() {
            let line = line.context("Failed to read line")?;
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let entry_type = if Self::is_ip_address(trimmed) {
                "host"
            } else {
                "host"
            };
            entries.push((trimmed.to_string(), entry_type));
        }

        writeln!(
            output_file,
            "malScraper_{},url,{}",
            report_name.replace(" ", "_").to_lowercase(),
            entries.iter().map(|(e, _)| e.as_str()).collect::<Vec<_>>().join(" ")
        )?;

        Ok(())
    }

    fn export_json(
        _paths: &Paths,
        report_path: &Path,
        output: &str,
        report_name: &str,
    ) -> Result<()> {
        use serde_json::json;
        use std::time::SystemTime;

        let file = File::open(report_path).context("Failed to open report file")?;
        let reader = BufReader::new(file);
        let mut indicators = Vec::new();

        for line in reader.lines() {
            let line = line.context("Failed to read line")?;
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let indicator_type = if Self::is_ip_address(trimmed) {
                "ipv4-addr"
            } else if Self::is_url(trimmed) {
                "url"
            } else if Self::is_hash(trimmed) {
                "file"
            } else {
                "domain-name"
            };

            indicators.push(json!({
                "value": trimmed,
                "type": indicator_type,
                "source": report_name,
            }));
        }

        let metadata = json!({
            "export_info": {
                "tool": "malScraper",
                "version": env!("CARGO_PKG_VERSION"),
                "export_date": SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                "report_name": report_name,
                "report_path": report_path.to_string_lossy(),
                "total_indicators": indicators.len(),
            },
            "indicators": indicators,
        });

        let json_string = serde_json::to_string_pretty(&metadata)?;
        std::fs::write(output, json_string).context("Failed to write JSON file")?;

        Ok(())
    }

    fn export_csv(report_path: &Path, output: &str, report_name: &str) -> Result<()> {
        use std::time::SystemTime;

        let file = File::open(report_path).context("Failed to open report file")?;
        let reader = BufReader::new(file);
        let mut output_file = File::create(output).context("Failed to create output file")?;

        // Write CSV header with metadata
        writeln!(output_file, "# CSV export generated by malScraper")?;
        writeln!(output_file, "# Report: {}", report_name)?;
        writeln!(
            output_file,
            "# Export Date: {}",
            SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        )?;
        writeln!(output_file, "#")?;
        writeln!(output_file, "Indicator,Type,Source,Timestamp")?;

        let timestamp = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        for line in reader.lines() {
            let line = line.context("Failed to read line")?;
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let indicator_type = if Self::is_ip_address(trimmed) {
                "ipv4-addr"
            } else if Self::is_url(trimmed) {
                "url"
            } else if Self::is_hash(trimmed) {
                "file-hash"
            } else {
                "domain-name"
            };

            writeln!(
                output_file,
                "{},{},{},{}",
                trimmed, indicator_type, report_name, timestamp
            )?;
        }

        Ok(())
    }

    fn export_stix(report_path: &Path, output: &str, report_name: &str) -> Result<()> {
        use serde_json::json;
        use std::time::SystemTime;

        let file = File::open(report_path).context("Failed to open report file")?;
        let reader = BufReader::new(file);
        let mut indicators = Vec::new();

        let timestamp = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        for line in reader.lines() {
            let line = line.context("Failed to read line")?;
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            let indicator_type = if Self::is_ip_address(trimmed) {
                "ipv4-addr"
            } else if Self::is_url(trimmed) {
                "url"
            } else if Self::is_hash(trimmed) {
                "file"
            } else {
                "domain-name"
            };

            // STIX 2.1 format - use chrono for proper timestamp formatting
            let created_time = chrono::DateTime::<chrono::Utc>::from_timestamp_secs(timestamp as i64)
                .unwrap_or_else(|| chrono::Utc::now())
                .format("%Y-%m-%dT%H:%M:%S%.3fZ")
                .to_string();

            indicators.push(json!({
                "type": "indicator",
                "spec_version": "2.1",
                "id": format!("indicator--{}", uuid::Uuid::new_v4()),
                "created": created_time,
                "modified": created_time,
                "pattern": format!("[{}:value = '{}']", indicator_type, trimmed),
                "pattern_type": "stix",
                "valid_from": created_time,
                "labels": ["malicious-activity"],
                "description": format!("Indicator from {} report", report_name),
            }));
        }

        let stix_bundle = json!({
            "type": "bundle",
            "id": format!("bundle--{}", uuid::Uuid::new_v4()),
            "spec_version": "2.1",
            "objects": indicators,
        });

        let json_string = serde_json::to_string_pretty(&stix_bundle)?;
        std::fs::write(output, json_string).context("Failed to write STIX file")?;

        Ok(())
    }

    fn export_taxii(report_path: &Path, output: &str, report_name: &str) -> Result<()> {
        // TAXII is a transport protocol, but we'll export as a TAXII-compatible collection
        // This is similar to STIX but formatted for TAXII servers
        Self::export_stix(report_path, output, report_name)
    }

    // Helper functions
    fn is_ip_address(s: &str) -> bool {
        // Simple IPv4 check
        let parts: Vec<&str> = s.split('.').collect();
        if parts.len() != 4 {
            return false;
        }
        parts.iter().all(|p| p.parse::<u8>().is_ok())
    }

    fn is_url(s: &str) -> bool {
        s.starts_with("http://") || s.starts_with("https://")
    }

    fn is_hash(s: &str) -> bool {
        // Check for common hash lengths
        matches!(s.len(), 32 | 40 | 64) && s.chars().all(|c| c.is_ascii_hexdigit())
    }
}

