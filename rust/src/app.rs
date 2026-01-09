use crate::completer::CommandCompleter;
use crate::config::{
    DIRECTORY_LIST_ITEMS, FEEDS, FULL_SCAN_DOWNLOADS, PayloadOption, Paths, CURRENT_VERSION,
};
use crate::compare::Compare;
use crate::custom_feeds::CustomFeedManager;
use crate::dedupe::Dedupe;
use crate::download::Downloader;
use crate::export::{Exporter, ExportFormat};
use crate::file_ops::{handle_payload_report, process_payload_report};
use crate::history::History;
use crate::printer::Printer;
use crate::search::Search;
use crate::stats::Statistics;
use crate::update::UpdateChecker;
use crate::utils::{clear_screen, get_base_dir, get_random_exit_message, get_random_splash, open_file, set_console_title};
use crate::validate::Validator;
use crate::whitelist::WhitelistManager;
use crate::cli::{Commands, FeedCommands, WhitelistCommands};
use anyhow::Result;
use colored::Colorize;
use dialoguer::Input;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::fs;
use std::path::Path;
use std::time::Instant;

pub struct MalScraper {
    paths: Paths,
    downloader: Downloader,
    update_checker: UpdateChecker,
}

impl MalScraper {
    pub async fn new() -> Result<Self> {
        let base_dir = get_base_dir()?;
        Self::new_with_dir(base_dir).await
    }

    pub async fn new_with_dir(base_dir: std::path::PathBuf) -> Result<Self> {
        let paths = Paths::new(base_dir);
        paths.ensure_directories()?;

        let downloader = Downloader::new();
        let update_checker = UpdateChecker::new(paths.clone());

        Ok(Self {
            paths,
            downloader,
            update_checker,
        })
    }

    pub fn print_banner(&self) {
        clear_screen();
        let width = crate::utils::get_terminal_width();

        // ASCII art banner (slant font style matching pyfiglet)
        let banner = r#"                    _______                                
   ____ ___  ____ _/ / ___/______________ _____  ___  _____
  / __ `__ \/ __ `/ /\__ \/ ___/ ___/ __ `/ __ \/ _ \/ ___/
 / / / / / / /_/ / /___/ / /__/ /  / /_/ / /_/ /  __/ /    
/_/ /_/ /_/\__,_/_//____/\___/_/   \__,_/ .___/\___/_/     
                                       /_/                  "#;

        for line in banner.lines() {
            let display_line = if line.len() > width {
                &line[..width]
            } else {
                line
            };
            println!("{}", display_line.cyan().bold());
        }

        // Calculate max label length for consistent padding
        let labels = vec!["Tool", "Author", "Bluesky", "Website", "Github", "Branch", "Version", "Tab Completion"];
        let max_label_len = labels.iter().map(|l| l.len()).max().unwrap_or(15);
        
        // Print all labels with consistent padding
        Printer::label_value_padded("Tool", "malScraper", max_label_len);
        Printer::label_value_padded("Author", "Ryan Monaghan", max_label_len);
        Printer::label_value_padded("Bluesky", "https://bsky.app/profile/rynmon.ie", max_label_len);
        Printer::label_value_padded("Website", "https://rynmon.ie", max_label_len);
        Printer::label_value_padded("Github", "https://github.com/rynmon/malScraper", max_label_len);
        Printer::label_value_padded("Branch", "Stable", max_label_len);
        Printer::label_value_padded("Version", &format!("{} (Rust)", CURRENT_VERSION), max_label_len);
        // Tab Completion uses cyan instead of magenta to match Python version
        let tab_completion_label = "Tab Completion".cyan();
        Printer::label_value_colored("Tab Completion", "Enabled (Rust)", &tab_completion_label, max_label_len);
        println!();
    }

    pub fn print_help(&self) {
        Printer::header("HELP MENU :: Available options shown below:", " :: ");
        println!();

        let menu_items = vec![
            ("Tutorial of how to use this tool", "TUTORIAL", Some("Tutorial")),
            ("Show this Help Menu", "HELP,GET-HELP,?,-?,/?,MENU", Some("Help")),
            ("Clear screen", "CLEAR,CLEAR-HOST,CLS", Some("Clear")),
            ("Return to Home Menu", "HOME,BACK,CD ..", Some("Home")),
            ("Open an existing report", "OPEN,REOPEN", Some("Open")),
            ("Quit malScraper", "QUIT,EXIT", Some("Quit")),
            ("Install the latest update", "INSTALL,UPDATE", Some("Install")),
            (
                "Perform Full-Scan (Note this may take some time)",
                "FULL,FULL-SCAN,FSCAN",
                Some("Full-Scan"),
            ),
            (
                "Perform Quick-Scan (Most recent 100 Payload Domains)",
                "QUICK,QUICK-SCAN,QSCAN",
                Some("Quick-Scan"),
            ),
            (
                "Show Statistics Dashboard",
                "STATS,STATISTICS,DASHBOARD",
                Some("Statistics"),
            ),
            (
                "Search reports for a term",
                "SEARCH",
                Some("Search"),
            ),
            (
                "Filter reports by feed type or pattern",
                "FILTER",
                Some("Filter"),
            ),
            (
                "Export report to firewall/SIEM format",
                "EXPORT",
                Some("Export"),
            ),
            (
                "Deduplicate all reports and create master list",
                "DEDUPE,UNIQUE",
                Some("Deduplicate"),
            ),
            (
                "Compare current scan with previous scan",
                "DIFF,CHANGES",
                Some("Compare"),
            ),
            (
                "Validate IP addresses and domains",
                "VALIDATE",
                Some("Validate"),
            ),
            (
                "Compare two reports side-by-side",
                "COMPARE",
                Some("Compare"),
            ),
            (
                "Manage whitelist (exclude false positives)",
                "WHITELIST",
                Some("whitelist"),
            ),
            (
                "Manage custom feed URLs",
                "FEEDS",
                Some("feed"),
            ),
        ];

        // Find the maximum description length for alignment
        // Add a small buffer to ensure consistent alignment
        let max_desc_len = menu_items
            .iter()
            .map(|(desc, _, _)| desc.len())
            .max()
            .unwrap_or(50)
            + 2; // Add 2 extra spaces for better visual alignment

        for (desc, cmds, highlight) in menu_items {
            Printer::menu_item_aligned(desc, cmds, highlight, max_desc_len);
        }

        println!();
        println!(
            "{} Press {} to auto-complete commands!",
            "💡 Tip:".cyan(),
            "TAB".yellow()
        );
        println!();
    }

    pub fn show_help_menu(&self) {
        clear_screen();
        self.print_help();
    }

    pub fn print_directory_list(&self) {
        clear_screen();
        Printer::success("Success - Files written to:");

        for item in DIRECTORY_LIST_ITEMS.iter() {
            let path = match item.path_key {
                "payload_report" => &self.paths.payload_report,
                "amp_report" => &self.paths.amp_report,
                "c2_report" => &self.paths.c2_report,
                "top_100" => &self.paths.top_100,
                "hex_report" => &self.paths.hex_report,
                "haus_mal_down" => &self.paths.haus_mal_down,
                "phish_tank" => &self.paths.phish_tank,
                _ => continue,
            };
            Printer::directory_item(item.number, item.name, path);
        }
        println!();
    }

    pub fn warn_defender(&self) {
        println!(
            "{} Some reports may be flagged or quarantined by antivirus software (such as Windows Defender) because they contain known malware indicators. These files are for research and defensive use only.",
            "Warning:".yellow().bold()
        );
    }

    pub fn get_payload_option(&self) -> Result<PayloadOption> {
        self.warn_defender();
        println!("\nHow would you like to handle PayloadReport.txt?");
        println!("{}: Leave as is (may be flagged by antivirus)", "1".cyan());
        println!("{}: Obfuscate IOCs (replace http with hxxp)", "2".cyan());
        println!("{}: Save as zip (PayloadReport.zip)", "3".cyan());
        println!("{}: Both obfuscate and zip", "4".cyan());

        loop {
            let choice: String = Input::new()
                .with_prompt("Enter your choice (1-4)")
                .interact_text()?;

            if let Some(option) = PayloadOption::from_choice(&choice) {
                return Ok(option);
            } else {
                Printer::error("Invalid input. Please enter 1, 2, 3, or 4.");
            }
        }
    }

    pub async fn download_with_status(
        &self,
        name: &str,
        feed_key: &str,
        path_key: &str,
        header: Option<&str>,
    ) -> Result<bool> {
        println!("{}:", name);
        if let Some(header) = header {
            fs::write(&self.get_path(path_key)?, header)?;
        }

        let start = Instant::now();
        let url = FEEDS
            .get(feed_key)
            .ok_or_else(|| anyhow::anyhow!("Unknown feed key: {}", feed_key))?;

        match self
            .downloader
            .download_file(url, &self.get_path(path_key)?, Some(name))
            .await
        {
            Ok(_) => {
                let elapsed = start.elapsed().as_secs_f64();
                Printer::success(&format!("Success ({:.1}s)", elapsed));
                Printer::success(&format!("{} saved.\n", self.get_path(path_key)?.display()));
                Ok(true)
            }
            Err(e) => {
                Printer::error(&format!("Failed: {}\n", e));
                self.downloader
                    .cleanup_failed_download(&self.get_path(path_key)?);
                Ok(false)
            }
        }
    }

    fn get_path(&self, path_key: &str) -> Result<&Path> {
        match path_key {
            "payload_report" => Ok(&self.paths.payload_report),
            "amp_report" => Ok(&self.paths.amp_report),
            "c2_report" => Ok(&self.paths.c2_report),
            "top_100" => Ok(&self.paths.top_100),
            "hex_report" => Ok(&self.paths.hex_report),
            "haus_mal_down" => Ok(&self.paths.haus_mal_down),
            "phish_tank" => Ok(&self.paths.phish_tank),
            _ => Err(anyhow::anyhow!("Unknown path key: {}", path_key)),
        }
    }

    pub async fn download_payload_feed_with_options(
        &self,
        option: PayloadOption,
    ) -> Result<(bool, Option<usize>)> {
        Printer::info("Downloading Payload Domains feed...");

        let url = FEEDS
            .get("payload_feed")
            .ok_or_else(|| anyhow::anyhow!("Payload feed URL not found"))?;

        let data = match self.downloader.download_text(url).await {
            Ok(text) => {
                Printer::success("Download complete.");
                tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
                text
            }
            Err(e) => {
                Printer::error(&format!("Failed to download payload report: {}", e));
                return Ok((false, None));
            }
        };

        let line_count = handle_payload_report(&self.paths, data, option)?;
        Ok((true, Some(line_count)))
    }

    pub async fn full_scan(&mut self) -> Result<()> {
        clear_screen();
        println!("{}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
        println!("{}", get_random_splash());

        let payload_option = self.get_payload_option()?;

        // Remove existing reports
        for path in [
            &self.paths.payload_report,
            &self.paths.amp_report,
            &self.paths.c2_report,
            &self.paths.top_100,
            &self.paths.hex_report,
            &self.paths.haus_mal_down,
            &self.paths.phish_tank,
        ] {
            if path.exists() {
                let _ = fs::remove_file(path);
            }
        }

        let mut status = std::collections::HashMap::new();
        let mut payload_line_count = None;

        println!("{}\n", "Starting downloads...".bold());

        // Download standard feeds
        for download in FULL_SCAN_DOWNLOADS.iter() {
            let success = self
                .download_with_status(
                    download.name,
                    download.feed_key,
                    download.path_key,
                    download.header,
                )
                .await?;
            status.insert(download.name.to_string(), success);
        }

        // Payload report (special handling)
        println!("Payload domains:");
        let start = Instant::now();
        match self.download_payload_feed_with_options(payload_option).await? {
            (true, Some(count)) => {
                payload_line_count = Some(count);
                let elapsed = start.elapsed().as_secs_f64();
                Printer::success(&format!("Success ({:.1}s)", elapsed));
                if payload_option == PayloadOption::LeaveAsIs
                    || payload_option == PayloadOption::Obfuscate
                {
                    Printer::success(&format!("{} saved.", self.paths.payload_report.display()));
                }
                if payload_option == PayloadOption::Zip || payload_option == PayloadOption::Both {
                    let zip_path = self.paths.payload_report.with_extension("zip");
                    Printer::success(&format!("{} saved.", zip_path.display()));
                }
                println!();
                if self.paths.payload_report.exists() {
                    process_payload_report(&self.paths)?;
                }
            }
            (false, _) => {
                Printer::error("Failed\n");
                status.insert("Payload domains".to_string(), false);
            }
            _ => {}
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;

        // Print summary
        let succeeded: Vec<_> = status
            .iter()
            .filter(|(_, &v)| v)
            .map(|(k, _)| k.clone())
            .collect();
        let failed: Vec<_> = status
            .iter()
            .filter(|(_, &v)| !v)
            .map(|(k, _)| k.clone())
            .collect();

        println!("{}", "Download Summary:".bold());
        println!("- {}/{} downloads succeeded.", succeeded.len(), status.len());
        if let Some(count) = payload_line_count {
            println!("- Payload domains: {} lines", count);
        }
        if !failed.is_empty() {
            println!("- {} download(s) failed: {}.", failed.len(), failed.join(", "));
        }
        println!();

        if !status.values().all(|&v| v) {
            Printer::warning("Warning: Some downloads may have failed. Check the reports.\n");
        }

        let _: String = Input::new()
            .with_prompt("Press Enter to continue...")
            .allow_empty(true)
            .interact_text()?;

        clear_screen();
        self.print_directory_list();
        self.show_home();
        Ok(())
    }

    pub async fn quick_scan(&mut self) -> Result<()> {
        clear_screen();
        println!("{}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
        println!("{}", get_random_splash());

        let payload_option = self.get_payload_option()?;

        // Remove existing reports
        for path in [&self.paths.payload_report, &self.paths.top_100] {
            if path.exists() {
                let _ = fs::remove_file(path);
            }
        }

        match self.download_payload_feed_with_options(payload_option).await? {
            (true, _) => {
                if self.paths.payload_report.exists() {
                    process_payload_report(&self.paths)?;
                }
            }
            (false, _) => {
                Printer::error("Failed to download payload report.");
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
        self.show_home();
        Ok(())
    }

    pub fn prompt_open_report(&self) -> Result<()> {
        loop {
            clear_screen();
            self.print_directory_list();

            let option: String = Input::new()
                .with_prompt("Which feed would you like to open?")
                .interact_text()?;

            let valid_options: std::collections::HashMap<_, _> = DIRECTORY_LIST_ITEMS
                .iter()
                .map(|item| {
                    (
                        item.number.to_string(),
                        self.get_path(item.path_key).unwrap(),
                    )
                })
                .collect();

            if let Some(path) = valid_options.get(&option) {
                if let Err(e) = open_file(path) {
                    Printer::error(&format!("Error opening file: {}", e));
                }
                break;
            } else if option.to_lowercase() == "home" {
                break;
            } else {
                Printer::error("Invalid Option.");
                let _: String = Input::new()
                    .with_prompt("Press Enter to try again...")
                    .allow_empty(true)
                    .interact_text()?;
            }
        }
        Ok(())
    }

    pub fn reopen(&self) -> Result<()> {
        clear_screen();
        self.print_directory_list();
        self.prompt_open_report()?;
        Ok(())
    }

    pub fn tutorial(&self) {
        clear_screen();
        println!("{}", "MalScraper Tutorial".bold());
        println!();
        println!("{}", "NAME".bold());
        println!(" - malScraper: Scrapes a list of Payload Domains, IOC's & C2 IPs from various feeds for easy blacklisting.");
        println!();
        println!("{}", "SYNOPSIS".bold());
        println!(" - Interactive mode: {} or {}", "malscraper".cyan(), "cargo run".cyan());
        println!(" - Non-interactive mode: {} <command>", "malscraper".cyan());
        println!(" - Example: {}", "malscraper".cyan());
        println!(" - Example: {} {}", "malscraper".cyan(), "quick-scan --output-dir ./reports".cyan());
        println!();
        println!("{}", "DESCRIPTION".bold());
        println!(" - A cross-platform tool for collecting malware information from various feeds.");
        println!(" - Supports interactive and non-interactive (CLI) modes for automation.");
        println!();
        println!("{}", "BASIC WORKFLOW".bold());
        println!("  1. Run {} for a fast check of the most recent 100 domains.", "Quick-Scan".cyan());
        println!("  2. Run {} to gather comprehensive data from all sources.", "Full-Scan".cyan());
        println!("  3. Use the numbered menu to open specific reports.");
        println!("  4. Reports are saved to your {} (Mac/Linux) or {} (Windows) folder.", "Desktop".cyan(), "Documents".cyan());
        println!();
        println!("{}", "ANALYSIS FEATURES".bold());
        println!("  - {}: View statistics dashboard with report metrics", "STATS".cyan());
        println!("  - {}: Search for specific terms across all reports", "SEARCH <term>".cyan());
        println!("  - {}: Filter reports by feed type or pattern", "FILTER [feed_type] [pattern]".cyan());
        println!("  - {}: Compare two reports side-by-side", "COMPARE <report1> <report2>".cyan());
        println!("  - {}: Compare current scan with previous scan", "DIFF".cyan());
        println!("  - {}: Validate IP addresses and domains", "VALIDATE <report>".cyan());
        println!();
        println!("{}", "DATA MANAGEMENT".bold());
        println!("  - {}: Deduplicate all reports into a master list", "DEDUPE".cyan());
        println!("  - {}: Export reports to firewall/SIEM formats", "EXPORT <format> <report>".cyan());
        println!("     Formats: iptables, windows, pfsense, json, csv, stix, taxii");
        println!("  - {}: Manage whitelist to exclude false positives", "WHITELIST".cyan());
        println!("     Commands: ADD <indicator> [reason], LIST, REMOVE <indicator>");
        println!("  - {}: Manage custom feed URLs", "FEEDS".cyan());
        println!("     Commands: ADD <url> [name] [description], LIST, REMOVE <name_or_url>");
        println!();
        println!("{}", "MENU NAVIGATION".bold());
        println!(" - Type {} to see available commands.", "HELP".cyan());
        println!(" - Type {} to view this tutorial again.", "TUTORIAL".cyan());
        println!(" - Type {} to check for updates.", "UPDATE".cyan());
        println!(" - Type {} to exit the application.", "QUIT".cyan());
        println!();
        println!("{}", "NON-INTERACTIVE MODE".bold());
        println!(" - Use CLI arguments for automation and scripting");
        println!(" - Example: {} {}", "malscraper".cyan(), "quick-scan --output-dir ./reports".cyan());
        println!(" - Example: {} {}", "malscraper".cyan(), "export iptables payload".cyan());
        println!(" - Example: {} {}", "malscraper".cyan(), "search malware.com".cyan());
        println!(" - Run {} {} to see all available commands", "malscraper".cyan(), "--help".cyan());
        println!();
    }

    pub fn show_home(&self) {
        self.print_banner();
        self.print_help();
    }

    pub async fn process_command(&mut self, command: &str) -> Result<()> {
        let command_upper = command.to_uppercase().trim().to_string();

        match command_upper.as_str() {
            "FULL" | "FULL-SCAN" | "FSCAN" => {
                self.full_scan().await?;
            }
            "QUICK" | "QUICK-SCAN" | "QSCAN" => {
                self.quick_scan().await?;
            }
            "QUIT" | "EXIT" => {
                println!("\n{}", get_random_exit_message());
                std::process::exit(0);
            }
            "CLEAR" | "CLEAR-HOST" | "CLS" => {
                clear_screen();
            }
            "HELP" | "GET-HELP" | "?" | "-?" | "/?" | "MENU" => {
                self.show_help_menu();
            }
            "BACK" | "CD .." | "HOME" => {
                self.show_home();
            }
            "TUTORIAL" => {
                self.tutorial();
            }
            "REOPEN" | "OPEN" => {
                self.reopen()?;
            }
            "INSTALL" | "UPDATE" => {
                // Force update check
                match self
                    .update_checker
                    .check_for_updates(CURRENT_VERSION, true)
                    .await
                {
                    Ok(Some(update_info)) => {
                        // Install the update
                        match self.update_checker.install_update(update_info).await {
                            Ok(()) => {
                                Printer::info("\nUpdate installed! The application will now exit.");
                                Printer::info("Please restart to use the new version.");
                                std::thread::sleep(std::time::Duration::from_secs(2));
                                std::process::exit(0);
                            }
                            Err(e) => {
                                Printer::error(&format!("Failed to install update: {}", e));
                                Printer::info("You can manually download from:");
                                Printer::info("https://github.com/rynmon/malScraper/releases");
                            }
                        }
                    }
                    Ok(None) => {
                        Printer::success("You are running the latest version!");
                    }
                    Err(e) => {
                        Printer::error(&format!("Failed to check for updates: {}", e));
                    }
                }
            }
            "STATS" | "STATISTICS" | "DASHBOARD" => {
                if let Err(e) = Statistics::display_dashboard(&self.paths) {
                    Printer::error(&format!("Failed to generate statistics: {}", e));
                }
            }
            "SEARCH" => {
                Printer::error("Usage: SEARCH <term>");
                Printer::info("Example: SEARCH example.com");
                Printer::info("Example: SEARCH malware");
            }
            cmd if cmd.starts_with("SEARCH ") => {
                let term = cmd.strip_prefix("SEARCH ").unwrap_or("").trim();
                if term.is_empty() {
                    Printer::error("Usage: SEARCH <term>");
                    Printer::info("Example: SEARCH example.com");
                } else {
                    match Search::search_all_reports(&self.paths, term, false) {
                        Ok(results) => Search::display_results(&results, term),
                        Err(e) => Printer::error(&format!("Search failed: {}", e)),
                    }
                }
            }
            "FILTER" => {
                Printer::error("Usage: FILTER [feed_type] [pattern]");
                Printer::info("Feed types: payload, amp, c2, hex, haus, phish, top100");
                Printer::info("Example: FILTER payload example.com");
                Printer::info("Example: FILTER c2");
                Printer::info("Example: FILTER .exe");
            }
            cmd if cmd.starts_with("FILTER ") => {
                let filter_args = cmd.strip_prefix("FILTER ").unwrap_or("").trim();
                if filter_args.is_empty() {
                    Printer::error("Usage: FILTER [feed_type] [pattern]");
                    Printer::info("Feed types: payload, amp, c2, hex, haus, phish, top100");
                    Printer::info("Example: FILTER payload example.com");
                    Printer::info("Example: FILTER c2");
                    Printer::info("Example: FILTER .exe");
                } else {
                    let args: Vec<&str> = filter_args.split_whitespace().collect();
                    // Try to determine if first arg is a feed type
                    let known_feeds = ["payload", "payloads", "amp", "c2", "c2servers", "c2-servers", 
                                       "hex", "hashes", "haus", "urlhaus", "url-haus", 
                                       "phish", "phishtank", "phish-tank", "top100", "top", "top-100"];
                    
                    let (feed_type, pattern) = if args.len() > 1 {
                        // Two or more args: first is feed type, rest is pattern
                        let feed = if known_feeds.contains(&args[0].to_lowercase().as_str()) {
                            Some(args[0])
                        } else {
                            None
                        };
                        let pat = Some(args[1..].join(" "));
                        (feed, pat)
                    } else {
                        // Single arg: could be feed type or pattern
                        if known_feeds.contains(&args[0].to_lowercase().as_str()) {
                            (Some(args[0]), None)
                        } else {
                            (None, Some(args[0].to_string()))
                        }
                    };

                    match Search::filter_reports(&self.paths, feed_type, pattern.as_deref()) {
                        Ok(results) => Search::display_filter_results(&results, feed_type, pattern.as_deref()),
                        Err(e) => Printer::error(&format!("Filter failed: {}", e)),
                    }
                }
            }
            "EXPORT" => {
                Printer::error("Usage: EXPORT <format> <report> [output_file]");
                Printer::info("Formats: iptables, windows, pfsense, json, csv, stix, taxii");
                Printer::info("Reports: payload, amp, c2, hex, haus, phish, top100");
                Printer::info("Example: EXPORT json payload");
                Printer::info("Example: EXPORT iptables c2 firewall_rules.sh");
            }
            cmd if cmd.starts_with("EXPORT ") => {
                let export_args = cmd.strip_prefix("EXPORT ").unwrap_or("").trim();
                if export_args.is_empty() {
                    Printer::error("Usage: EXPORT <format> <report> [output_file]");
                    Printer::info("Formats: iptables, windows, pfsense, json, csv, stix, taxii");
                    Printer::info("Reports: payload, amp, c2, hex, haus, phish, top100");
                    Printer::info("Example: EXPORT json payload");
                    Printer::info("Example: EXPORT iptables c2 firewall_rules.sh");
                } else {
                    let args: Vec<&str> = export_args.split_whitespace().collect();
                    if args.len() < 2 {
                        Printer::error("Usage: EXPORT <format> <report> [output_file]");
                        Printer::info("Example: EXPORT json payload");
                    } else {
                        let format_str = args[0];
                        let report_name = args[1];
                        let output_file = args.get(2).copied();

                        match ExportFormat::from_str(format_str) {
                            Some(format) => {
                                Printer::info(&format!("Exporting {} to {} format...", report_name, format.as_str()));
                                match Exporter::export(&self.paths, format, report_name, output_file) {
                                    Ok(output_path) => {
                                        Printer::success(&format!("Export completed: {}", output_path));
                                        Printer::info("File saved to current directory");
                                    }
                                    Err(e) => {
                                        Printer::error(&format!("Export failed: {}", e));
                                    }
                                }
                            }
                            None => {
                                Printer::error(&format!("Unknown export format: {}", format_str));
                                Printer::info("Valid formats: iptables, windows, pfsense, json, csv, stix, taxii");
                            }
                        }
                    }
                }
            }
            "DEDUPE" | "UNIQUE" => {
                // DEDUPE can be called without arguments, so execute it
                match Dedupe::deduplicate_all_reports(&self.paths, None) {
                    Ok(output_path) => {
                        Printer::success(&format!("Deduplication completed: {}", output_path));
                        Printer::info("Master list saved to current directory");
                    }
                    Err(e) => {
                        Printer::error(&format!("Deduplication failed: {}", e));
                        Printer::info("Usage: DEDUPE [output_file]");
                        Printer::info("Example: DEDUPE");
                        Printer::info("Example: DEDUPE MasterList.txt");
                    }
                }
            }
            cmd if cmd.starts_with("DEDUPE ") || cmd.starts_with("UNIQUE ") => {
                let args: Vec<&str> = command.split_whitespace().collect();
                let output_file = args.get(1).copied();

                match Dedupe::deduplicate_all_reports(&self.paths, output_file) {
                    Ok(output_path) => {
                        Printer::success(&format!("Deduplication completed: {}", output_path));
                        Printer::info("Master list saved to current directory");
                    }
                    Err(e) => {
                        Printer::error(&format!("Deduplication failed: {}", e));
                        Printer::info("Usage: DEDUPE [output_file]");
                        Printer::info("Example: DEDUPE");
                        Printer::info("Example: DEDUPE MasterList.txt");
                    }
                }
            }
            "DIFF" | "CHANGES" => {
                match History::compare_with_history(&self.paths) {
                    Ok(_) => {
                        // History::compare_with_history handles all output
                    }
                    Err(e) => {
                        Printer::error(&format!("Failed to compare with history: {}", e));
                        Printer::info("Usage: DIFF or CHANGES");
                        Printer::info("Example: DIFF");
                    }
                }
            }
            "VALIDATE" => {
                Printer::error("Usage: VALIDATE <report>");
                Printer::info("Reports: payload, amp, c2, hex, haus, phish, top100");
                Printer::info("Example: VALIDATE payload");
            }
            cmd if cmd.starts_with("VALIDATE ") => {
                let report_name = cmd.strip_prefix("VALIDATE ").unwrap_or("").trim();
                if report_name.is_empty() {
                    Printer::error("Usage: VALIDATE <report>");
                    Printer::info("Reports: payload, amp, c2, hex, haus, phish, top100");
                    Printer::info("Example: VALIDATE payload");
                } else {
                    match Validator::validate_report(&self.paths, report_name).await {
                        Ok(results) => {
                            Validator::display_results(&results, report_name);
                        }
                        Err(e) => {
                            Printer::error(&format!("Validation failed: {}", e));
                        }
                    }
                }
            }
            "COMPARE" => {
                Printer::error("Usage: COMPARE <report1> <report2>");
                Printer::info("Example: COMPARE payload c2");
                Printer::info("Available reports: payload, amp, c2, top100, hex, haus, phish");
            }
            cmd if cmd.starts_with("COMPARE ") => {
                let args: Vec<&str> = cmd.strip_prefix("COMPARE ").unwrap_or("").trim().split_whitespace().collect();
                if args.len() < 2 {
                    Printer::error("Usage: COMPARE <report1> <report2>");
                    Printer::info("Example: COMPARE payload c2");
                    Printer::info("Available reports: payload, amp, c2, top100, hex, haus, phish");
                } else {
                    match Compare::compare_reports(&self.paths, args[0], args[1]) {
                        Ok(results) => {
                            Compare::display_results(&results);
                        }
                        Err(e) => {
                            Printer::error(&format!("Comparison failed: {}", e));
                        }
                    }
                }
            }
            "WHITELIST" => {
                Printer::error("Usage: WHITELIST <command>");
                Printer::info("Commands: ADD <indicator> [reason], LIST, REMOVE <indicator>");
                Printer::info("Example: WHITELIST ADD example.com \"False positive\"");
                Printer::info("Example: WHITELIST LIST");
                Printer::info("Example: WHITELIST REMOVE example.com");
            }
            cmd if cmd.starts_with("WHITELIST ") => {
                let whitelist_args = cmd.strip_prefix("WHITELIST ").unwrap_or("").trim();
                if whitelist_args.is_empty() {
                    Printer::error("Usage: WHITELIST <command>");
                    Printer::info("Commands: ADD <indicator> [reason], LIST, REMOVE <indicator>");
                    Printer::info("Example: WHITELIST ADD example.com \"False positive\"");
                    Printer::info("Example: WHITELIST LIST");
                    Printer::info("Example: WHITELIST REMOVE example.com");
                } else {
                    let args: Vec<&str> = whitelist_args.split_whitespace().collect();
                    match args[0].to_uppercase().as_str() {
                        "ADD" => {
                            if args.len() < 2 {
                                Printer::error("Usage: WHITELIST ADD <indicator> [reason]");
                                Printer::info("Example: WHITELIST ADD example.com \"False positive\"");
                            } else {
                                let indicator = args[1];
                                let reason = if args.len() > 2 {
                                    Some(args[2..].join(" "))
                                } else {
                                    None
                                };

                                match WhitelistManager::add_indicator(&self.paths, indicator, reason) {
                                    Ok(_) => {
                                        // Success message is printed by add_indicator
                                    }
                                    Err(e) => {
                                        Printer::error(&format!("Failed to add to whitelist: {}", e));
                                    }
                                }
                            }
                        }
                        "LIST" => {
                            if let Err(e) = WhitelistManager::list_indicators(&self.paths) {
                                Printer::error(&format!("Failed to list whitelist: {}", e));
                            }
                        }
                        "REMOVE" | "DELETE" => {
                            if args.len() < 2 {
                                Printer::error("Usage: WHITELIST REMOVE <indicator>");
                                Printer::info("Example: WHITELIST REMOVE example.com");
                            } else {
                                let indicator = args[1..].join(" ");
                                match WhitelistManager::remove_indicator(&self.paths, &indicator) {
                                    Ok(_) => {
                                        // Success message is printed by remove_indicator
                                    }
                                    Err(e) => {
                                        Printer::error(&format!("Failed to remove from whitelist: {}", e));
                                    }
                                }
                            }
                        }
                        _ => {
                            Printer::error(&format!("Unknown command: {}", args[0]));
                            Printer::info("Valid commands: ADD, LIST, REMOVE");
                        }
                    }
                }
            }
            "FEEDS" => {
                Printer::error("Usage: FEEDS <command>");
                Printer::info("Commands: ADD <url> [name] [description], LIST, REMOVE <name_or_url>");
                Printer::info("Example: FEEDS ADD https://example.com/feed.txt");
                Printer::info("Example: FEEDS LIST");
                Printer::info("Example: FEEDS REMOVE \"My Feed\"");
            }
            cmd if cmd.starts_with("FEEDS ") => {
                let feeds_args = cmd.strip_prefix("FEEDS ").unwrap_or("").trim();
                if feeds_args.is_empty() {
                    Printer::error("Usage: FEEDS <command>");
                    Printer::info("Commands: ADD <url> [name] [description], LIST, REMOVE <name_or_url>");
                    Printer::info("Example: FEEDS ADD https://example.com/feed.txt");
                    Printer::info("Example: FEEDS LIST");
                } else {
                    let args: Vec<&str> = feeds_args.split_whitespace().collect();
                    match args[0].to_uppercase().as_str() {
                        "ADD" => {
                            if args.len() < 2 {
                                Printer::error("Usage: FEEDS ADD <url> [name] [description]");
                                Printer::info("Example: FEEDS ADD https://example.com/feed.txt \"My Feed\" \"Custom threat feed\"");
                            } else {
                                let url = args[1];
                                let name = args.get(2).copied();
                                let description = if args.len() > 3 {
                                    Some(args[3..].join(" "))
                                } else {
                                    None
                                };

                                match CustomFeedManager::add_feed(&self.paths, url, name, description) {
                                    Ok(_) => {
                                        Printer::success("Custom feed added successfully!");
                                    }
                                    Err(e) => {
                                        Printer::error(&format!("Failed to add feed: {}", e));
                                    }
                                }
                            }
                        }
                        "LIST" => {
                            if let Err(e) = CustomFeedManager::list_feeds(&self.paths) {
                                Printer::error(&format!("Failed to list feeds: {}", e));
                            }
                        }
                        "REMOVE" | "DELETE" => {
                            if args.len() < 2 {
                                Printer::error("Usage: FEEDS REMOVE <name_or_url>");
                                Printer::info("Example: FEEDS REMOVE \"My Feed\"");
                            } else {
                                let name_or_url = args[1..].join(" ");
                                match CustomFeedManager::remove_feed(&self.paths, &name_or_url) {
                                    Ok(_) => {
                                        Printer::success("Custom feed removed successfully!");
                                    }
                                    Err(e) => {
                                        Printer::error(&format!("Failed to remove feed: {}", e));
                                    }
                                }
                            }
                        }
                        _ => {
                            Printer::error(&format!("Unknown command: {}", args[0]));
                            Printer::info("Valid commands: ADD, LIST, REMOVE");
                        }
                    }
                }
            }
            _ => {
                Printer::error(&format!("Unknown command: '{}'", command));
                Printer::info("Type HELP to see all available commands");
                Printer::info("Type TUTORIAL for usage examples");
                println!();
            }
        }

        Ok(())
    }

    pub async fn handle_cli_command(&mut self, command: Commands) -> Result<()> {
        match command {
            Commands::QuickScan { output_dir } => {
                if let Some(dir) = output_dir {
                    self.paths = Paths::new(dir);
                    self.paths.ensure_directories()?;
                }
                // Use default payload option for non-interactive mode
                self.quick_scan_non_interactive(PayloadOption::LeaveAsIs).await?;
            }
            Commands::FullScan { output_dir } => {
                if let Some(dir) = output_dir {
                    self.paths = Paths::new(dir);
                    self.paths.ensure_directories()?;
                }
                // Use default payload option for non-interactive mode
                self.full_scan_non_interactive(PayloadOption::LeaveAsIs).await?;
            }
            Commands::Stats => {
                Statistics::display_dashboard(&self.paths)?;
            }
            Commands::Search { term } => {
                match Search::search_all_reports(&self.paths, &term, false) {
                    Ok(results) => Search::display_results(&results, &term),
                    Err(e) => {
                        Printer::error(&format!("Search failed: {}", e));
                        return Err(e);
                    }
                }
            }
            Commands::Filter { feed_type, pattern } => {
                match Search::filter_reports(
                    &self.paths,
                    feed_type.as_deref(),
                    pattern.as_deref(),
                ) {
                    Ok(results) => {
                        Search::display_filter_results(
                            &results,
                            feed_type.as_deref(),
                            pattern.as_deref(),
                        );
                    }
                    Err(e) => {
                        Printer::error(&format!("Filter failed: {}", e));
                        return Err(e);
                    }
                }
            }
            Commands::Export { format, report, output } => {
                let format_enum = ExportFormat::from_str(&format)
                    .ok_or_else(|| anyhow::anyhow!("Unknown export format: {}", format))?;
                let output_str = output.as_ref().and_then(|p| p.to_str());
                match Exporter::export(&self.paths, format_enum, &report, output_str) {
                    Ok(output_path) => {
                        Printer::success(&format!("Export completed: {}", output_path));
                    }
                    Err(e) => {
                        Printer::error(&format!("Export failed: {}", e));
                        return Err(e);
                    }
                }
            }
            Commands::Dedupe { output } => {
                let output_str = output.as_ref().and_then(|p| p.to_str());
                match Dedupe::deduplicate_all_reports(&self.paths, output_str) {
                    Ok(output_path) => {
                        Printer::success(&format!("Deduplication completed: {}", output_path));
                    }
                    Err(e) => {
                        Printer::error(&format!("Deduplication failed: {}", e));
                        return Err(e);
                    }
                }
            }
            Commands::Diff => {
                History::compare_with_history(&self.paths)?;
            }
            Commands::Compare { report1, report2 } => {
                match Compare::compare_reports(&self.paths, &report1, &report2) {
                    Ok(results) => {
                        Compare::display_results(&results);
                    }
                    Err(e) => {
                        Printer::error(&format!("Comparison failed: {}", e));
                        return Err(e);
                    }
                }
            }
            Commands::Whitelist { command } => {
                match command {
                    WhitelistCommands::Add { indicator, reason } => {
                        match WhitelistManager::add_indicator(&self.paths, &indicator, reason) {
                            Ok(_) => {
                                // Success message is printed by add_indicator
                            }
                            Err(e) => {
                                Printer::error(&format!("Failed to add to whitelist: {}", e));
                                return Err(e);
                            }
                        }
                    }
                    WhitelistCommands::List => {
                        WhitelistManager::list_indicators(&self.paths)?;
                    }
                    WhitelistCommands::Remove { indicator } => {
                        match WhitelistManager::remove_indicator(&self.paths, &indicator) {
                            Ok(_) => {
                                // Success message is printed by remove_indicator
                            }
                            Err(e) => {
                                Printer::error(&format!("Failed to remove from whitelist: {}", e));
                                return Err(e);
                            }
                        }
                    }
                }
            }
            Commands::Validate { report } => {
                match Validator::validate_report(&self.paths, &report).await {
                    Ok(results) => {
                        Validator::display_results(&results, &report);
                    }
                    Err(e) => {
                        Printer::error(&format!("Validation failed: {}", e));
                        return Err(e);
                    }
                }
            }
            Commands::Feeds { command } => {
                match command {
                    FeedCommands::Add { url, name, description } => {
                        match CustomFeedManager::add_feed(
                            &self.paths,
                            &url,
                            name.as_deref(),
                            description,
                        ) {
                            Ok(_) => {
                                Printer::success("Custom feed added successfully!");
                            }
                            Err(e) => {
                                Printer::error(&format!("Failed to add feed: {}", e));
                                return Err(e);
                            }
                        }
                    }
                    FeedCommands::List => {
                        CustomFeedManager::list_feeds(&self.paths)?;
                    }
                    FeedCommands::Remove { name_or_url } => {
                        match CustomFeedManager::remove_feed(&self.paths, &name_or_url) {
                            Ok(_) => {
                                Printer::success("Custom feed removed successfully!");
                            }
                            Err(e) => {
                                Printer::error(&format!("Failed to remove feed: {}", e));
                                return Err(e);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    async fn full_scan_non_interactive(&mut self, payload_option: PayloadOption) -> Result<()> {
        clear_screen();
        println!("{}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
        println!("{}", get_random_splash());

        // Remove existing reports
        for path in [
            &self.paths.payload_report,
            &self.paths.amp_report,
            &self.paths.c2_report,
            &self.paths.top_100,
            &self.paths.hex_report,
            &self.paths.haus_mal_down,
            &self.paths.phish_tank,
        ] {
            if path.exists() {
                let _ = fs::remove_file(path);
            }
        }

        let mut status = std::collections::HashMap::new();
        let mut payload_line_count = None;

        println!("{}\n", "Starting downloads...".bold());

        // Download standard feeds
        for download in FULL_SCAN_DOWNLOADS.iter() {
            let success = self
                .download_with_status(
                    download.name,
                    download.feed_key,
                    download.path_key,
                    download.header,
                )
                .await?;
            status.insert(download.name.to_string(), success);
        }

        // Payload report (special handling)
        println!("Payload domains:");
        let start = Instant::now();
        match self.download_payload_feed_with_options(payload_option).await? {
            (true, Some(count)) => {
                payload_line_count = Some(count);
                let elapsed = start.elapsed().as_secs_f64();
                Printer::success(&format!("Success ({:.1}s)", elapsed));
                if payload_option == PayloadOption::LeaveAsIs
                    || payload_option == PayloadOption::Obfuscate
                {
                    Printer::success(&format!("{} saved.", self.paths.payload_report.display()));
                }
                if payload_option == PayloadOption::Zip || payload_option == PayloadOption::Both {
                    let zip_path = self.paths.payload_report.with_extension("zip");
                    Printer::success(&format!("{} saved.", zip_path.display()));
                }
                println!();
                if self.paths.payload_report.exists() {
                    process_payload_report(&self.paths)?;
                }
            }
            (false, _) => {
                Printer::error("Failed\n");
                status.insert("Payload domains".to_string(), false);
            }
            _ => {}
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;

        // Print summary
        let succeeded: Vec<_> = status
            .iter()
            .filter(|(_, &v)| v)
            .map(|(k, _)| k.clone())
            .collect();
        let failed: Vec<_> = status
            .iter()
            .filter(|(_, &v)| !v)
            .map(|(k, _)| k.clone())
            .collect();

        println!("{}", "Download Summary:".bold());
        println!("- {}/{} downloads succeeded.", succeeded.len(), status.len());
        if let Some(count) = payload_line_count {
            println!("- Payload domains: {} lines", count);
        }
        if !failed.is_empty() {
            println!("- {} download(s) failed: {}.", failed.len(), failed.join(", "));
        }
        println!();

        if !status.values().all(|&v| v) {
            Printer::warning("Warning: Some downloads may have failed. Check the reports.\n");
        }

        Ok(())
    }

    async fn quick_scan_non_interactive(&mut self, payload_option: PayloadOption) -> Result<()> {
        clear_screen();
        println!("{}", chrono::Local::now().format("%Y-%m-%d %H:%M:%S"));
        println!("{}", get_random_splash());

        // Remove existing reports
        for path in [&self.paths.payload_report, &self.paths.top_100] {
            if path.exists() {
                let _ = fs::remove_file(path);
            }
        }

        match self.download_payload_feed_with_options(payload_option).await? {
            (true, _) => {
                if self.paths.payload_report.exists() {
                    process_payload_report(&self.paths)?;
                }
            }
            (false, _) => {
                Printer::error("Failed to download payload report.");
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        // Set console window title
        set_console_title("malScraper");
        
        self.paths.ensure_directories()?;

        // Check for updates (silently, only once per day)
        if self.update_checker.should_check_for_updates() {
            if let Ok(Some(update_info)) = self
                .update_checker
                .check_for_updates(CURRENT_VERSION, false)
                .await
            {
                // User confirmed they want to update, install it
                match self.update_checker.install_update(update_info).await {
                    Ok(()) => {
                        Printer::info("\nUpdate installed! The application will now exit.");
                        Printer::info("Please restart to use the new version.");
                        std::thread::sleep(std::time::Duration::from_secs(2));
                        std::process::exit(0);
                    }
                    Err(e) => {
                        Printer::error(&format!("Failed to install update: {}", e));
                        Printer::info("You can manually download from:");
                        Printer::info("https://github.com/rynmon/malScraper/releases");
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    }
                }
            }
        }

        self.show_home();

        // Create rustyline editor with tab completion
        let config = rustyline::Config::builder()
            .completion_type(rustyline::CompletionType::List)
            .build();
        let completer = CommandCompleter::new();
        let mut rl = Editor::with_config(config)?;
        rl.set_helper(Some(completer));

        // Main loop with tab completion
        loop {
            let readline = rl.readline("malScraper> ");
            match readline {
                Ok(line) => {
                    let command = line.trim();
                    if command.is_empty() {
                        continue;
                    }
                    
                    // Add to history
                    let _ = rl.add_history_entry(command);
                    
                    if let Err(e) = self.process_command(command).await {
                        Printer::error(&format!("\nUnexpected error: {}", e));
                        println!("The application will continue running. If this error persists, please restart.");
                        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                    }
                }
                Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                    // Ctrl+C or Ctrl+D
                    println!("\n{}", get_random_exit_message());
                    break;
                }
                Err(err) => {
                    Printer::error(&format!("Error: {:?}", err));
                    break;
                }
            }
        }

        Ok(())
    }
}

