use crate::config::REQUEST_TIMEOUT_SECS;
use crate::printer::Printer;
use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use std::time::Duration;
use tokio::io::AsyncWriteExt;

pub struct Downloader {
    client: reqwest::Client,
}

impl Downloader {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    pub async fn download_file(
        &self,
        url: &str,
        output_path: &Path,
        description: Option<&str>,
    ) -> Result<()> {
        if let Some(desc) = description {
            Printer::info(&format!("Downloading {}...", desc));
        }

        let response = self
            .client
            .get(url)
            .send()
            .await
            .context("Failed to send request")?;

        let total_size = response.content_length().unwrap_or(0);

        let response = response
            .error_for_status()
            .context("HTTP request failed")?;

        let mut file = tokio::fs::File::create(output_path)
            .await
            .context("Failed to create output file")?;

        let mut stream = response.bytes_stream();
        let mut downloaded: u64 = 0;

        if total_size > 0 {
            // Show progress bar
            let pb = ProgressBar::new(total_size);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{msg} [{wide_bar}] {percent}%")
                    .unwrap()
                    .progress_chars("█░"),
            );
            pb.set_message("Progress:");

            use futures::StreamExt;
            while let Some(item) = stream.next().await {
                let chunk = item.context("Failed to read chunk")?;
                file.write_all(&chunk).await.context("Failed to write chunk")?;
                downloaded += chunk.len() as u64;
                pb.set_position(downloaded);
            }

            pb.finish_with_message("Complete");
        } else {
            // Show spinner for unknown size
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner} {msg}")
                    .unwrap(),
            );
            pb.set_message("Downloading...");

            use futures::StreamExt;
            while let Some(item) = stream.next().await {
                let chunk = item.context("Failed to read chunk")?;
                file.write_all(&chunk).await.context("Failed to write chunk")?;
                pb.inc(1);
            }

            pb.finish_with_message("Complete");
        }

        Ok(())
    }

    pub async fn download_text(&self, url: &str) -> Result<String> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .context("Failed to send request")?;

        let response = response
            .error_for_status()
            .context("HTTP request failed")?;

        let text = response.text().await.context("Failed to read response")?;
        Ok(text)
    }

    pub fn cleanup_failed_download(&self, output_path: &Path) {
        if output_path.exists() {
            if let Err(e) = std::fs::remove_file(output_path) {
                Printer::warning(&format!(
                    "Warning: Could not remove partial file {}: {}",
                    output_path.display(),
                    e
                ));
            }
        }
    }
}

impl Default for Downloader {
    fn default() -> Self {
        Self::new()
    }
}

