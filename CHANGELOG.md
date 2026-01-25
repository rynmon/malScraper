# Changelog

All notable changes to malScraper will be documented in this file.

## [2.0.6] - 2025-01-XX

### Added
- Windows Terminal support - application now uses Windows Terminal when set as default
- Automatic console window sizing (50 lines height, 120 width) for better content visibility
- Console viewport scrolling to ensure banner is visible on startup
- Window icon setting for traditional console windows (title bar icon)

### Changed
- Switched from WINDOWS subsystem to CONSOLE subsystem for better Windows Terminal compatibility
- Improved window initialization to show all content without scrolling
- Better ANSI/VT100 color support for Windows Terminal

### Fixed
- Fixed console window opening with proper size to display all menu items
- Fixed banner and labels being scrolled off-screen on startup

## [1.5.6] - 2025-01-XX

### Fixed
- Fixed update check functionality - added User-Agent header required by GitHub API
- Fixed GitHub API URL (changed from Ryan-Monaghan to rynmon)
- Improved error handling in update checker with better error messages
- Fixed version parsing to handle 'v' prefix in release tags

### Changed
- Update check now works correctly and shows proper error messages
- UPDATE command now fully functional (was showing "coming soon")

## [1.5.5] - 2025-01-XX

### Fixed
- Fixed Windows taskbar icon display (now shows custom icon instead of console host icon)
- Fixed text alignment in help menu (all command aliases now properly aligned)
- Fixed label padding in banner (consistent spacing for all labels)
- Fixed version display to automatically sync with Cargo.toml

### Added
- Automatic version syncing from Cargo.toml at compile time
- Windows application icon embedded in executable
- Improved console allocation for Windows subsystem applications

### Changed
- Version now automatically reads from Cargo.toml (no manual updates needed)
- Better visual alignment throughout the UI

## [1.5.0] - 2025-01-XX

### 🎉 Major Release: Rust Rewrite

**BREAKING CHANGE**: Python and Bash versions are now deprecated. This is a complete rewrite in Rust.

### Added
- Complete Rust rewrite for better performance and security
- Pre-built binaries for Windows, macOS, and Linux
- Tab completion for commands (press TAB)
- Faster downloads (2-3x speed improvement)
- Lower memory usage (~10-20MB vs 50-100MB)
- Single binary distribution (no dependencies)
- Automated GitHub Actions builds for releases
- Better error handling and user feedback

### Changed
- **Performance**: 3-5x faster overall
- **Startup time**: <100ms (was 1-2 seconds)
- **Distribution**: Single executable file
- **Memory safety**: Rust's ownership system

### Deprecated
- Python version (`python/malScraper.py`) - no longer maintained
- Bash version (`bash/malScraper.sh`) - no longer maintained

### Migration
- See [MIGRATION.md](MIGRATION.md) for migration guide
- All commands work identically
- Same file locations
- Existing reports are compatible

## [1.4.8] - Previous Python Version

See Python version history in `python/README.md` for details.

---

## Version History Format

- `Added` - New features
- `Changed` - Changes in existing functionality
- `Deprecated` - Soon-to-be removed features
- `Removed` - Removed features
- `Fixed` - Bug fixes
- `Security` - Security fixes

