use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "malscraper")]
#[command(about = "Cross-platform tool to scrape malware domains, IOCs, and C2 IPs from various feeds")]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Output directory for reports (default: user data directory)
    #[arg(long, global = true)]
    pub output_dir: Option<PathBuf>,

    /// Run in non-interactive mode
    #[arg(long, global = true)]
    pub non_interactive: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Perform a quick scan (most recent 100 payload domains)
    QuickScan {
        /// Output directory for reports
        #[arg(long)]
        output_dir: Option<PathBuf>,
    },
    /// Perform a full scan (all feeds)
    FullScan {
        /// Output directory for reports
        #[arg(long)]
        output_dir: Option<PathBuf>,
    },
    /// Show statistics dashboard
    Stats,
    /// Search reports for a term
    Search {
        /// Search term
        term: String,
    },
    /// Filter reports by feed type or pattern
    Filter {
        /// Feed type (optional)
        feed_type: Option<String>,
        /// Pattern to filter by (optional)
        pattern: Option<String>,
    },
    /// Export report to firewall/SIEM format
    Export {
        /// Export format (iptables, windows, pfsense, json, csv, stix, taxii)
        format: String,
        /// Report name (payload, amp, c2, hex, haus, phish, top100)
        report: String,
        /// Output file path (optional)
        output: Option<PathBuf>,
    },
    /// Deduplicate all reports and create master list
    Dedupe {
        /// Output file path (optional, default: MasterList.txt)
        output: Option<PathBuf>,
    },
    /// Compare current scan with previous scan
    Diff,
    /// Validate IP addresses and domains
    Validate {
        /// Report name (payload, amp, c2, hex, haus, phish, top100)
        report: String,
    },
    /// Compare two reports side-by-side
    Compare {
        /// First report name (payload, amp, c2, hex, haus, phish, top100)
        report1: String,
        /// Second report name (payload, amp, c2, hex, haus, phish, top100)
        report2: String,
    },
    /// Manage whitelist (exclude false positives)
    Whitelist {
        #[command(subcommand)]
        command: WhitelistCommands,
    },
    /// Manage custom feed URLs
    Feeds {
        #[command(subcommand)]
        command: FeedCommands,
    },
}

#[derive(Subcommand)]
pub enum WhitelistCommands {
    /// Add an indicator to the whitelist
    Add {
        /// Indicator to whitelist (domain, IP, URL, or hash)
        indicator: String,
        /// Reason for whitelisting (optional)
        reason: Option<String>,
    },
    /// List all whitelisted indicators
    List,
    /// Remove an indicator from the whitelist
    Remove {
        /// Indicator to remove from whitelist
        indicator: String,
    },
}

#[derive(Subcommand)]
pub enum FeedCommands {
    /// Add a custom feed
    Add {
        /// Feed URL
        url: String,
        /// Feed name (optional)
        name: Option<String>,
        /// Feed description (optional)
        description: Option<String>,
    },
    /// List all custom feeds
    List,
    /// Remove a custom feed
    Remove {
        /// Feed name or URL
        name_or_url: String,
    },
}

