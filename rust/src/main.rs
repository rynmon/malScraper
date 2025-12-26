mod app;
mod completer;
mod config;
mod download;
mod file_ops;
mod printer;
mod update;
mod utils;

use app::MalScraper;
use std::process;

#[tokio::main]
async fn main() {
    // Handle Ctrl+C gracefully
    ctrlc::set_handler(|| {
        println!("\nBye... 👋😢");
        process::exit(0);
    })
    .expect("Error setting Ctrl+C handler");

    let mut app = match MalScraper::new().await {
        Ok(app) => app,
        Err(e) => {
            eprintln!("Error initializing malScraper: {}", e);
            process::exit(1);
        }
    };

    if let Err(e) = app.run().await {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

