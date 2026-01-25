use anyhow::Result;
use semver::Version;
use std::path::Path;

pub fn get_base_dir() -> Result<std::path::PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;

    #[cfg(target_os = "windows")]
    {
        Ok(home.join("Documents").join("malScraper"))
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(home.join("Desktop").join("malScraper"))
    }
}

pub fn is_newer_version(current: &str, latest: &str) -> bool {
    let current_clean = current.trim_start_matches('v').trim();
    let latest_clean = latest.trim_start_matches('v').trim();

    match (Version::parse(current_clean), Version::parse(latest_clean)) {
        (Ok(current_ver), Ok(latest_ver)) => latest_ver > current_ver,
        _ => {
            eprintln!("Warning: Could not compare versions '{}' and '{}'", current, latest);
            false
        }
    }
}

pub fn clear_screen() {
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd")
            .args(["/C", "cls"])
            .status();
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = std::process::Command::new("clear").status();
    }
}

pub fn get_terminal_width() -> usize {
    crossterm::terminal::size()
        .map(|(w, _)| w as usize)
        .unwrap_or(80)
}

pub fn open_file(path: &Path) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &path.to_string_lossy()])
            .spawn()?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(path).spawn()?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(path).spawn()?;
    }

    Ok(())
}

pub fn get_random_splash() -> &'static str {
    use rand::seq::SliceRandom;
    use rand::thread_rng;
    let splashes = [
        "ðŸ”Ž Generating list...",
        "ðŸ”Ž Scraping data...",
        "ðŸ”Ž Spinning web...",
        "ðŸ”Ž Hunting threats...",
        "ðŸ”Ž Collecting indicators...",
    ];
    splashes.choose(&mut thread_rng()).unwrap_or(&splashes[0])
}

pub fn get_random_exit_message() -> &'static str {
    use rand::seq::SliceRandom;
    use rand::thread_rng;
    let messages = [
        "Bye... ðŸ‘‹ðŸ˜¢",
        "Cya... ðŸ‘‹ðŸ˜¢",
        "Byeeeeeeeeeeee... ðŸ‘‹ðŸ˜¢",
        "Until next time... ðŸ‘‹",
        "Happy hunting! ðŸ‘‹",
    ];
    messages.choose(&mut thread_rng()).unwrap_or(&messages[0])
}

pub fn set_console_title(title: &str) {
    #[cfg(target_os = "windows")]
    {
        use winapi::um::wincon::SetConsoleTitleA;
        use std::ffi::CString;

        unsafe {
            if let Ok(cstr) = CString::new(title) {
                SetConsoleTitleA(cstr.as_ptr());
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        // For Unix-like systems, use escape sequences
        print!("\x1b]0;{}\x07", title);
        use std::io::Write;
        let _ = std::io::stdout().flush();
    }
}

pub fn scroll_console_to_top() {
    #[cfg(target_os = "windows")]
    {
        use winapi::um::processenv::GetStdHandle;
        use winapi::um::winbase::STD_OUTPUT_HANDLE;
        use winapi::um::wincon::{SetConsoleWindowInfo, GetConsoleScreenBufferInfo, COORD, SMALL_RECT};
        use winapi::um::handleapi::INVALID_HANDLE_VALUE;
        
        unsafe {
            let console_handle = GetStdHandle(STD_OUTPUT_HANDLE);
            if console_handle != INVALID_HANDLE_VALUE {
                // Get current buffer info
                let mut buffer_info: winapi::um::wincon::CONSOLE_SCREEN_BUFFER_INFO = std::mem::zeroed();
                if GetConsoleScreenBufferInfo(console_handle, &mut buffer_info) != 0 {
                    // Set window to show from the top (Y = 0)
                    let window_rect = SMALL_RECT {
                        Left: 0,
                        Top: 0,
                        Right: buffer_info.dwSize.X - 1,
                        Bottom: (buffer_info.srWindow.Bottom - buffer_info.srWindow.Top) as i16,
                    };
                    SetConsoleWindowInfo(console_handle, 1, &window_rect);
                }
            }
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        // No-op on non-Windows systems
    }
}
