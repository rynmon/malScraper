# malScraper v1.5.0

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Cross-platform tool to scrape malware domains, IOCs, and C2 IPs from various feeds for easy blacklisting.**

> ⚠️ **Note**: The Python and Bash versions have been deprecated. This project now uses Rust exclusively for better performance, security, and distribution.

## 🚀 Quick Start

### Download Pre-built Binaries

**Windows:**
- Download `malscraper-x86_64-pc-windows-msvc.exe` from [Releases](https://github.com/rynmon/malScraper/releases)
- Rename to `malscraper.exe` and run

**macOS:**
- Download `malscraper-x86_64-apple-darwin` or `malscraper-aarch64-apple-darwin` from [Releases](https://github.com/rynmon/malScraper/releases)
- Make executable: `chmod +x malscraper-*`
- Run: `./malscraper-*`

**Linux:**
- Download `malscraper-x86_64-unknown-linux-gnu` or `malscraper-aarch64-unknown-linux-gnu` from [Releases](https://github.com/rynmon/malScraper/releases)
- Make executable: `chmod +x malscraper-*`
- Run: `./malscraper-*`

### Build from Source

**Prerequisites:**
- [Rust 1.70+](https://rustup.rs/)
- Visual Studio Build Tools (Windows) or GCC/Clang (Linux/macOS)

**Build:**
```bash
cd rust
cargo build --release
```

The binary will be at `rust/target/release/malscraper` (or `.exe` on Windows).

## ✨ Features

- **High Performance**: 3-5x faster than Python version
- **Single Binary**: No runtime dependencies required
- **Cross-Platform**: Windows, macOS, and Linux
- **Tab Completion**: Built-in command completion (press TAB)
- **Async Downloads**: Fast parallel downloads with progress bars
- **Memory Safe**: Rust's ownership system prevents common bugs
- **Auto-Updates**: Built-in update checking via GitHub Releases

## 📋 Usage

1. Run the tool:
   ```bash
   malscraper
   ```

2. Available commands:
   - `FULL` or `FULL-SCAN` - Complete scan of all feeds
   - `QUICK` or `QUICK-SCAN` - Quick scan (most recent 100 domains)
   - `HELP` - Show help menu
   - `TUTORIAL` - Show tutorial
   - `OPEN` or `REOPEN` - Open a previously downloaded report
   - `UPDATE` - Check for and install updates
   - `QUIT` or `EXIT` - Exit the application

3. Press `TAB` for command auto-completion!

## 📁 File Locations

Reports are saved to:
- **Windows**: `%USERPROFILE%\Documents\malScraper\`
- **macOS/Linux**: `~/Desktop/malScraper/`

## ⚠️ Antivirus Warning

Some reports (especially `PayloadReport.txt`) may be flagged by antivirus software because they contain known malware indicators. These files are for research and defensive use only.

- You'll be prompted to obfuscate or zip the payload report
- Consider adding an exclusion for the report directory

## 🔄 Migration from Python/Bash Versions

If you were using the Python or Bash versions:

1. **Download the Rust binary** from [Releases](https://github.com/rynmon/malScraper/releases)
2. **Your existing reports** will still be in the same location
3. **Commands are identical** - no learning curve!
4. **Better performance** - downloads and processing are faster

## 🛠️ Development

```bash
# Clone the repository
git clone https://github.com/rynmon/malScraper.git
cd malScraper/rust

# Build
cargo build --release

# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy
```

## 📊 Performance Comparison

| Metric | Python | Rust |
|--------|--------|------|
| Startup Time | ~1-2s | <100ms |
| Download Speed | Baseline | 2-3x faster |
| Memory Usage | ~50-100MB | ~10-20MB |
| Binary Size | N/A | ~5-10MB |
| Dependencies | Python + packages | Single binary |

## 🤝 Contributing

Contributions are welcome! Please feel free to:
- Open issues for bugs or feature requests
- Submit pull requests
- Improve documentation

## 📝 License

This project is licensed under the [MIT License](LICENSE).

## 👤 Author

**Ryan Monaghan** ([@rynmonaghan](https://bsky.app/profile/rynmon.ie))

- Website: https://rynmon.ie
- GitHub: [@rynmon](https://github.com/rynmon)
- Bluesky: [@rynmon.ie](https://bsky.app/profile/rynmon.ie)

## 🙏 Acknowledgments

- All the security researchers and organizations providing the threat intelligence feeds
- The Rust community for excellent tooling and libraries

---

**Note**: Python and Bash versions are deprecated. Please use the Rust version for the best experience and continued support.
