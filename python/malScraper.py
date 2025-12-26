#!/usr/bin/env python3
"""
malScraper - Cross-platform tool to scrape malware domains, IOCs, and C2 IPs from various feeds
Converted from bash script to Python for Windows, Mac, and Linux compatibility
Original Author: Ryan Monaghan (@rynmonaghan)
Updated and Optimized: March 2025
"""

import os
import sys
import random
import datetime
import subprocess
import platform
import signal
import json
import re
import zipfile
import shutil
import hashlib
from pathlib import Path
from typing import Optional, Dict, Tuple, List
import time
import importlib.util

# Constants
CHUNK_SIZE = 8192
PROGRESS_BAR_WIDTH = 20
REQUEST_TIMEOUT = 30
UPDATE_CHECK_TIMEOUT = 10
TOP_DOMAINS_COUNT = 100

# Colors for terminal output (works on Windows, Mac, Linux)
class Colors:
    RED = '\033[0;31m'
    GREEN = '\033[0;32m'
    YELLOW = '\033[1;33m'
    PURPLE = '\033[1;35m'
    CYAN = '\033[1;36m'
    BOLD = '\033[1m'
    NORMAL = '\033[0m'

# Printer helper class for consistent printing with colors
class Printer:
    """Helper class for consistent printing with colors"""
    
    BULLET = f"{Colors.BOLD}[*]{Colors.NORMAL}"
    
    @staticmethod
    def info(msg: str) -> None:
        """Print info message"""
        print(f"{Colors.CYAN}{msg}{Colors.NORMAL}")
    
    @staticmethod
    def success(msg: str) -> None:
        """Print success message"""
        print(f"{Colors.GREEN}{msg}{Colors.NORMAL}")
    
    @staticmethod
    def error(msg: str) -> None:
        """Print error message"""
        print(f"{Colors.RED}{msg}{Colors.NORMAL}")
    
    @staticmethod
    def warning(msg: str) -> None:
        """Print warning message"""
        print(f"{Colors.YELLOW}{msg}{Colors.NORMAL}")
    
    @staticmethod
    def menu_item(description: str, commands: str, highlight: Optional[str] = None) -> None:
        """Print a formatted menu item
        
        Args:
            description: Menu item description
            commands: Available commands for this item
            highlight: Optional text to highlight in description
        """
        if highlight and highlight in description:
            # Replace highlight text with colored version
            parts = description.split(highlight, 1)
            if len(parts) == 2:
                desc = f"{parts[0]}{Colors.CYAN}{highlight}{Colors.NORMAL}{parts[1]}"
            else:
                desc = description
        else:
            desc = description
        
        # Calculate padding for better alignment (simplified - works for most terminals)
        desc_len = len(description)
        padding = max(0, 50 - desc_len) if desc_len < 50 else 2
        
        print(f"{Printer.BULLET} {desc}{' ' * padding}{Colors.YELLOW}{commands}{Colors.NORMAL}")
    
    @staticmethod
    def header(text: str, separator: str = " :: ") -> None:
        """Print a header with optional separator
        
        Args:
            text: Header text (may contain separator)
            separator: Separator string to split on
        """
        if separator in text:
            parts = text.split(separator, 1)
            print(f"{Colors.CYAN}{parts[0]}{Colors.NORMAL} {Colors.BOLD}{separator}{Colors.NORMAL} {parts[1] if len(parts) > 1 else ''}")
        else:
            print(f"{Colors.CYAN}{Colors.BOLD}{text}{Colors.NORMAL}")
    
    @staticmethod
    def label_value(label: str, value: str, label_color: str = Colors.PURPLE) -> None:
        """Print a label: value pair (for banner)
        
        Args:
            label: Label text
            value: Value text
            label_color: Color for the label
        """
        print(f"\t{label_color}{label}\t :: {value}{Colors.NORMAL}")
    
    @staticmethod
    def directory_item(number: int, name: str, path: Path) -> None:
        """Print a directory list item
        
        Args:
            number: Item number
            name: Item name
            path: File path
        """
        print(f"{Colors.GREEN}{Colors.BOLD}{number}. {Colors.NORMAL}"
              f"{Colors.RED}{Colors.BOLD}{name}:{Colors.NORMAL}{Colors.NORMAL} {path}")

# Menu data structures
HELP_MENU_ITEMS = [
    {"description": "Tutorial of how to use this tool", "highlight": "Tutorial", "commands": "TUTORIAL"},
    {"description": "Show this Help Menu", "highlight": "Help", "commands": "HELP,GET-HELP,?,-?,/?,MENU"},
    {"description": "Clear screen", "highlight": "Clear", "commands": "CLEAR,CLEAR-HOST,CLS"},
    {"description": "Return to Home Menu", "highlight": "Home", "commands": "HOME,BACK,CD .."},
    {"description": "Open an existing report", "highlight": "Open", "commands": "OPEN,REOPEN"},
    {"description": "Quit malScraper", "highlight": "Quit", "commands": "QUIT,EXIT"},
    {"description": "Install the latest update", "highlight": "Install", "commands": "INSTALL,UPDATE"},
    {"description": "Perform Full-Scan (Note this may take some time)", "highlight": "Full-Scan", "commands": "FULL,FULL-SCAN,FSCAN"},
    {"description": "Perform Quick-Scan (Most recent 100 Payload Domains)", "highlight": "Quick-Scan", "commands": "QUICK,QUICK-SCAN,QSCAN"},
]

DIRECTORY_LIST_ITEMS = [
    {"number": 1, "name": "Payload Domains", "path_key": "payload_report"},
    {"number": 2, "name": "AMP Report", "path_key": "amp_report"},
    {"number": 3, "name": "C2 Servers", "path_key": "c2_report"},
    {"number": 4, "name": "Hex Report", "path_key": "hex_report"},
    {"number": 5, "name": "URLHaus Maldownloads", "path_key": "haus_mal_down"},
    {"number": 6, "name": "PhishTank Phishing Pages", "path_key": "phish_tank"},
    {"number": 7, "name": "Most Recent 100", "path_key": "top_100"},
]

# Full scan download configuration
FULL_SCAN_DOWNLOADS = [
    {"name": "C2 servers report", "feed_key": "c2_feed", "path_key": "c2_report", 
     "header": "#############################################\n# C2 Servers Report sourced from http://cybercrime-tracker.net/ \n#############################################\n"},
    {"name": "Hex report", "feed_key": "hex_feed", "path_key": "hex_report"},
    {"name": "URLHaus Malware downloads", "feed_key": "haus_mal_down", "path_key": "haus_mal_down"},
    {"name": "PhishTank data", "feed_key": "phish_tank", "path_key": "phish_tank"},
]

# --- Dependency check and auto-install (runs once at the very top) ---
required = {"requests", "pyfiglet", "prompt_toolkit", "packaging"}
missing = [pkg for pkg in required if importlib.util.find_spec(pkg) is None]

if missing:
    print(f"{Colors.YELLOW}{Colors.BOLD}Missing required packages: {', '.join(missing)}{Colors.NORMAL}")
    while True:
        choice = input(f"{Colors.PURPLE}Would you like to install them now? (Y/N): {Colors.NORMAL}").strip().lower()
        if choice in ("y", "yes"):
            subprocess.check_call([sys.executable, "-m", "pip", "install", *missing])
            print(f"{Colors.GREEN}Please restart the application.{Colors.NORMAL}")
            sys.exit(0)  # Exit with 0 for successful installation
        elif choice in ("n", "no"):
            print(f"{Colors.RED}Cannot continue without required packages. Exiting.{Colors.NORMAL}")
            sys.exit(1)
        else:
            print(f"{Colors.RED}Invalid input. Please enter Y or N.{Colors.NORMAL}")

# Third-party imports (guaranteed present after above check)
from prompt_toolkit import prompt
from prompt_toolkit.completion import WordCompleter
import requests
import pyfiglet
from packaging import version

# Command completer class for tab completion
class CommandCompleter:
    """Provides tab completion for malScraper commands"""
    
    def __init__(self):
        # Define all available commands and their aliases
        self.commands = {
            # Main scan commands
            'full': ['full', 'full-scan', 'fscan'],
            'quick': ['quick', 'quick-scan', 'qscan'],
            
            # Navigation commands
            'help': ['help', 'get-help', '?', '-?', '/?', 'menu'],
            'tutorial': ['tutorial'],
            'home': ['home', 'back', 'cd ..'],
            'clear': ['clear', 'clear-host', 'cls'],
            
            # File operations
            'open': ['open', 'reopen'],
            
            # System commands
            'quit': ['quit', 'exit'],
            'update': ['install', 'update']
        }
        
        # Flatten the commands list for easier searching
        self.all_commands = []
        for cmd_list in self.commands.values():
            self.all_commands.extend(cmd_list)
        
        # Create prompt_toolkit completer if available
        self.prompt_completer = WordCompleter(self.all_commands, ignore_case=True)
    
    def complete(self, text, state):
        """Complete function for readline (fallback)"""
        if state == 0:
            # This is the first time for this text, so build a match list
            if text:
                self.matches = [s for s in self.all_commands if s.lower().startswith(text.lower())]
            else:
                self.matches = self.all_commands[:]
        
        try:
            return self.matches[state]
        except IndexError:
            return None

# --- Atomic update check (before any other logic) ---
# Note: This runs before MalScraper class is instantiated, so we need to handle it carefully
UPDATE_FLAG = Path(__file__).parent / "update_pending.json"
if UPDATE_FLAG.exists():
    try:
        print(f"{Colors.CYAN}Applying pending update...{Colors.NORMAL}")
        
        with open(UPDATE_FLAG, "r", encoding='utf-8') as f:
            update_info = json.load(f)
        
        new_script_path = Path(update_info["new_script_path"])
        current_script = Path(__file__).resolve()
        backup_path = Path(update_info.get("backup_path", str(current_script.with_suffix('.bak'))))
        
        # Verify new script exists
        if not new_script_path.exists():
            print(f"{Colors.RED}Update file not found. Skipping update.{Colors.NORMAL}")
            UPDATE_FLAG.unlink()
        else:
            # Quick verification: check if file is valid Python and has required class
            try:
                with open(new_script_path, 'r', encoding='utf-8') as f:
                    content = f.read()
                    # Quick syntax check
                    compile(content, str(new_script_path), 'exec')
                    # Check for required class
                    if 'class MalScraper' not in content:
                        raise ValueError("Updated script missing required class")
            except (SyntaxError, ValueError) as e:
                print(f"{Colors.RED}Update verification failed: {e}. Skipping update.{Colors.NORMAL}")
                UPDATE_FLAG.unlink()
            else:
                # Backup current script
                print(f"{Colors.CYAN}Backing up current version...{Colors.NORMAL}")
                shutil.copy2(current_script, backup_path)
                
                # Replace with new script
                print(f"{Colors.CYAN}Installing new version...{Colors.NORMAL}")
                shutil.copy2(new_script_path, current_script)
                
                # Verify the installed script
                try:
                    with open(current_script, 'r', encoding='utf-8') as f:
                        content = f.read()
                        compile(content, str(current_script), 'exec')
                        if 'class MalScraper' not in content:
                            raise ValueError("Installed script missing required class")
                except (SyntaxError, ValueError) as e:
                    print(f"{Colors.RED}Installed update failed verification: {e}. Rolling back...{Colors.NORMAL}")
                    if backup_path.exists():
                        shutil.copy2(backup_path, current_script)
                        print(f"{Colors.GREEN}Rolled back to previous version.{Colors.NORMAL}")
                else:
                    print(f"{Colors.GREEN}{Colors.BOLD}Update successfully installed!{Colors.NORMAL}")
                    print(f"{Colors.CYAN}A backup of your previous version was saved to:{Colors.NORMAL} {backup_path}")
                    print(f"{Colors.CYAN}You are now running the latest version!{Colors.NORMAL}")
        
        # Clean up
        UPDATE_FLAG.unlink()
        temp_dir = Path(update_info.get("temp_dir", ""))
        if temp_dir and temp_dir.exists():
            try:
                shutil.rmtree(temp_dir)
            except Exception as e:
                print(f"{Colors.YELLOW}Warning: Could not clean up temp directory: {e}{Colors.NORMAL}")
                
    except Exception as e:
        print(f"{Colors.RED}{Colors.BOLD}Error finalizing update: {e}{Colors.NORMAL}")
        print(f"Continuing with current version.")
        try:
            UPDATE_FLAG.unlink()
        except Exception:
            pass

# Current version - Update this manually when releasing a new version
CURRENT_VERSION = "1.4.8"

# Splash-text for loading screens
SPLASH_TEXTS = [
    "🔎 Generating list...",
    "🔎 Scraping data...",
    "🔎 Spinning web...",
    "🔎 Hunting threats...",
    "🔎 Collecting indicators..."
]

# Exit messages
EXIT_MESSAGES = [
    "Bye... 👋😢",
    "Cya... 👋😢", 
    "Byeeeeeeeeeeee... 👋😢",
    "Until next time... 👋",
    "Happy hunting! 👋"
]

# Feed URLs
FEEDS = {
    "payload_feed": "https://urlhaus.abuse.ch/downloads/text/",
    "c2_feed": "http://cybercrime-tracker.net/all.php",
    "hex_feed": "https://raw.githubusercontent.com/Neo23x0/signature-base/master/iocs/hash-iocs.txt",
    "phish_tank": "https://data.phishtank.com/data/online-valid.csv",
    "haus_mal_down": "https://urlhaus.abuse.ch/downloads/csv/"
}

# GitHub API for version checking
RELEASE_URL = "https://api.github.com/repos/Ryan-Monaghan/malScraper/releases/latest"

class MalScraper:
    """Main application class for malScraper"""
    
    def __init__(self):
        """Initialize the application"""
        # Setup base directory and paths
        self.base_dir = self._get_base_dir()
        self.paths = self._setup_paths()
        self.ensure_directories()
        
        # Setup signal handlers
        signal.signal(signal.SIGINT, self._handle_exit)  
        if platform.system() != "Windows":
            signal.signal(signal.SIGTERM, self._handle_exit)
        
        # Setup tab completion
        self.completer = CommandCompleter()
    
    def _get_base_dir(self):
        """Determine the appropriate base directory for the platform"""
        home = Path.home()
        
        if platform.system() == "Windows":
            return home / "Documents" / "malScraper"
        else:  # For Mac and Linux
            return home / "Desktop" / "malScraper"
    
    def _setup_paths(self):
        """Setup file paths based on the base directory"""
        return {
            "payload_report": self.base_dir / "PayloadReport.txt",
            "amp_report": self.base_dir / "AMPReport.txt",
            "c2_report": self.base_dir / "C2Report.txt",
            "top_100": self.base_dir / "Top100.txt",
            "hex_report": self.base_dir / "HexReport.csv",
            "haus_mal_down": self.base_dir / "HausMalDown.csv",
            "phish_tank": self.base_dir / "Phishing" / "PhishTank.csv",
            "updates_dir": self.base_dir / "Updates"
        }
    
    def ensure_directories(self):
        """Ensure all required directories exist"""
        self.base_dir.mkdir(parents=True, exist_ok=True)
        (self.base_dir / "Phishing").mkdir(exist_ok=True)
        (self.base_dir / "Updates").mkdir(exist_ok=True)

    def _get_terminal_width(self):
        """Get the current terminal width, or return 80 if unavailable"""
        try:
            return os.get_terminal_size().columns
        except OSError:
            return 80  # Fallback width
    
    def _clear_screen(self):
        """Clear the terminal screen using subprocess for security"""
        try:
            if platform.system() == 'Windows':
                subprocess.run(['cls'], shell=True, check=False)
            else:
                subprocess.run(['clear'], check=False)
        except Exception:
            # Fallback: print newlines if subprocess fails
            print('\n' * 50)
    
    def _print_banner(self):
        """Print the application banner"""
        self._clear_screen()
        width = self._get_terminal_width()
        
        # Use pyfiglet with a nice font if available
        banner = pyfiglet.figlet_format("malScraper", font="slant")
        for line in banner.splitlines():
            print(line[:width])
        
        # Application info using Printer helper
        Printer.label_value("Tool", "malScraper")
        Printer.label_value("Author", "Ryan Monaghan")
        Printer.label_value("Bluesky", "https://bsky.app/profile/rynmon.ie")
        Printer.label_value("Website", "https://rynmon.ie")
        Printer.label_value("Github", "https://github.com/rynmon/malScraper")
        Printer.label_value("Branch", "Stable")
        Printer.label_value("Version", f"{CURRENT_VERSION} (Python-compatible)")
        print(f"\t{Colors.CYAN}Tab Completion{Colors.NORMAL} :: {Colors.GREEN}Enabled (prompt_toolkit){Colors.NORMAL}")
        print(f"{Colors.NORMAL}\n")
    
    def _print_help(self):
        """Display the help menu"""
        Printer.header("HELP MENU :: Available options shown below:")
        print()
        
        # Print menu items from data structure
        for item in HELP_MENU_ITEMS:
            Printer.menu_item(
                item["description"],
                item["commands"],
                item.get("highlight")
            )
        
        print(f"\n{Colors.CYAN}💡 Tip:{Colors.NORMAL} Press {Colors.YELLOW}TAB{Colors.NORMAL} to auto-complete commands!")
        print()
    
    def _show_help_menu(self):
        """Helper method to show help menu (for lambda in command_map)"""
        self._clear_screen()
        self._print_help()

    def _print_directory_list(self):
        """Display the directory paths where reports are stored"""
        self._clear_screen()
        Printer.success("Success - Files written to:")
        
        # Print directory items from data structure
        for item in DIRECTORY_LIST_ITEMS:
            Printer.directory_item(
                item["number"],
                item["name"],
                self.paths[item["path_key"]]
            )
        print()
    
    def _open_file(self, file_path):
        """Open a file with the default application for the platform"""
        try:
            if platform.system() == "Windows":
                os.startfile(file_path)
            elif platform.system() == "Darwin":  # macOS
                subprocess.run(["open", str(file_path)], check=True)
            else:  # Linux
                subprocess.run(["xdg-open", str(file_path)], check=True)
            return True
        except Exception as e:
            Printer.error(f"Error opening file: {e}")
            return False
    
    def _download_file(self, url, output_path, description=None):
        """Download a file with progress indicator"""
        try:
            if description:
                Printer.info(f"Downloading {description}...")
            response = requests.get(url, stream=True, timeout=REQUEST_TIMEOUT)
            response.raise_for_status()  # Raise exception for HTTP errors
            total_size = int(response.headers.get('content-length', 0))
            with open(output_path, 'wb') as f:
                if total_size > 0:
                    # If we know the size, show a progress bar
                    downloaded = 0
                    progress_chars = PROGRESS_BAR_WIDTH
                    for chunk in response.iter_content(chunk_size=CHUNK_SIZE):
                        if chunk:
                            f.write(chunk)
                            downloaded += len(chunk)
                            percent = min(int(downloaded * 100 / total_size), 100)
                            filled = min(int(progress_chars * downloaded / total_size), progress_chars)
                            bar = '█' * filled + '░' * (progress_chars - filled)
                            sys.stdout.write(f"\r{Colors.CYAN}Progress: {Colors.NORMAL}[{bar}] {percent}%")
                            sys.stdout.flush()
                    sys.stdout.write("\n")  # Newline after progress bar
                else:
                    # If we don't know the size, just show activity
                    spinner = ['|', '/', '-', '\\']
                    idx = 0
                    for chunk in response.iter_content(chunk_size=CHUNK_SIZE):
                        if chunk:
                            f.write(chunk)
                            idx = (idx + 1) % len(spinner)
                            sys.stdout.write(f"\rDownloading... {spinner[idx]}")
                            sys.stdout.flush()
                    sys.stdout.write("\n")  # Newline after spinner
            return True
        except requests.exceptions.HTTPError as e:
            Printer.error(f"HTTP error: {e.response.status_code} {e.response.reason}")
            self._cleanup_failed_download(output_path)
            return False
        except requests.exceptions.ConnectionError:
            Printer.error("Connection error: Could not connect to server.")
            self._cleanup_failed_download(output_path)
            return False
        except requests.exceptions.Timeout:
            Printer.error("Timeout error: The request timed out.")
            self._cleanup_failed_download(output_path)
            return False
        except Exception as e:
            Printer.error(f"Unexpected error during download: {e}")
            self._cleanup_failed_download(output_path)
            return False
    
    def _cleanup_failed_download(self, output_path: Optional[Path]) -> None:
        """Clean up a failed download by removing the partial file"""
        if output_path and output_path.exists():
            try:
                output_path.unlink()
            except Exception as e:
                Printer.warning(f"Warning: Could not remove partial file {output_path}: {e}")
    
    def _calculate_checksum(self, file_path: Path) -> str:
        """Calculate SHA256 checksum of a file
        
        Args:
            file_path: Path to the file to checksum
            
        Returns:
            SHA256 checksum as hexadecimal string
        """
        sha256_hash = hashlib.sha256()
        try:
            with open(file_path, "rb") as f:
                for byte_block in iter(lambda: f.read(4096), b""):
                    sha256_hash.update(byte_block)
            return sha256_hash.hexdigest()
        except Exception as e:
            Printer.error(f"Error calculating checksum: {e}")
            return ""
    
    def _verify_checksum(self, file_path: Path, expected_checksum: Optional[str] = None) -> bool:
        """Verify file checksum (SHA256)
        
        Args:
            file_path: Path to the file to verify
            expected_checksum: Expected SHA256 checksum (hexadecimal string)
            
        Returns:
            True if checksum matches or if no expected checksum provided, False otherwise
        """
        if not expected_checksum:
            # If no checksum provided, just verify file exists and is readable
            return file_path.exists() and file_path.is_file()
        
        try:
            calculated = self._calculate_checksum(file_path)
            if not calculated:
                return False
            return calculated.lower() == expected_checksum.lower().strip()
        except Exception as e:
            Printer.error(f"Error verifying checksum: {e}")
            return False
    
    def _is_newer_version(self, current: str, latest: str) -> bool:
        """Compare versions using packaging library (more robust than manual parsing)
        
        Args:
            current: Current version string
            latest: Latest version string to compare against
            
        Returns:
            True if latest version is newer than current, False otherwise
        """
        try:
            # Handle 'v' prefix and strip whitespace
            current = current.lstrip('v').strip()
            latest = latest.lstrip('v').strip()
            
            # Use packaging library for proper semantic version comparison
            current_parsed = version.parse(current)
            latest_parsed = version.parse(latest)
            
            return latest_parsed > current_parsed
        except Exception as e:
            Printer.warning(f"Warning: Could not compare versions '{current}' and '{latest}': {e}")
            return False
    
    def _should_check_for_updates(self) -> bool:
        """Determine if we should check for updates (avoid checking too frequently)
        
        Returns:
            True if we should check for updates, False if we should skip
        """
        last_check_file = self.paths['updates_dir'] / '.last_update_check'
        
        # Always check if file doesn't exist
        if not last_check_file.exists():
            return True
        
        try:
            # Check if last check was more than 24 hours ago
            last_check_time = datetime.datetime.fromtimestamp(last_check_file.stat().st_mtime)
            hours_since_check = (datetime.datetime.now() - last_check_time).total_seconds() / 3600
            
            # Check once per day, or if forced
            return hours_since_check >= 24
        except Exception:
            # If we can't read the file, check anyway
            return True
    
    def _update_check_timestamp(self):
        """Update the timestamp of last update check"""
        last_check_file = self.paths['updates_dir'] / '.last_update_check'
        try:
            last_check_file.touch()
        except Exception:
            pass  # Non-critical, continue if this fails
    
    def _verify_update_success(self, script_path: Path) -> bool:
        """Verify that the updated script is valid and can be loaded
        
        Args:
            script_path: Path to the script to verify
            
        Returns:
            True if script is valid, False otherwise
        """
        try:
            # Try to parse the file as Python
            with open(script_path, 'r', encoding='utf-8') as f:
                content = f.read()
                # Quick syntax check - try to compile
                compile(content, str(script_path), 'exec')
            
            # Check that it has the expected structure
            if 'class MalScraper' not in content:
                Printer.error("Updated script missing required class.")
                return False
            
            return True
        except SyntaxError as e:
            Printer.error(f"Updated script has syntax errors: {e}")
            return False
        except Exception as e:
            Printer.error(f"Error verifying update: {e}")
            return False
    
    def _rollback_update(self, backup_path: Path, current_script: Path) -> bool:
        """Rollback to previous version if update fails
        
        Args:
            backup_path: Path to the backup file
            current_script: Path to the current script
            
        Returns:
            True if rollback successful, False otherwise
        """
        try:
            if not backup_path.exists():
                Printer.error("Backup not found, cannot rollback.")
                return False
            
            Printer.warning("Rolling back to previous version...")
            shutil.copy2(backup_path, current_script)
            Printer.success("Rollback successful.")
            return True
        except Exception as e:
            Printer.error(f"Rollback failed: {e}")
            return False
    
    def _unpack_result(self, result):
        """Helper to unpack result tuple consistently
        
        Args:
            result: Either a tuple (success, line_count) or just success bool
            
        Returns:
            Tuple of (success: bool, line_count: Optional[int])
        """
        if isinstance(result, tuple):
            return result
        return result, None
    
    def _download_with_status(self, name: str, feed_key: str, path_key: str, 
                              header: Optional[str] = None) -> bool:
        """Download a feed and update status
        
        Args:
            name: Display name for the feed
            feed_key: Key in FEEDS dictionary
            path_key: Key in paths dictionary
            header: Optional header to write to file
            
        Returns:
            True if download succeeded, False otherwise
        """
        print(f"{name}:")
        if header:
            with open(self.paths[path_key], 'w', encoding='utf-8') as f:
                f.write(header)
        start = time.time()
        success = self._download_file(FEEDS[feed_key], self.paths[path_key], name)
        elapsed = time.time() - start
        if success:
            Printer.success(f"Success ({elapsed:.1f}s)")
            Printer.success(f"{self.paths[path_key]} saved.\n")
        else:
            Printer.error("Failed\n")
        return success
    
    def _process_payload_report(self):
        """Process the payload report to create AMP report"""
        try:
            # Strip domains for easy blacklisting
            with open(self.paths['payload_report'], 'r', encoding='utf-8') as infile, \
                 open(self.paths['amp_report'], 'w', encoding='utf-8') as outfile:
                
                for line in infile:
                    # Find domains like "http://domain.com/"
                    match = re.search(r'http://([^/]*)', line)
                    if match:
                        domain = match.group(1)
                        # Remove www. prefix
                        domain = domain.replace('www.', '')
                        outfile.write(f"{domain}\n")
            
            # Create Top 100 report
            with open(self.paths['payload_report'], 'r', encoding='utf-8') as infile, \
                 open(self.paths['top_100'], 'w', encoding='utf-8') as outfile:
                
                for i, line in enumerate(infile):
                    if i >= TOP_DOMAINS_COUNT:
                        break
                    outfile.write(line)
                    
            return True
        
        except Exception as e:
            Printer.error(f"Error processing payload report: {e}")
            return False
    
    def _check_for_updates(self, force: bool = False) -> Tuple[bool, Optional[str], Optional[str], Optional[Dict]]:
        """Check for updates by querying the GitHub API
        
        Args:
            force: If True, check even if recently checked
            
        Returns:
            Tuple of (update_available, version, download_url, release_data)
        """
        # Check if we should skip this check
        if not force and not self._should_check_for_updates():
            return False, None, None, None
        
        try:
            Printer.info("Checking for updates...")
            
            response = requests.get(RELEASE_URL, timeout=UPDATE_CHECK_TIMEOUT)
            response.raise_for_status()
            
            release_data = response.json()
            latest_version = release_data.get('tag_name', '0.0')
            
            # Use packaging library for version comparison
            if not self._is_newer_version(CURRENT_VERSION, latest_version):
                Printer.success(f"Running latest version: {CURRENT_VERSION}")
                self._update_check_timestamp()
                time.sleep(1)
                return False, None, None, None
            
            # Update available
            print(f"\n{Colors.YELLOW}{Colors.BOLD}New version available!{Colors.NORMAL}")
            Printer.info(f"Current version: {CURRENT_VERSION}")
            Printer.info(f"Latest version: {latest_version}")
            
            # Get download URLs - prefer the asset if available, otherwise use zipball
            assets = release_data.get('assets', [])
            download_url = None
            checksum = None
            
            for asset in assets:
                name = asset.get('name', '')
                if name.endswith('.zip') or name.endswith('.py'):
                    download_url = asset.get('browser_download_url')
                    # Look for checksum file (common naming: filename.sha256 or filename.checksum)
                    base_name = name.rsplit('.', 1)[0] if '.' in name else name
                    checksum_asset = next(
                        (a for a in assets if a.get('name', '').startswith(base_name) and 
                         (a.get('name', '').endswith('.sha256') or 
                          a.get('name', '').endswith('.checksum') or
                          'sha256' in a.get('name', '').lower())),
                        None
                    )
                    if checksum_asset:
                        try:
                            checksum_response = requests.get(
                                checksum_asset.get('browser_download_url'),
                                timeout=UPDATE_CHECK_TIMEOUT
                            )
                            if checksum_response.ok:
                                checksum_text = checksum_response.text.strip()
                                # Extract checksum (might be in format "checksum filename" or just "checksum")
                                checksum = checksum_text.split()[0] if checksum_text.split() else None
                        except Exception:
                            pass  # Non-critical, continue without checksum
                    break
            
            # Fallback to zipball if no suitable asset found
            if not download_url:
                download_url = release_data.get('zipball_url', '')
            
            # Get release notes for display
            release_notes = release_data.get('body', 'No release notes available')
            
            # Display release notes
            Printer.info("\nRelease notes:")
            notes_lines = release_notes.split('\n')
            for line in notes_lines[:5]:
                print(f"  {line}")
            if len(notes_lines) > 5:
                Printer.warning("  ...and more")
            
            # Prompt user
            while True:
                option = input(f"\n{Colors.GREEN}Would you like to update now? (Y/N): {Colors.NORMAL}").strip().upper()
                if option in ['YES', 'Y']:
                    self._update_check_timestamp()
                    # Include checksum in release_data if available
                    if checksum:
                        release_data['checksum'] = checksum
                    return True, latest_version, download_url, release_data
                elif option in ['NO', 'N']:
                    self._update_check_timestamp()
                    print("Continuing with current version...")
                    time.sleep(1)
                    return False, None, None, None
                else:
                    Printer.error("Invalid input. Please enter Y or N.")
                
        except requests.exceptions.ConnectionError:
            Printer.warning("Could not check for updates: No internet connection")
        except requests.exceptions.Timeout:
            Printer.warning("Update check timed out")
        except Exception as e:
            Printer.warning(f"Error checking for updates: {e}")
        
        time.sleep(1)
        return False, None, None, None
    
    def _handle_exit(self, signal_received=None, frame=None):
        """Handle exit gracefully"""
        # Clear line in case we're interrupting a prompt
        print("\r" + " " * 80 + "\r", end="")
        
        # Print goodbye message
        print(f"\n{random.choice(EXIT_MESSAGES)}")
        
        # Force exit
        sys.exit(0)
    
    def _confirm_exit(self):
        """Confirm exit"""
        while True:
            option = input("Are you sure? (Y/N) ").strip().upper()
            if option in ["Y", "YES"]:
                self._handle_exit()
            elif option in ["N", "NO"]:
                break
            else:
                print(f"{Colors.RED}Invalid input. Please enter Y or N.{Colors.NORMAL}")
    
    def _warn_defender(self):
        print(f"{Colors.YELLOW}{Colors.BOLD}Warning:{Colors.NORMAL} Some reports may be flagged or quarantined by antivirus software (such as Windows Defender) because they contain known malware indicators. These files are for research and defensive use only.")

    def _obfuscate_payload_report(self):
        """Obfuscate PayloadReport.txt by replacing http with hxxp"""
        try:
            with open(self.paths['payload_report'], 'r', encoding='utf-8') as infile:
                lines = infile.readlines()
            with open(self.paths['payload_report'], 'w', encoding='utf-8') as outfile:
                for line in lines:
                    outfile.write(line.replace('http', 'hxxp'))
            Printer.success("PayloadReport.txt obfuscated (http -> hxxp).")
        except Exception as e:
            Printer.error(f"Error obfuscating PayloadReport.txt: {e}")

    def _zip_payload_report(self):
        """Zip PayloadReport.txt as PayloadReport.zip"""
        try:
            zip_path = self.paths['payload_report'].with_suffix('.zip')
            with zipfile.ZipFile(zip_path, 'w', zipfile.ZIP_DEFLATED) as zipf:
                zipf.write(self.paths['payload_report'], arcname='PayloadReport.txt')
            Printer.success(f"PayloadReport.txt zipped as {zip_path}.")
        except Exception as e:
            Printer.error(f"Error zipping PayloadReport.txt: {e}")

    def _get_payload_option(self):
        """Prompt user for PayloadReport.txt handling option and return choice."""
        self._warn_defender()
        while True:
            print("\nHow would you like to handle PayloadReport.txt?")
            print(f"{Colors.CYAN}1{Colors.NORMAL}: Leave as is (may be flagged by antivirus)")
            print(f"{Colors.CYAN}2{Colors.NORMAL}: Obfuscate IOCs (replace http with hxxp)")
            print(f"{Colors.CYAN}3{Colors.NORMAL}: Save as zip (PayloadReport.zip)")
            print(f"{Colors.CYAN}4{Colors.NORMAL}: Both obfuscate and zip")
            choice = input("Enter your choice (1-4): ").strip()
            if choice in {'1', '2', '3', '4'}:
                return choice
            else:
                Printer.error("Invalid input. Please enter 1, 2, 3, or 4.")

    def _download_payload_feed_with_options(self, choice: str, return_line_count: bool = False) -> Tuple[bool, Optional[int]]:
        """Download payload feed with obfuscation/zip options
        
        Args:
            choice: User's choice for handling the payload report ('1', '2', '3', or '4')
            return_line_count: Whether to return the line count of the downloaded data
            
        Returns:
            Tuple of (success: bool, line_count: Optional[int])
        """
        Printer.info("Downloading Payload Domains feed...")
        try:
            response = requests.get(FEEDS["payload_feed"], timeout=REQUEST_TIMEOUT)
            response.raise_for_status()
            data = response.text
            line_count = len(data.splitlines())
            Printer.success("Download complete.")
            time.sleep(1.5)
        except Exception as e:
            Printer.error(f"Failed to download payload report: {e}")
            if return_line_count:
                return False, None
            return False
        # Obfuscate in memory if needed
        if choice in {'2', '4'}:
            data = data.replace('http', 'hxxp')
        # Write to disk as chosen
        if choice in {'1', '2'}:
            with open(self.paths['payload_report'], 'w', encoding='utf-8') as f:
                f.write(data)
            Printer.success("PayloadReport.txt saved.")
        if choice in {'3', '4'}:
            zip_path = self.paths['payload_report'].with_suffix('.zip')
            with zipfile.ZipFile(zip_path, 'w', zipfile.ZIP_DEFLATED) as zipf:
                zipf.writestr('PayloadReport.txt', data)
            Printer.success(f"PayloadReport.txt zipped as {zip_path}.")
        if return_line_count:
            return True, line_count
        return True

    def full_scan(self):
        """Perform a full scan of all feeds"""
        self._clear_screen()
        print(datetime.datetime.now())
        print(random.choice(SPLASH_TEXTS))
        
        # Prompt for PayloadReport.txt handling before any downloads
        payload_option = self._get_payload_option()
        
        # Remove existing reports if they exist
        for path in self.paths.values():
            if isinstance(path, Path) and path.is_file():
                path.unlink()
        
        status = {}
        payload_line_count = None
        print(f"{Colors.BOLD}Starting downloads...{Colors.NORMAL}\n")
        
        # Download standard feeds using data structure
        for download in FULL_SCAN_DOWNLOADS:
            status[download["name"]] = self._download_with_status(
                download["name"],
                download["feed_key"],
                download["path_key"],
                download.get("header")
            )
        
        # Payload report (special handling)
        print(f"Payload domains:")
        start = time.time()
        payload_success, payload_line_count = self._unpack_result(
            self._download_payload_feed_with_options(payload_option, return_line_count=True)
        )
        elapsed = time.time() - start
        if payload_success:
            Printer.success(f"Success ({elapsed:.1f}s)")
            if payload_option in {'1', '2'}:
                Printer.success(f"{self.paths['payload_report']} saved.")
            if payload_option in {'3', '4'}:
                zip_path = self.paths['payload_report'].with_suffix('.zip')
                Printer.success(f"{zip_path} saved.")
            print()
            if self.paths['payload_report'].exists():
                status["Payload domains"] = self._process_payload_report()
            else:
                status["Payload domains"] = True
        else:
            Printer.error("Failed\n")
            status["Payload domains"] = False
        
        time.sleep(1.5)
        # Print summary
        succeeded = [k for k, v in status.items() if v]
        failed = [k for k, v in status.items() if not v]
        print(f"{Colors.BOLD}Download Summary:{Colors.NORMAL}")
        print(f"- {len(succeeded)}/{len(status)} downloads succeeded.")
        if payload_line_count is not None:
            print(f"- Payload domains: {payload_line_count} lines")
        if failed:
            print(f"- {len(failed)} download(s) failed: {', '.join(failed)}.")
        print()
        if not all(status.values()):
            Printer.warning("Warning: Some downloads may have failed. Check the reports.\n")
        input("Press Enter to continue...")
        self._clear_screen()
        self._print_directory_list()
        self.show_home()

    def quick_scan(self):
        """Perform a quick scan - only download the most recent 100 payload domains"""
        self._clear_screen()
        print(datetime.datetime.now())
        print(random.choice(SPLASH_TEXTS))
        
        # Prompt for PayloadReport.txt handling before any downloads
        payload_option = self._get_payload_option()
        
        # Remove existing reports
        for path in [self.paths['payload_report'], self.paths['top_100']]:
            if isinstance(path, Path) and path.is_file():
                path.unlink()
        
        # Download payload report with options (in-memory)
        payload_success, _ = self._unpack_result(
            self._download_payload_feed_with_options(payload_option, return_line_count=True)
        )
        if payload_success:
            if self.paths['payload_report'].exists():
                self._process_payload_report()
        else:
            Printer.error("Failed to download payload report.")
            time.sleep(2)
        time.sleep(1.5)
        self.show_home()
    
    def _prompt_open_report(self, first_time=False):
        """Prompt user to open a report"""
        first = first_time
        while True:
            if not first:
                self._clear_screen()
                self._print_directory_list()
            else:
                first = False
            option = input("Which feed would you like to open? ")
            print()
            # Use DIRECTORY_LIST_ITEMS data structure
            valid_options = {str(item["number"]): self.paths[item["path_key"]] 
                           for item in DIRECTORY_LIST_ITEMS}
            if option in valid_options:
                self._open_file(valid_options[option])
                break
            elif option.lower() == "home":
                break
            else:
                Printer.error("Invalid Option.")
                input("Press Enter to try again...")

    def reopen(self):
        """Allow user to reopen a previously downloaded report"""
        self._clear_screen()
        self._print_directory_list()
        self._prompt_open_report()
    
    def tutorial(self):
        """Display the tutorial text"""
        tut_text = f"""
{Colors.BOLD}MalScraper Tutorial{Colors.NORMAL}

{Colors.BOLD}NAME{Colors.NORMAL}
 - malScraper.py: Scrapes a list of Payload Domains, IOC's & C2 IPs from various feeds for easy blacklisting.

{Colors.BOLD}SYNOPSIS{Colors.NORMAL}
 - Run: {Colors.CYAN}python malScraper.py{Colors.NORMAL}
 - Example: {Colors.CYAN}python malScraper.py{Colors.NORMAL}

{Colors.BOLD}DESCRIPTION{Colors.NORMAL}
 - A cross-platform tool for collecting malware information from various feeds.

{Colors.BOLD}WORKFLOW{Colors.NORMAL}
  1. Run {Colors.CYAN}Quick-Scan{Colors.NORMAL} for a fast check of the most recent 100 domains.
  2. Run {Colors.CYAN}Full-Scan{Colors.NORMAL} to gather comprehensive data from all sources.
  3. Use the numbered menu to open specific reports.
  4. Reports are saved to your {Colors.CYAN}Desktop{Colors.NORMAL} (Mac/Linux) or {Colors.CYAN}Documents{Colors.NORMAL} (Windows) folder.

{Colors.BOLD}MENU NAVIGATION{Colors.NORMAL}
 - Type {Colors.CYAN}HELP{Colors.NORMAL} to see available commands.
 - Type {Colors.CYAN}TUTORIAL{Colors.NORMAL} to view this tutorial again.
 - Type {Colors.CYAN}UPDATE{Colors.NORMAL} to check for updates.
 - Type {Colors.CYAN}QUIT{Colors.NORMAL} to exit the application.

"""
        self._clear_screen()
        print(tut_text)
    
    def install_update(self, version: Optional[str] = None, download_url: Optional[str] = None, release_data: Optional[Dict] = None):
        """Install the downloaded update (atomic, on next launch) with improvements
        
        Args:
            version: Version string of the update
            download_url: URL to download the update from
            release_data: Release data from GitHub API (may include checksum)
        """
        # Only check for updates if not already provided
        if not (version and download_url and release_data):
            update_available, version, download_url, release_data = self._check_for_updates(force=True)
            if not update_available:
                return
        
        # Extract checksum if available
        expected_checksum = release_data.get('checksum') if release_data else None
        
        # Download the update
        update_file = self.paths['updates_dir'] / f"malScraper-{version}.zip"
        Printer.info("Downloading update...")
        
        if not self._download_file(download_url, update_file, f"malScraper version {version}"):
            Printer.error("Update download failed.")
            time.sleep(2)
            return
        
        # Verify checksum if available
        if expected_checksum:
            Printer.info("Verifying download integrity...")
            if not self._verify_checksum(update_file, expected_checksum):
                Printer.error("Checksum verification failed! Update may be corrupted.")
                Printer.warning(f"Expected: {expected_checksum}")
                Printer.warning(f"Got: {self._calculate_checksum(update_file)}")
                try:
                    update_file.unlink()
                except Exception:
                    pass
                time.sleep(2)
                return
            Printer.success("Checksum verified.")
        
        Printer.info("Preparing update...")
        
        try:
            # Get the current script path
            current_script = Path(__file__).resolve()
            script_name = current_script.name
            backup_path = current_script.with_suffix('.bak')
            
            # Create backup BEFORE doing anything
            Printer.info("Creating backup...")
            shutil.copy2(current_script, backup_path)
            
            # Create a temporary directory for extraction
            temp_dir = self.paths['updates_dir'] / f"temp_{version}"
            if temp_dir.exists():
                shutil.rmtree(temp_dir)
            temp_dir.mkdir(exist_ok=True)
            
            # Extract the update
            Printer.info("Extracting update...")
            with zipfile.ZipFile(update_file, 'r') as zip_ref:
                zip_ref.extractall(temp_dir)
            
            # Find the updated script (it might be in a subdirectory after extraction)
            new_script = next(
                (path for path in temp_dir.glob('**/*.py') 
                 if path.name == script_name or path.name == 'malScraper.py'),
                None
            )
            
            if not new_script:
                Printer.error("Could not find the updated script in the package.")
                time.sleep(2)
                return
            
            # Verify the new script before installing
            Printer.info("Verifying update...")
            if not self._verify_update_success(new_script):
                Printer.error("Update verification failed. Rolling back...")
                self._rollback_update(backup_path, current_script)
                return
            
            # Write update flag for atomic replacement on next launch
            update_flag = Path(__file__).parent / "update_pending.json"
            with open(update_flag, "w", encoding='utf-8') as f:
                json.dump({
                    "new_script_path": str(new_script),
                    "temp_dir": str(temp_dir),
                    "backup_path": str(backup_path),
                    "version": version
                }, f, indent=2)
            
            Printer.success("Update prepared successfully!")
            Printer.info("The new version will be installed the next time you start malScraper.")
            Printer.info("Please exit and restart the application to complete the update.")
            input("Press Enter to exit and complete the update...")
            sys.exit(0)
            
        except Exception as e:
            Printer.error(f"Error preparing update: {e}")
            # Try to rollback if backup exists
            if 'backup_path' in locals() and backup_path.exists():
                self._rollback_update(backup_path, current_script)
            time.sleep(2)
    
    def show_home(self):
        """Display the home menu"""
        self._print_banner()
        self._print_help()

    def process_command(self, option):
        """Process user command input"""
        option = option.upper()
        
        # Command mapping dictionary
        command_map = {
            ("FULL", "FULL-SCAN", "FSCAN"): self.full_scan,
            ("QUICK", "QUICK-SCAN", "QSCAN"): self.quick_scan,
            ("QUIT", "EXIT"): self._confirm_exit,
            ("CLEAR", "CLEAR-HOST", "CLS"): self._clear_screen,
            ("HELP", "GET-HELP", "?", "-?", "/?", "MENU"): lambda: self._show_help_menu(),
            ("BACK", "CD ..", "HOME"): self.show_home,
            ("TUTORIAL",): self.tutorial,
            ("REOPEN", "OPEN"): self.reopen,
            ("INSTALL", "UPDATE"): self.install_update,
        }
        
        # Find matching command
        for commands, handler in command_map.items():
            if option in commands:
                handler()
                return
        
        # Invalid command
        self._clear_screen()
        Printer.error("Error - invalid operation\n")
        self._print_help()
    
    def run(self):
        """Main application loop"""
        # Setup required directories
        self.ensure_directories()
        
        # Check for updates (with caching - won't check if checked recently)
        update_available, version, download_url, release_data = self._check_for_updates(force=False)
        if update_available:
            self.install_update(version, download_url, release_data)
        
        # Show initial banner and help
        self.show_home()

        # Main application loop
        while True:
            try:
                command = prompt("malScraper> ", completer=self.completer.prompt_completer)
                self.process_command(command)
            except EOFError:  # Handle Ctrl+D
                self._handle_exit()
            except KeyboardInterrupt:  # Handle Ctrl+C
                self._handle_exit()
            except Exception as e:
                Printer.error(f"\nUnexpected error: {str(e)}")
                print("The application will continue running. If this error persists, please restart.")
                time.sleep(2)


if __name__ == "__main__":
    app = MalScraper()
    app.run()
