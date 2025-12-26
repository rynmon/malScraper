use crate::config::{Paths, PayloadOption, TOP_DOMAINS_COUNT};
use crate::printer::Printer;
use anyhow::{Context, Result};
use regex::Regex;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use zip::write::{FileOptions, ZipWriter};
use zip::CompressionMethod;

pub fn process_payload_report(paths: &Paths) -> Result<()> {
    // Create AMP report by extracting domains
    let payload_file = File::open(&paths.payload_report)
        .context("Failed to open payload report")?;
    let reader = BufReader::new(payload_file);

    let mut amp_file = File::create(&paths.amp_report)
        .context("Failed to create AMP report")?;

    let domain_regex = Regex::new(r"http://([^/]*)").context("Failed to compile regex")?;

    for line in reader.lines() {
        let line = line.context("Failed to read line")?;
        if let Some(captures) = domain_regex.captures(&line) {
            if let Some(domain) = captures.get(1) {
                let mut domain = domain.as_str().to_string();
                domain = domain.replace("www.", "");
                writeln!(amp_file, "{}", domain)?;
            }
        }
    }

    // Create Top 100 report
    let payload_file = File::open(&paths.payload_report)
        .context("Failed to open payload report")?;
    let reader = BufReader::new(payload_file);

    let mut top_100_file = File::create(&paths.top_100)
        .context("Failed to create Top 100 report")?;

    for (i, line) in reader.lines().enumerate() {
        if i >= TOP_DOMAINS_COUNT {
            break;
        }
        let line = line.context("Failed to read line")?;
        writeln!(top_100_file, "{}", line)?;
    }

    Ok(())
}


pub fn handle_payload_report(
    paths: &Paths,
    data: String,
    option: PayloadOption,
) -> Result<usize> {
    let line_count = data.lines().count();

    // Obfuscate in memory if needed
    let processed_data = match option {
        PayloadOption::Obfuscate | PayloadOption::Both => data.replace("http", "hxxp"),
        _ => data,
    };

    // Write to disk as chosen
    match option {
        PayloadOption::LeaveAsIs | PayloadOption::Obfuscate => {
            std::fs::write(&paths.payload_report, &processed_data)
                .context("Failed to write payload report")?;
            Printer::success("PayloadReport.txt saved.");
        }
        PayloadOption::Zip | PayloadOption::Both => {
            let zip_path = paths.payload_report.with_extension("zip");
            let file = File::create(&zip_path).context("Failed to create zip file")?;
            let mut zip = ZipWriter::new(file);

            let options = FileOptions::default()
                .compression_method(CompressionMethod::Deflated)
                .unix_permissions(0o755);

            zip.start_file("PayloadReport.txt", options)
                .context("Failed to add file to zip")?;
            zip.write_all(processed_data.as_bytes())
                .context("Failed to write content to zip")?;

            zip.finish().context("Failed to finish zip file")?;
            Printer::success(&format!(
                "PayloadReport.txt zipped as {}.",
                zip_path.display()
            ));
        }
    }

    Ok(line_count)
}

#[allow(dead_code)]
pub fn calculate_checksum(path: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    use std::fs::File;
    use std::io::Read;

    let mut file = File::open(path).context("Failed to open file for checksum")?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 4096];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hex::encode(hasher.finalize()))
}

#[allow(dead_code)]
pub fn verify_checksum(path: &Path, expected: Option<&str>) -> Result<bool> {
    if !path.exists() {
        return Ok(false);
    }

    if let Some(expected) = expected {
        let calculated = calculate_checksum(path)?;
        Ok(calculated.to_lowercase() == expected.trim().to_lowercase())
    } else {
        Ok(true)
    }
}

