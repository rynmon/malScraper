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

# --- Dependency check and auto-install (runs once at the very top) ---
required = {"requests", "pyfiglet", "prompt_toolkit"}
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
UPDATE_FLAG = Path(__file__).parent / "update_pending.json"
if UPDATE_FLAG.exists():
    try:
        with open(UPDATE_FLAG, "r") as f:
            update_info = json.load(f)
        new_script_path = Path(update_info["new_script_path"])
        current_script = Path(__file__).resolve()
        backup_path = current_script.with_suffix('.bak')
        # Backup current script
        shutil.copy2(current_script, backup_path)
        # Replace with new script
        shutil.copy2(new_script_path, current_script)
        print(f"{Colors.GREEN}{Colors.BOLD}Update successfully installed!{Colors.NORMAL}")
        print(f"{Colors.CYAN}A backup of your previous version was saved to:{Colors.NORMAL} {backup_path}")
        print(f"{Colors.CYAN}You are now running the latest version!{Colors.NORMAL}")
        # Clean up
        UPDATE_FLAG.unlink()
        # Optionally, remove temp dir
        temp_dir = Path(update_info.get("temp_dir", ""))
        if temp_dir and temp_dir.exists():
            shutil.rmtree(temp_dir)
    except Exception as e:
        print(f"{Colors.RED}{Colors.BOLD}Error finalizing update: {e}{Colors.NORMAL}")
        print(f"Continuing with current version.")
        UPDATE_FLAG.unlink()

# Current version - Update this manually when releasing a new version
CURRENT_VERSION = "1.4.7"

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
        
        # Application info
        print(f"\t{Colors.PURPLE}Tool\t :: malScraper")
        print(f"\tAuthor\t :: Ryan Monaghan")
        print(f"\tBluesky\t :: https://bsky.app/profile/rynmon.ie")
        print(f"\tWebsite\t :: https://rynmon.ie")
        print(f"\tGithub\t :: https://github.com/rynmon/malScraper")
        print(f"\tBranch\t :: Stable")
        print(f"\tVersion\t :: {CURRENT_VERSION} (Python-compatible)")
        print(f"\t{Colors.CYAN}Tab Completion{Colors.NORMAL} :: {Colors.GREEN}Enabled (prompt_toolkit){Colors.NORMAL}")
        print(f"{Colors.NORMAL}\n")
    
    def _print_help(self):
        """Display the help menu"""
        print(f"{Colors.CYAN}HELP MENU{Colors.NORMAL} {Colors.BOLD}::{Colors.NORMAL} Available {Colors.YELLOW}options{Colors.NORMAL} shown below:\n")
        print(f"{Colors.BOLD}[*]{Colors.NORMAL} {Colors.CYAN}Tutorial{Colors.NORMAL} of how to use this tool\t\t\t\t\t{Colors.YELLOW}TUTORIAL{Colors.NORMAL}")
        print(f"{Colors.BOLD}[*]{Colors.NORMAL} Show this {Colors.CYAN}Help{Colors.NORMAL} Menu\t\t\t\t\t\t\t{Colors.YELLOW}HELP,GET-HELP,?,-?,/?,MENU{Colors.NORMAL}")
        print(f"{Colors.BOLD}[*]{Colors.NORMAL} {Colors.CYAN}Clear{Colors.NORMAL} screen\t\t\t\t\t\t\t{Colors.YELLOW}CLEAR,CLEAR-HOST,CLS{Colors.NORMAL}")
        print(f"{Colors.BOLD}[*]{Colors.NORMAL} Return to {Colors.CYAN}Home{Colors.NORMAL} Menu\t\t\t\t\t\t\t{Colors.YELLOW}HOME,BACK,CD ..{Colors.NORMAL}")
        print(f"{Colors.BOLD}[*]{Colors.NORMAL} {Colors.CYAN}Open{Colors.NORMAL} an existing report\t\t\t\t\t\t{Colors.YELLOW}OPEN,REOPEN{Colors.NORMAL}")
        print(f"{Colors.BOLD}[*]{Colors.NORMAL} {Colors.CYAN}Quit{Colors.NORMAL} malScraper\t\t\t\t\t\t\t{Colors.YELLOW}QUIT,EXIT{Colors.NORMAL}")
        print(f"{Colors.BOLD}[*]{Colors.NORMAL} {Colors.CYAN}Install{Colors.NORMAL} the latest {Colors.CYAN}update{Colors.NORMAL}\t\t\t\t\t\t{Colors.YELLOW}INSTALL,UPDATE{Colors.NORMAL}")
        print(f"{Colors.BOLD}[*]{Colors.NORMAL} Perform {Colors.CYAN}Full-Scan{Colors.NORMAL} (Note this may take some time)\t\t\t{Colors.YELLOW}FULL,FULL-SCAN,FSCAN{Colors.NORMAL}")
        print(f"{Colors.BOLD}[*]{Colors.NORMAL} Perform {Colors.CYAN}Quick-Scan{Colors.NORMAL} (Most recent 100 Payload Domains)\t\t{Colors.YELLOW}QUICK,QUICK-SCAN,QSCAN{Colors.NORMAL}")
        print(f"\n{Colors.CYAN}💡 Tip:{Colors.NORMAL} Press {Colors.YELLOW}TAB{Colors.NORMAL} to auto-complete commands!")
        print()

    def _print_directory_list(self):
        """Display the directory paths where reports are stored"""
        self._clear_screen()
        print(f"{Colors.GREEN}{Colors.BOLD}Success - Files written to:{Colors.NORMAL}{Colors.NORMAL}")
        print(f"{Colors.GREEN}{Colors.BOLD}1. {Colors.NORMAL}{Colors.RED}{Colors.BOLD}Payload Domains:{Colors.NORMAL}{Colors.NORMAL} {self.paths['payload_report']}")
        print(f"{Colors.GREEN}{Colors.BOLD}2. {Colors.NORMAL}{Colors.RED}{Colors.BOLD}AMP Report:{Colors.NORMAL}{Colors.NORMAL} {self.paths['amp_report']}")
        print(f"{Colors.GREEN}{Colors.BOLD}3. {Colors.NORMAL}{Colors.RED}{Colors.BOLD}C2 Servers:{Colors.NORMAL}{Colors.NORMAL} {self.paths['c2_report']}")
        print(f"{Colors.GREEN}{Colors.BOLD}4. {Colors.NORMAL}{Colors.RED}{Colors.BOLD}Hex Report:{Colors.NORMAL}{Colors.NORMAL} {self.paths['hex_report']}")
        print(f"{Colors.GREEN}{Colors.BOLD}5. {Colors.NORMAL}{Colors.RED}{Colors.BOLD}URLHaus Maldownloads:{Colors.NORMAL}{Colors.NORMAL} {self.paths['haus_mal_down']}")
        print(f"{Colors.GREEN}{Colors.BOLD}6. {Colors.NORMAL}{Colors.RED}{Colors.BOLD}PhishTank Phishing Pages:{Colors.NORMAL}{Colors.NORMAL} {self.paths['phish_tank']}")
        print(f"{Colors.GREEN}{Colors.BOLD}7. {Colors.NORMAL}{Colors.RED}{Colors.BOLD}Most Recent 100:{Colors.NORMAL}{Colors.NORMAL} {self.paths['top_100']}\n")
    
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
            print(f"{Colors.RED}Error opening file: {e}{Colors.NORMAL}")
            return False
    
    def _download_file(self, url, output_path, description=None):
        """Download a file with progress indicator"""
        try:
            if description:
                print(f"Downloading {description}...")
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
            print(f"{Colors.RED}HTTP error: {e.response.status_code} {e.response.reason}{Colors.NORMAL}")
            self._cleanup_failed_download(output_path)
            return False
        except requests.exceptions.ConnectionError:
            print(f"{Colors.RED}Connection error: Could not connect to server.{Colors.NORMAL}")
            self._cleanup_failed_download(output_path)
            return False
        except requests.exceptions.Timeout:
            print(f"{Colors.RED}Timeout error: The request timed out.{Colors.NORMAL}")
            self._cleanup_failed_download(output_path)
            return False
        except Exception as e:
            print(f"{Colors.RED}Unexpected error during download: {e}{Colors.NORMAL}")
            self._cleanup_failed_download(output_path)
            return False
    
    def _cleanup_failed_download(self, output_path: Optional[Path]) -> None:
        """Clean up a failed download by removing the partial file"""
        if output_path and output_path.exists():
            try:
                output_path.unlink()
            except Exception as e:
                print(f"{Colors.YELLOW}Warning: Could not remove partial file {output_path}: {e}{Colors.NORMAL}")
    
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
            print(f"{Colors.RED}Error processing payload report: {e}{Colors.NORMAL}")
            return False
    
    def _check_for_updates(self):
        """Check for updates by querying the GitHub API"""
        try:
            print(f"{Colors.CYAN}Checking for updates...{Colors.NORMAL}")
            
            response = requests.get(RELEASE_URL, timeout=UPDATE_CHECK_TIMEOUT)
            response.raise_for_status()
            
            release_data = response.json()
            latest_version = release_data.get('tag_name', '0.0')
            
            # Get download URLs - prefer the asset if available, otherwise use zipball
            assets = release_data.get('assets', [])
            download_url = None
            for asset in assets:
                if asset.get('name', '').endswith('.zip') or asset.get('name', '').endswith('.py'):
                    download_url = asset.get('browser_download_url')
                    break
            
            # Fallback to zipball if no suitable asset found
            if not download_url:
                download_url = release_data.get('zipball_url', '')
            
            # Get release notes for display
            release_notes = release_data.get('body', 'No release notes available')
            
            # Compare versions using semantic versioning
            # Split version strings into components and convert to integers for comparison
            current_parts = [int(part) for part in CURRENT_VERSION.split('.')]
            latest_parts = [int(part) for part in latest_version.split('.')]
            
            # Pad the shorter version with zeros for proper comparison
            while len(current_parts) < len(latest_parts):
                current_parts.append(0)
            while len(latest_parts) < len(current_parts):
                latest_parts.append(0)
            
            # Compare each component
            is_newer_version = False
            for cur, lat in zip(current_parts, latest_parts):
                if lat > cur:
                    is_newer_version = True
                    break
                elif cur > lat:
                    # Current version is newer (development or pre-release version)
                    break
            
            if not is_newer_version:
                print(f"{Colors.GREEN}{Colors.BOLD}Running latest version: {CURRENT_VERSION}{Colors.NORMAL}")
                time.sleep(1)
                return False, None, None, None
            else:
                print(f"\n{Colors.YELLOW}{Colors.BOLD}New version available!{Colors.NORMAL}")
                print(f"{Colors.CYAN}Current version:{Colors.NORMAL} {CURRENT_VERSION}")
                print(f"{Colors.CYAN}Latest version:{Colors.NORMAL} {latest_version}")
                print(f"\n{Colors.CYAN}Release notes:{Colors.NORMAL}")
                
                # Format and display release notes (limit to ~5 lines for readability)
                notes_lines = release_notes.split('\n')
                for i, line in enumerate(notes_lines[:5]):
                    print(f"  {line}")
                if len(notes_lines) > 5:
                    print(f"  {Colors.YELLOW}...and more{Colors.NORMAL}")
                
                while True:
                    option = input(f"\n{Colors.GREEN}Would you like to update now? (Y/N): {Colors.NORMAL}").strip().upper()
                    if option in ['YES', 'Y']:
                        return True, latest_version, download_url, release_data
                    elif option in ['NO', 'N']:
                        print("Continuing with current version...")
                        time.sleep(1)
                        return False, None, None, None
                    else:
                        print(f"{Colors.RED}Invalid input. Please enter Y or N.{Colors.NORMAL}")
                
        except requests.exceptions.ConnectionError:
            print(f"{Colors.YELLOW}Could not check for updates: No internet connection{Colors.NORMAL}")
        except requests.exceptions.Timeout:
            print(f"{Colors.YELLOW}Update check timed out{Colors.NORMAL}")
        except Exception as e:
            print(f"{Colors.YELLOW}Error checking for updates: {e}{Colors.NORMAL}")
            
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
            print(f"{Colors.GREEN}PayloadReport.txt obfuscated (http -> hxxp).{Colors.NORMAL}")
        except Exception as e:
            print(f"{Colors.RED}Error obfuscating PayloadReport.txt: {e}{Colors.NORMAL}")

    def _zip_payload_report(self):
        """Zip PayloadReport.txt as PayloadReport.zip"""
        try:
            zip_path = self.paths['payload_report'].with_suffix('.zip')
            with zipfile.ZipFile(zip_path, 'w', zipfile.ZIP_DEFLATED) as zipf:
                zipf.write(self.paths['payload_report'], arcname='PayloadReport.txt')
            print(f"{Colors.GREEN}PayloadReport.txt zipped as {zip_path}.{Colors.NORMAL}")
        except Exception as e:
            print(f"{Colors.RED}Error zipping PayloadReport.txt: {e}{Colors.NORMAL}")

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
                print(f"{Colors.RED}Invalid input. Please enter 1, 2, 3, or 4.{Colors.NORMAL}")

    def _download_payload_feed_with_options(self, choice: str, return_line_count: bool = False) -> Tuple[bool, Optional[int]]:
        """Download payload feed with obfuscation/zip options
        
        Args:
            choice: User's choice for handling the payload report ('1', '2', '3', or '4')
            return_line_count: Whether to return the line count of the downloaded data
            
        Returns:
            Tuple of (success: bool, line_count: Optional[int])
        """
        print(f"{Colors.CYAN}Downloading Payload Domains feed...{Colors.NORMAL}")
        try:
            response = requests.get(FEEDS["payload_feed"], timeout=REQUEST_TIMEOUT)
            response.raise_for_status()
            data = response.text
            line_count = len(data.splitlines())
            print(f"{Colors.GREEN}Download complete.{Colors.NORMAL}")
            time.sleep(1.5)
        except Exception as e:
            print(f"{Colors.RED}Failed to download payload report: {e}{Colors.NORMAL}")
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
            print(f"{Colors.GREEN}PayloadReport.txt saved.{Colors.NORMAL}")
        if choice in {'3', '4'}:
            zip_path = self.paths['payload_report'].with_suffix('.zip')
            with zipfile.ZipFile(zip_path, 'w', zipfile.ZIP_DEFLATED) as zipf:
                zipf.writestr('PayloadReport.txt', data)
            print(f"{Colors.GREEN}PayloadReport.txt zipped as {zip_path}.{Colors.NORMAL}")
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
        
        # C2 report header
        with open(self.paths['c2_report'], 'w') as f:
            f.write("#############################################\n")
            f.write("# C2 Servers Report sourced from http://cybercrime-tracker.net/ \n")
            f.write("#############################################\n")
        
        status = {}
        payload_line_count = None
        print(f"{Colors.BOLD}Starting downloads...{Colors.NORMAL}\n")
        # C2 servers report
        print(f"C2 servers report:")
        start = time.time()
        if self._download_file(FEEDS["c2_feed"], self.paths['c2_report'], "C2 servers report"):
            elapsed = time.time() - start
            print(f"{Colors.GREEN}Success{Colors.NORMAL} ({elapsed:.1f}s)")
            print(f"{Colors.GREEN}{self.paths['c2_report']} saved.{Colors.NORMAL}\n")
            status["C2 servers report"] = True
        else:
            print(f"{Colors.RED}Failed{Colors.NORMAL}\n")
            status["C2 servers report"] = False
        # Hex report
        print(f"Hex report:")
        start = time.time()
        if self._download_file(FEEDS["hex_feed"], self.paths['hex_report'], "Hex report"):
            elapsed = time.time() - start
            print(f"{Colors.GREEN}Success{Colors.NORMAL} ({elapsed:.1f}s)")
            print(f"{Colors.GREEN}{self.paths['hex_report']} saved.{Colors.NORMAL}\n")
            status["Hex report"] = True
        else:
            print(f"{Colors.RED}Failed{Colors.NORMAL}\n")
            status["Hex report"] = False
        # Payload report (special handling)
        print(f"Payload domains:")
        start = time.time()
        result = self._download_payload_feed_with_options(payload_option, return_line_count=True)
        if isinstance(result, tuple):
            payload_success, payload_line_count = result
        else:
            payload_success = result
            payload_line_count = None
        elapsed = time.time() - start
        if payload_success:
            print(f"{Colors.GREEN}Success{Colors.NORMAL} ({elapsed:.1f}s)")
            # Print saved file(s) info for payload
            if payload_option in {'1', '2'}:
                print(f"{Colors.GREEN}{self.paths['payload_report']} saved.{Colors.NORMAL}")
            if payload_option in {'3', '4'}:
                zip_path = self.paths['payload_report'].with_suffix('.zip')
                print(f"{Colors.GREEN}{zip_path} saved.{Colors.NORMAL}")
            print()
            if self.paths['payload_report'].exists():
                if not self._process_payload_report():
                    status["Payload domains"] = False
                else:
                    status["Payload domains"] = True
            else:
                status["Payload domains"] = True
        else:
            print(f"{Colors.RED}Failed{Colors.NORMAL}\n")
            status["Payload domains"] = False
        # URLHaus Malware downloads
        print(f"URLHaus Malware downloads:")
        start = time.time()
        if self._download_file(FEEDS["haus_mal_down"], self.paths['haus_mal_down'], "URLHaus Malware downloads"):
            elapsed = time.time() - start
            print(f"{Colors.GREEN}Success{Colors.NORMAL} ({elapsed:.1f}s)")
            print(f"{Colors.GREEN}{self.paths['haus_mal_down']} saved.{Colors.NORMAL}\n")
            status["URLHaus Malware downloads"] = True
        else:
            print(f"{Colors.RED}Failed{Colors.NORMAL}\n")
            status["URLHaus Malware downloads"] = False
        # PhishTank data
        print(f"PhishTank data:")
        start = time.time()
        if self._download_file(FEEDS["phish_tank"], self.paths['phish_tank'], "PhishTank data"):
            elapsed = time.time() - start
            print(f"{Colors.GREEN}Success{Colors.NORMAL} ({elapsed:.1f}s)")
            print(f"{Colors.GREEN}{self.paths['phish_tank']} saved.{Colors.NORMAL}\n")
            status["PhishTank data"] = True
        else:
            print(f"{Colors.RED}Failed{Colors.NORMAL}\n")
            status["PhishTank data"] = False
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
            print(f"{Colors.YELLOW}{Colors.BOLD}Warning: {Colors.NORMAL}Some downloads may have failed. Check the reports.{Colors.NORMAL}\n")
        input("Press Enter to continue...")
        # Clear screen and show directory listing
        self._clear_screen()
        self._print_directory_list()
        # Return to home menu
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
        result = self._download_payload_feed_with_options(payload_option, return_line_count=True)
        if isinstance(result, tuple):
            payload_success, payload_line_count = result
        else:
            payload_success = result
            payload_line_count = None
        if payload_success:
            if self.paths['payload_report'].exists():
                self._process_payload_report()
        else:
            print(f"{Colors.RED}Failed to download payload report.{Colors.NORMAL}")
            time.sleep(2)
        time.sleep(1.5)
        # Return to home menu
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
            valid_options = {
                "1": self.paths['payload_report'],
                "2": self.paths['amp_report'],
                "3": self.paths['c2_report'],
                "4": self.paths['hex_report'],
                "5": self.paths['haus_mal_down'],
                "6": self.paths['phish_tank'],
                "7": self.paths['top_100']
            }
            if option in valid_options:
                self._open_file(valid_options[option])
                break
            elif option.lower() == "home":
                break
            else:
                print(f"{Colors.RED}{Colors.BOLD}Error: {Colors.NORMAL}Invalid Option.")
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
        """Install the downloaded update (atomic, on next launch)
        
        Args:
            version: Version string of the update
            download_url: URL to download the update from
            release_data: Release data from GitHub API
        """
        # Only check for updates if not already provided
        if not (version and download_url and release_data):
            update_available, version, download_url, release_data = self._check_for_updates()
            if not update_available:
                return
        
        # Download the update
        update_file = self.paths['updates_dir'] / f"malScraper-{version}.zip"
        if not self._download_file(download_url, update_file, f"malScraper version {version}"):
            print(f"{Colors.RED}Update download failed.{Colors.NORMAL}")
            time.sleep(2)
            return
        
        print(f"{Colors.CYAN}Preparing update...{Colors.NORMAL}")
        
        try:
            # Get the current script path
            current_script = Path(__file__).resolve()
            script_name = current_script.name
            
            # Create a temporary directory for extraction
            temp_dir = self.paths['updates_dir'] / f"temp_{version}"
            if temp_dir.exists():
                shutil.rmtree(temp_dir)
            temp_dir.mkdir(exist_ok=True)
            
            # Extract the update
            with zipfile.ZipFile(update_file, 'r') as zip_ref:
                zip_ref.extractall(temp_dir)
            
            # Find the updated script (it might be in a subdirectory after extraction)
            new_script = None
            for path in temp_dir.glob('**/*.py'):
                if path.name == script_name or path.name == 'malScraper.py':
                    new_script = path
                    break
            
            if not new_script:
                print(f"{Colors.RED}Could not find the updated script in the package.{Colors.NORMAL}")
                time.sleep(2)
                return
            
            # Write update flag for atomic replacement on next launch
            update_flag = Path(__file__).parent / "update_pending.json"
            with open(update_flag, "w") as f:
                json.dump({"new_script_path": str(new_script), "temp_dir": str(temp_dir)}, f)
            print(f"{Colors.GREEN}{Colors.BOLD}Update downloaded!{Colors.NORMAL}")
            print(f"{Colors.CYAN}The new version will be installed the next time you start malScraper.{Colors.NORMAL}")
            print(f"{Colors.CYAN}Please exit and restart the application to complete the update.{Colors.NORMAL}")
            input("Press Enter to exit and complete the update...")
            sys.exit(0)
        except Exception as e:
            print(f"{Colors.RED}{Colors.BOLD}Error preparing update: {e}{Colors.NORMAL}")
            time.sleep(2)
    
    def show_home(self):
        """Display the home menu"""
        self._print_banner()
        self._print_help()

    def process_command(self, option):
        """Process user command input"""
        option = option.upper()
        
        if option in ["FULL", "FULL-SCAN", "FSCAN"]:
            self.full_scan()
        
        elif option in ["QUICK", "QUICK-SCAN", "QSCAN"]:
            self.quick_scan()
        
        elif option in ["QUIT", "EXIT"]:
            self._confirm_exit()
        
        elif option in ["CLEAR", "CLEAR-HOST", "CLS"]:
            self._clear_screen()
        
        elif option in ["HELP", "GET-HELP", "?", "-?", "/?", "MENU"]:
            self._clear_screen()
            self._print_help()
        
        elif option in ["BACK", "CD ..", "HOME"]:
            # Fixed: Actually show the home menu
            self.show_home()
        
        elif option == "TUTORIAL":
            self.tutorial()
        
        elif option in ["REOPEN", "OPEN"]:
            self.reopen()
        
        elif option in ["INSTALL", "UPDATE"]:
            self.install_update()
        
        else:
            self._clear_screen()
            print(f"{Colors.RED}{Colors.BOLD}Error - {Colors.NORMAL}invalid operation\n")
            self._print_help()
    
    def run(self):
        """Main application loop"""
        # Setup required directories
        self.ensure_directories()
        
        # Check for updates
        update_available, version, download_url, release_data = self._check_for_updates()
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
                print(f"\n{Colors.RED}{Colors.BOLD}Unexpected error: {Colors.NORMAL}{str(e)}")
                print(f"The application will continue running. If this error persists, please restart.")
                time.sleep(2)


if __name__ == "__main__":
    app = MalScraper()
    app.run()