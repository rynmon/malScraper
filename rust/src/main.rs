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
    // Enable ANSI/VT100 color support and set console window size on Windows
    // This works with both Windows Terminal and traditional console
    #[cfg(target_os = "windows")]
    {
        use winapi::um::consoleapi::{GetConsoleMode, SetConsoleMode};
        use winapi::um::processenv::GetStdHandle;
        use winapi::um::winbase::STD_OUTPUT_HANDLE;
        use winapi::um::wincon::{ENABLE_VIRTUAL_TERMINAL_PROCESSING, SetConsoleScreenBufferSize, SetConsoleWindowInfo, SetConsoleCursorPosition, GetConsoleWindow, COORD, SMALL_RECT};
        use winapi::um::handleapi::INVALID_HANDLE_VALUE;
        use winapi::um::winuser::{GetSystemMetrics, SetWindowPos, SWP_NOMOVE, SWP_NOZORDER, SM_CXFRAME, SM_CYFRAME, SM_CYCAPTION};
        use winapi::shared::windef::HWND;
        
        unsafe {
            let console_handle = GetStdHandle(STD_OUTPUT_HANDLE);
            if console_handle != INVALID_HANDLE_VALUE {
                // Enable ANSI/VT100 color support
                let mut mode: u32 = 0;
                if GetConsoleMode(console_handle, &mut mode) != 0 {
                    mode |= ENABLE_VIRTUAL_TERMINAL_PROCESSING;
                    SetConsoleMode(console_handle, mode);
                }
                
                // Set buffer size (width x height in characters)
                // Use a larger buffer to accommodate all menu items and allow scrolling
                let buffer_size = COORD {
                    X: 120,  // Width: 120 characters
                    Y: 3000, // Height: 3000 lines (allows scrolling)
                };
                SetConsoleScreenBufferSize(console_handle, buffer_size);
                
                // Set window size (visible area) - sized to show banner + labels + help menu + prompt
                // Banner: 5 lines, Labels: 8 lines, Help menu: ~30 lines (header + 19 items + spacing + tip + prompt)
                // Total: ~43-45 lines, using 50 to ensure everything fits with some buffer
                let window_rect = SMALL_RECT {
                    Left: 0,
                    Top: 0,
                    Right: 119,  // 120 - 1 (0-indexed)
                    Bottom: 49,   // 50 lines visible (0-indexed, so 49)
                };
                SetConsoleWindowInfo(console_handle, 1, &window_rect);
                
                // Try to resize the actual window using the window handle
                // This works better with Windows Terminal
                let console_window: HWND = GetConsoleWindow();
                if !console_window.is_null() {
                    // Try to set the window icon from the embedded icon resource
                    // Note: Windows Terminal may not respect this, but it works for traditional console
                    use winapi::um::winuser::{LoadIconW, SendMessageW, WM_SETICON, ICON_SMALL, ICON_BIG, MAKEINTRESOURCEW};
                    use winapi::um::libloaderapi::GetModuleHandleW;
                    use winapi::shared::windef::HICON;
                    
                    // Get the module handle for the current executable
                    let hinstance = GetModuleHandleW(std::ptr::null());
                    if !hinstance.is_null() {
                        // Load icon from resource (ID 1, which is what we embedded in build.rs)
                        // For small icon (16x16) - used in title bar
                        let small_icon: HICON = LoadIconW(hinstance, MAKEINTRESOURCEW(1)) as HICON;
                        if !small_icon.is_null() {
                            SendMessageW(console_window, WM_SETICON, ICON_SMALL as usize, small_icon as isize);
                        }
                        
                        // For big icon (32x32) - used in Alt+Tab
                        let big_icon: HICON = LoadIconW(hinstance, MAKEINTRESOURCEW(1)) as HICON;
                        if !big_icon.is_null() {
                            SendMessageW(console_window, WM_SETICON, ICON_BIG as usize, big_icon as isize);
                        }
                    }
                    
                    // Get character cell size (approximate: 8x16 pixels for most fonts)
                    // For more accuracy, we'd need GetConsoleFontSize, but this is a reasonable default
                    let char_width = 8;
                    let char_height = 16;
                    
                    // Calculate window size in pixels
                    let window_width = 120 * char_width;
                    let window_height = 50 * char_height;
                    
                    // Add window frame/border size
                    let frame_x = GetSystemMetrics(SM_CXFRAME) * 2;
                    let frame_y = GetSystemMetrics(SM_CYFRAME) * 2;
                    let caption = GetSystemMetrics(SM_CYCAPTION);
                    
                    let total_width = window_width + frame_x;
                    let total_height = window_height + frame_y + caption;
                    
                    // Resize the window
                    SetWindowPos(
                        console_window,
                        winapi::um::winuser::HWND_TOP,
                        0,
                        0,
                        total_width as i32,
                        total_height as i32,
                        SWP_NOMOVE | SWP_NOZORDER,
                    );
                }
                
                // Scroll to the top to ensure the banner is visible
                // Set cursor to top-left (0, 0) to ensure we're viewing from the beginning
                let top_pos = COORD { X: 0, Y: 0 };
                SetConsoleCursorPosition(console_handle, top_pos);
                
                // Small delay to allow Windows Terminal to process the resize
                // This helps ensure the window size is applied before content is displayed
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
        
        // Also try using the 'mode' command as a fallback for Windows Terminal
        // This sometimes works better with Windows Terminal than the API calls
        let _ = std::process::Command::new("cmd")
            .args(["/C", "mode", "con:", "cols=120", "lines=50"])
            .output();
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

