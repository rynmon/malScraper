# malScraper v2.0.7

[![GitHub stars](https://img.shields.io/github/stars/rynmon/malScraper?style=flat-square&logo=github)](https://github.com/rynmon/malScraper/stargazers)
[![GitHub forks](https://img.shields.io/github/forks/rynmon/malScraper?style=flat-square&logo=github)](https://github.com/rynmon/malScraper/network/members)
[![GitHub issues](https://img.shields.io/github/issues/rynmon/malScraper?style=flat-square&logo=github)](https://github.com/rynmon/malScraper/issues)
[![GitHub release](https://img.shields.io/github/v/release/rynmon/malScraper?style=flat-square&logo=github)](https://github.com/rynmon/malScraper/releases)
[![GitHub license](https://img.shields.io/github/license/rynmon/malScraper?style=flat-square&logo=github)](https://github.com/rynmon/malScraper/blob/main/LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Cross-platform tool to scrape malware domains, IOCs, and C2 IPs from various feeds for easy blacklisting.**

> **Note**: The Python and Bash versions have been deprecated. This project now uses Rust exclusively for better performance, security, and distribution.

## Quick Start

### Download Pre-built Binaries

**Windows:**
- Download `malscraper-x86_64-pc-windows-msvc.exe` from [Releases](https://github.com/rynmon/malScraper/releases)
- Rename to `malscraper.exe` and run

**macOS:**
- Download `malscraper-x86_64-apple-darwin` or `malscraper-aarch64-apple-darwin` from [Releases](https://github.com/rynmon/malScraper/releases)
- Make executable: `chmod +x malscraper-*`
- Run: `./malscraper-*`

**macOS (as a real app):**
- Download `malscraper-aarch64-apple-darwin.app.tar.gz` (Apple Silicon) or `malscraper-x86_64-apple-darwin.app.tar.gz` (Intel) from [Releases](https://github.com/rynmon/malScraper/releases)
- Extract: `tar xzf malscraper-*.app.tar.gz`
- Drag `malScraper.app` into `/Applications`
- Double-click to launch (opens a Terminal window running malScraper). First launch may show a Gatekeeper warning since the app isn't notarized — right-click → Open to bypass.

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

**macOS — build for Apple Silicon (ARM), Intel, or universal:**

A helper script is provided for explicit target selection:

```bash
cd rust
./build.sh                 # native build (ARM on Apple Silicon, Intel on x86_64 Macs)
./build.sh --arm           # force aarch64-apple-darwin (Apple Silicon)
./build.sh --x86_64        # force x86_64-apple-darwin (Intel)
./build.sh --universal     # universal binary (ARM + Intel via lipo)
```

The script auto-installs the requested rustup target if missing. Output binaries land in
`rust/target/<triple>/release/malscraper`, and the universal binary at
`rust/target/universal-apple-darwin/release/malscraper`.

You can also invoke cargo directly:

```bash
rustup target add aarch64-apple-darwin
cargo build --release --target aarch64-apple-darwin
```

**macOS — build a `.app` bundle:**

To produce a double-clickable `malScraper.app` (Finder/Launchpad/Spotlight launchable, opens a Terminal window running the TUI):

```bash
cd rust
./build.sh --app             # native architecture
./build.sh --app-arm         # aarch64-apple-darwin
./build.sh --app-x86_64      # x86_64-apple-darwin
./build.sh --app-universal   # universal (ARM + Intel)
```

The bundle is written to `rust/dist/malScraper.app`. The build script
ad-hoc codesigns it so it runs on Apple Silicon, but the bundle is **not**
notarized — first launch from Finder will show "Apple cannot verify…", which
the user clears by right-click → Open once.

To install:

```bash
cp -R rust/dist/malScraper.app /Applications/
```

To rebuild only the bundle from an already-built binary:

```bash
./scripts/bundle-macos.sh target/aarch64-apple-darwin/release/malscraper
```

The icon is generated from `rust/assets/icon.png` (the rynmon.ie brand
asterisk on a dark rounded-rect background, 512×512 RGBA). The vector
source lives at `rust/assets/icon.svg`. To upgrade Retina sharpness, drop
a 1024×1024 RGBA PNG at `rust/assets/icon.png` and rebuild — the bundler
prefers PNG over the legacy `icon.ico`.

## Features

### Core Features
- **High Performance**: 3-5x faster than Python version
- **Single Binary**: No runtime dependencies required
- **Cross-Platform**: Windows, macOS, and Linux
- **Tab Completion**: Built-in command completion (press TAB)
- **Async Downloads**: Fast parallel downloads with progress bars
- **Memory Safe**: Rust's ownership system prevents common bugs
- **Auto-Updates**: Built-in update checking via GitHub Releases

### New in v2.0.0

#### Analysis & Intelligence
- **Statistics Dashboard** (`STATS`) - View comprehensive metrics and analytics for all reports
- **Search & Filter** (`SEARCH`, `FILTER`) - Search across reports with regex support
- **Report Comparison** (`COMPARE`) - Compare two reports side-by-side to see differences
- **Historical Tracking** (`DIFF`, `CHANGES`) - Track changes over time and identify new indicators

#### Data Management
- **Deduplication** (`DEDUPE`, `UNIQUE`) - Remove duplicates across all reports and create unified master lists
- **Validation** (`VALIDATE`) - Validate IP addresses and domains, check if domains are still active
- **Whitelist Management** (`WHITELIST`) - Whitelist false positives and exclude known-good indicators

#### Export & Integration
- **Export Formats** (`EXPORT`) - Export to multiple formats:
  - Firewall rules: iptables, Windows Firewall, pfSense
  - SIEM formats: JSON, CSV with metadata
  - Threat intelligence: STIX/TAXII

#### Customization & Automation
- **Custom Feeds** (`FEEDS`) - Add, list, and remove your own custom feed URLs
- **Non-Interactive Mode** - CLI arguments for automation and scripting
  ```bash
  malscraper quick-scan --output-dir ./reports
  malscraper export iptables payload
  malscraper search malware.com
  ```
- **Automatic Updates** (`UPDATE`, `INSTALL`) - Automatically download and install updates with platform detection

## Usage

1. Run the tool:
   ```bash
   malscraper
   ```

2. Available commands:

   **Basic Operations:**
   - `FULL` or `FULL-SCAN` - Complete scan of all feeds
   - `QUICK` or `QUICK-SCAN` - Quick scan (most recent 100 domains)
   - `OPEN` or `REOPEN` - Open a previously downloaded report
   - `UPDATE` - Check for and install updates
   - `HELP` - Show help menu
   - `TUTORIAL` - Show tutorial
   - `QUIT` or `EXIT` - Exit the application

   **Analysis Features:**
   - `STATS` - View statistics dashboard
   - `SEARCH <term>` - Search for specific terms across reports
   - `FILTER [feed_type] [pattern]` - Filter reports by criteria
   - `COMPARE <report1> <report2>` - Compare two reports
   - `DIFF` or `CHANGES` - Compare current scan with previous scan

   **Data Management:**
   - `DEDUPE` or `UNIQUE` - Deduplicate all reports into master list
   - `VALIDATE <report>` - Validate IP addresses and domains
   - `WHITELIST ADD <indicator> [reason]` - Add to whitelist
   - `WHITELIST LIST` - List all whitelisted indicators
   - `WHITELIST REMOVE <indicator>` - Remove from whitelist

   **Export & Integration:**
   - `EXPORT <format> <report>` - Export to firewall/SIEM formats
     - Formats: `iptables`, `windows`, `pfsense`, `json`, `csv`, `stix`, `taxii`
     - Reports: `payload`, `amp`, `c2`, `hex`, `haus`, `phish`, `top100`

   **Customization:**
   - `FEEDS ADD <url> [name] [description]` - Add custom feed
   - `FEEDS LIST` - List all custom feeds
   - `FEEDS REMOVE <name_or_url>` - Remove custom feed

3. Press `TAB` for command auto-completion!

### Non-Interactive Mode (CLI)

For automation and scripting, use CLI arguments:

```bash
# Quick scan with custom output directory
malscraper quick-scan --output-dir ./reports

# Full scan
malscraper full-scan --output-dir ./reports

# Export to iptables format
malscraper export iptables payload

# Search across reports
malscraper search malware.com

# View statistics
malscraper stats

# See all available commands
malscraper --help
```

## File Locations

Reports are saved to:
- **Windows**: `%USERPROFILE%\Documents\malScraper\`
- **macOS/Linux**: `~/Desktop/malScraper/`

## Antivirus Warning

Some reports (especially `PayloadReport.txt`) may be flagged by antivirus software because they contain known malware indicators. These files are for research and defensive use only.

- You'll be prompted to obfuscate or zip the payload report
- Consider adding an exclusion for the report directory

## Migration from Python/Bash Versions

If you were using the Python or Bash versions:

1. **Download the Rust binary** from [Releases](https://github.com/rynmon/malScraper/releases)
2. **Your existing reports** will still be in the same location
3. **Commands are identical** - no learning curve!
4. **Better performance** - downloads and processing are faster

## Development

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

## Performance Comparison

| Metric | Python | Rust |
|--------|--------|------|
| Startup Time | ~1-2s | <100ms |
| Download Speed | Baseline | 2-3x faster |
| Memory Usage | ~50-100MB | ~10-20MB |
| Binary Size | N/A | ~5-10MB |
| Dependencies | Python + packages | Single binary |

## Contributing

Contributions are welcome! Please feel free to:
- Open issues for bugs or feature requests
- Submit pull requests
- Improve documentation

## License

This project is licensed under the [MIT License](LICENSE).
---

**Note**: Python and Bash versions are deprecated. Please use the Rust version for the best experience and continued support.
