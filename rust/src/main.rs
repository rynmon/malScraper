mod app;
mod cli;
mod compare;
mod completer;
mod config;
mod custom_feeds;
mod dedupe;
mod download;
mod export;
mod file_ops;
mod history;
mod printer;
mod search;
mod stats;
mod update;
mod utils;
mod validate;
mod whitelist;

use app::MalScraper;
use clap::Parser;
use cli::Cli;
use std::process;

#[tokio::main]
async fn main() {
    // Allocate console for Windows subsystem applications
    // This allows the app to show its custom icon in the taskbar
    // while still functioning as a console application
    #[cfg(target_os = "windows")]
    {
        use winapi::um::consoleapi::{AllocConsole, GetConsoleMode, SetConsoleMode};
        use winapi::um::fileapi::{CreateFileA, OPEN_EXISTING};
        use winapi::um::handleapi::INVALID_HANDLE_VALUE;
        use winapi::um::winnt::{FILE_SHARE_WRITE, GENERIC_READ, GENERIC_WRITE};
        use winapi::um::processenv::{SetStdHandle, GetStdHandle};
        use winapi::um::winbase::{STD_OUTPUT_HANDLE, STD_INPUT_HANDLE, STD_ERROR_HANDLE};
        use winapi::um::wincon::{SetConsoleScreenBufferSize, COORD, SMALL_RECT, SetConsoleWindowInfo, ENABLE_VIRTUAL_TERMINAL_PROCESSING};
        use std::ffi::CString;
        
        unsafe {
            // Allocate a new console window
            if AllocConsole() != 0 {
                // Reopen stdin
                let stdin_handle = CreateFileA(
                    CString::new("CONIN$").unwrap().as_ptr(),
                    GENERIC_READ | GENERIC_WRITE,
                    FILE_SHARE_WRITE,
                    std::ptr::null_mut(),
                    OPEN_EXISTING,
                    0,
                    std::ptr::null_mut(),
                );
                if stdin_handle != INVALID_HANDLE_VALUE {
                    SetStdHandle(STD_INPUT_HANDLE, stdin_handle);
                }
                
                // Reopen stdout
                let stdout_handle = CreateFileA(
                    CString::new("CONOUT$").unwrap().as_ptr(),
                    GENERIC_READ | GENERIC_WRITE,
                    FILE_SHARE_WRITE,
                    std::ptr::null_mut(),
                    OPEN_EXISTING,
                    0,
                    std::ptr::null_mut(),
                );
                if stdout_handle != INVALID_HANDLE_VALUE {
                    SetStdHandle(STD_OUTPUT_HANDLE, stdout_handle);
                }
                
                // Reopen stderr
                let stderr_handle = CreateFileA(
                    CString::new("CONOUT$").unwrap().as_ptr(),
                    GENERIC_READ | GENERIC_WRITE,
                    FILE_SHARE_WRITE,
                    std::ptr::null_mut(),
                    OPEN_EXISTING,
                    0,
                    std::ptr::null_mut(),
                );
                if stderr_handle != INVALID_HANDLE_VALUE {
                    SetStdHandle(STD_ERROR_HANDLE, stderr_handle);
                }
                
                // Resize console window to ensure all menu items are visible
                let console_handle = GetStdHandle(STD_OUTPUT_HANDLE);
                if console_handle != INVALID_HANDLE_VALUE {
                    // Enable ANSI color support on Windows 10+
                    let mut mode: u32 = 0;
                    if GetConsoleMode(console_handle, &mut mode) != 0 {
                        mode |= ENABLE_VIRTUAL_TERMINAL_PROCESSING;
                        SetConsoleMode(console_handle, mode);
                    }
                    
                    // Set buffer size (width x height in characters)
                    // Use a larger buffer to accommodate all menu items
                    let buffer_size = COORD {
                        X: 120,  // Width: 120 characters
                        Y: 3000, // Height: 3000 lines (allows scrolling)
                    };
                    SetConsoleScreenBufferSize(console_handle, buffer_size);
                    
                    // Set window size (visible area)
                    let window_rect = SMALL_RECT {
                        Left: 0,
                        Top: 0,
                        Right: 119,  // 120 - 1 (0-indexed)
                        Bottom: 39,  // 40 lines visible (can be adjusted)
                    };
                    SetConsoleWindowInfo(console_handle, 1, &window_rect);
                }
            }
        }
    }
    
    // Handle Ctrl+C gracefully
    ctrlc::set_handler(|| {
        println!("\nBye... 👋😢");
        process::exit(0);
    })
    .expect("Error setting Ctrl+C handler");

    let cli = Cli::parse();

    // Initialize app with custom output directory if provided, otherwise use default
    let mut app = if let Some(output_dir) = cli.output_dir {
        match MalScraper::new_with_dir(output_dir).await {
            Ok(app) => app,
            Err(e) => {
                eprintln!("Error initializing malScraper: {}", e);
                process::exit(1);
            }
        }
    } else {
        match MalScraper::new().await {
            Ok(app) => app,
            Err(e) => {
                eprintln!("Error initializing malScraper: {}", e);
                process::exit(1);
            }
        }
    };

    // Handle CLI commands (non-interactive mode)
    if let Some(command) = cli.command {
        if let Err(e) = app.handle_cli_command(command).await {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
        return;
    }

    // Interactive mode
    if let Err(e) = app.run().await {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

