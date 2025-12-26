use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

pub const REQUEST_TIMEOUT_SECS: u64 = 30;
pub const UPDATE_CHECK_TIMEOUT_SECS: u64 = 10;
pub const TOP_DOMAINS_COUNT: usize = 100;
// Automatically sync version from Cargo.toml at compile time
pub const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub static FEEDS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    m.insert("payload_feed", "https://urlhaus.abuse.ch/downloads/text/");
    m.insert("c2_feed", "http://cybercrime-tracker.net/all.php");
    m.insert(
        "hex_feed",
        "https://raw.githubusercontent.com/Neo23x0/signature-base/master/iocs/hash-iocs.txt",
    );
    m.insert(
        "phish_tank",
        "https://data.phishtank.com/data/online-valid.csv",
    );
    m.insert("haus_mal_down", "https://urlhaus.abuse.ch/downloads/csv/");
    m
});

pub static RELEASE_URL: &str =
    "https://api.github.com/repos/Ryan-Monaghan/malScraper/releases/latest";

#[derive(Debug, Clone)]
pub struct Paths {
    pub base_dir: PathBuf,
    pub payload_report: PathBuf,
    pub amp_report: PathBuf,
    pub c2_report: PathBuf,
    pub top_100: PathBuf,
    pub hex_report: PathBuf,
    pub haus_mal_down: PathBuf,
    pub phish_tank: PathBuf,
    pub updates_dir: PathBuf,
}

impl Paths {
    pub fn new(base_dir: PathBuf) -> Self {
        Self {
            payload_report: base_dir.join("PayloadReport.txt"),
            amp_report: base_dir.join("AMPReport.txt"),
            c2_report: base_dir.join("C2Report.txt"),
            top_100: base_dir.join("Top100.txt"),
            hex_report: base_dir.join("HexReport.csv"),
            haus_mal_down: base_dir.join("HausMalDown.csv"),
            phish_tank: base_dir.join("Phishing").join("PhishTank.csv"),
            updates_dir: base_dir.join("Updates"),
            base_dir,
        }
    }

    pub fn ensure_directories(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.base_dir)?;
        std::fs::create_dir_all(self.phish_tank.parent().unwrap())?;
        std::fs::create_dir_all(&self.updates_dir)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PayloadOption {
    LeaveAsIs,
    Obfuscate,
    Zip,
    Both,
}

impl PayloadOption {
    pub fn from_choice(choice: &str) -> Option<Self> {
        match choice {
            "1" => Some(Self::LeaveAsIs),
            "2" => Some(Self::Obfuscate),
            "3" => Some(Self::Zip),
            "4" => Some(Self::Both),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FullScanDownload {
    pub name: &'static str,
    pub feed_key: &'static str,
    pub path_key: &'static str,
    pub header: Option<&'static str>,
}

pub static FULL_SCAN_DOWNLOADS: Lazy<Vec<FullScanDownload>> = Lazy::new(|| {
    vec![
        FullScanDownload {
            name: "C2 servers report",
            feed_key: "c2_feed",
            path_key: "c2_report",
            header: Some(
                "#############################################\n# C2 Servers Report sourced from http://cybercrime-tracker.net/ \n#############################################\n",
            ),
        },
        FullScanDownload {
            name: "Hex report",
            feed_key: "hex_feed",
            path_key: "hex_report",
            header: None,
        },
        FullScanDownload {
            name: "URLHaus Malware downloads",
            feed_key: "haus_mal_down",
            path_key: "haus_mal_down",
            header: None,
        },
        FullScanDownload {
            name: "PhishTank data",
            feed_key: "phish_tank",
            path_key: "phish_tank",
            header: None,
        },
    ]
});

#[derive(Debug, Clone)]
pub struct DirectoryItem {
    pub number: u8,
    pub name: &'static str,
    pub path_key: &'static str,
}

pub static DIRECTORY_LIST_ITEMS: Lazy<Vec<DirectoryItem>> = Lazy::new(|| {
    vec![
        DirectoryItem {
            number: 1,
            name: "Payload Domains",
            path_key: "payload_report",
        },
        DirectoryItem {
            number: 2,
            name: "AMP Report",
            path_key: "amp_report",
        },
        DirectoryItem {
            number: 3,
            name: "C2 Servers",
            path_key: "c2_report",
        },
        DirectoryItem {
            number: 4,
            name: "Hex Report",
            path_key: "hex_report",
        },
        DirectoryItem {
            number: 5,
            name: "URLHaus Maldownloads",
            path_key: "haus_mal_down",
        },
        DirectoryItem {
            number: 6,
            name: "PhishTank Phishing Pages",
            path_key: "phish_tank",
        },
        DirectoryItem {
            number: 7,
            name: "Most Recent 100",
            path_key: "top_100",
        },
    ]
});

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub new_script_path: String,
    pub temp_dir: String,
    pub backup_path: String,
    pub version: String,
}

