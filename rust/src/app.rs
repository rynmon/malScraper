use crate::completer::CommandCompleter;
use crate::config::{
    DIRECTORY_LIST_ITEMS, FEEDS, FULL_SCAN_DOWNLOADS, PayloadOption, Paths, CURRENT_VERSION,
};
use crate::download::Downloader;
use crate::file_ops::{handle_payload_report, process_payload_report};
use crate::printer::Printer;
use crate::update::UpdateChecker;
use crate::utils::{clear_screen, get_base_dir, get_random_exit_message, get_random_splash, open_file, set_console_title};
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
        println!(" - Run: {} or {}", "malscraper".cyan(), "cargo run".cyan());
        println!(" - Example: {}", "malscraper".cyan());
        println!();
        println!("{}", "DESCRIPTION".bold());
        println!(" - A cross-platform tool for collecting malware information from various feeds.");
        println!();
        println!("{}", "WORKFLOW".bold());
        println!("  1. Run {} for a fast check of the most recent 100 domains.", "Quick-Scan".cyan());
        println!("  2. Run {} to gather comprehensive data from all sources.", "Full-Scan".cyan());
        println!("  3. Use the numbered menu to open specific reports.");
        println!("  4. Reports are saved to your {} (Mac/Linux) or {} (Windows) folder.", "Desktop".cyan(), "Documents".cyan());
        println!();
        println!("{}", "MENU NAVIGATION".bold());
        println!(" - Type {} to see available commands.", "HELP".cyan());
        println!(" - Type {} to view this tutorial again.", "TUTORIAL".cyan());
        println!(" - Type {} to check for updates.", "UPDATE".cyan());
        println!(" - Type {} to exit the application.", "QUIT".cyan());
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
                    Ok(Some(_update_info)) => {
                        Printer::info("Update available! Please download the latest release from:");
                        Printer::info("https://github.com/rynmon/malScraper/releases");
                        Printer::warning("Automatic update installation coming in a future release.");
                    }
                    Ok(None) => {
                        Printer::success("You are running the latest version!");
                    }
                    Err(e) => {
                        Printer::error(&format!("Failed to check for updates: {}", e));
                    }
                }
            }
            _ => {
                clear_screen();
                Printer::error("Error - invalid operation\n");
                self.print_help();
            }
        }

        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        // Set console window title
        set_console_title("malScraper");
        
        self.paths.ensure_directories()?;

        // Check for updates (silently, only once per day)
        if self.update_checker.should_check_for_updates() {
            if let Ok(Some(_update_info)) = self
                .update_checker
                .check_for_updates(CURRENT_VERSION, false)
                .await
            {
                // Update was detected and user was prompted
                // The update checker handles the prompt and display
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

