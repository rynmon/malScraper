[malScraper v1.3](https://ryan-monaghan.github.io/malScraper/)
===============
![malScraper Screenshot](https://raw.githubusercontent.com/Ryan-Monaghan/ryanmonaghan.github.io/master/Screenshot%20from%202020-01-23%2014-21-06.png)
![2023-12-27_00-37-09](https://github.com/Ryan-Monaghan/malScraper/assets/8824673/5a2fa461-6b67-4070-8df0-cd8c41901fe6)
## Table of Contents
- [Introduction](#introduction)
- [Features](#features)
- [Version History](#version-history)
- [Installation](#installation)
- [Usage](#usage)
- [Contributing](#contributing)
- [Version Checking](#version-checking)
- [License](#license)

## Introduction
- malScraper is a modular tool that streamlines the process of scraping and managing lists of Payload Domains, IOC's & C2 IPs from various feeds. It simplifies the task of blacklisting for security and threat intelligence purposes.

## Features
- **Modular Design**: Easily extendable with additional modules.
- **Version Checking**: Verify if you are using the latest version on startup.
- **Reopen Functionality**: Ability to reopen a previously composed report.

## Current Release Notes
# malScraper 1.4 - Python Expansion

## Overview
Version 1.4 represents a major milestone for malScraper with a complete rewrite in Python, making it fully cross-platform compatible with Windows, macOS, and Linux environments. This update maintains all the key functionality of the original bash script while adding new features, improving performance, and enhancing the user experience.

## Key Features

### Cross-Platform Compatibility
- Now works seamlessly on Windows, macOS, and Linux
- Platform-specific optimizations for file paths and system operations
- No external dependencies beyond Python and the requests library

### Enhanced User Interface
- Improved ASCII art header with pyfiglet support
- Consistent color-coded output across all platforms
- Better formatted menus and command outputs
- Progress bars for downloads

### Improved Update System
- Detailed version checking with GitHub API integration
- Visual download progress indicators
- Automatic backup of previous version before updating
- Clean restart process after updates
- Display of release notes within the application

### Robust Error Handling
- Graceful exit handling (Ctrl+C, Ctrl+D)
- Comprehensive exception management
- Network connectivity error handling
- File operation error recovery

### Performance Enhancements
- Faster data processing
- More efficient memory usage
- Improved file I/O operations

## Installation

### Requirements
- Python 3.6 or higher
- Requests library (`pip install requests`)
- Optional: pyfiglet for enhanced ASCII art (`pip install pyfiglet`)

### Windows
python -m pip install requests
python -m pip install pyfiglet  # Optional
python malScraper.py

### macOS/Linux
pip3 install requests
pip3 install pyfiglet  # Optional
python3 malScraper.py

## Upgrading from Previous Versions
Users of previous bash-based versions can simply download the new Python script and run it. The application will maintain the same folder structure and file naming conventions.

## Known Issues
- Some terminal emulators may not fully support color codes
- Character encoding issues may occur in certain environments with emoji display

## Version History
- **1.3** - Initial Python Conversion
  - The tool has been converted to Python for improved functionality.

- **1.2** - Version Checking
  - malScraper now verifies if you are using the latest version on startup.

- **1.1** - Reopen Functionality
  - Added the ability to reopen a previously composed report.

- **1.0** - Initial Implementation
  - Core functionality was implemented in the first release.

## Installation
1. Clone the repository.
   ```python
   git clone https://github.com/rynmon/malScraper
2. Navigate to the project directory.
   ```python
   cd malScraper
3. Install dependencies.
   ```python
   pip install -r requirements.txt

## Usage
- To use malScraper, follow these steps:

1. Run the tool.
   ```python
   python malScraper.py
2. Follow the on-screen instructions to configure and use the tool.

## Contributing
- Contributions are welcome! If you find a bug or have an enhancement in mind, feel free to open an issue or submit a pull request.

## Version Checking
- malScraper will automatically check for updates on startup. Make sure you are using the latest version to benefit from new features and improvements.

## License
- This project is licensed under the [MIT License](https://github.com/Ryan-Monaghan/malScraper/blob/master/LICENSE).
