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
    // Allocate console for Windows subsystem applications
    // This allows the app to show its custom icon in the taskbar
    // while still functioning as a console application
    #[cfg(target_os = "windows")]
    {
        use winapi::um::consoleapi::AllocConsole;
        use winapi::um::fileapi::{CreateFileA, OPEN_EXISTING};
        use winapi::um::handleapi::INVALID_HANDLE_VALUE;
        use winapi::um::winnt::{FILE_SHARE_WRITE, GENERIC_READ, GENERIC_WRITE};
        use winapi::um::processenv::SetStdHandle;
        use winapi::um::winbase::{STD_OUTPUT_HANDLE, STD_INPUT_HANDLE, STD_ERROR_HANDLE};
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
            }
        }
    }
    
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

