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
from pathlib import Path
import time
import importlib.util

# Colors for terminal output (works on Windows, Mac, Linux)
class Colors:
    RED = '\033[0;31m'
    GREEN = '\033[0;32m'
    YELLOW = '\033[1;33m'
    PURPLE = '\033[1;35m'
    CYAN = '\033[1;36m'
    BOLD = '\033[1m'
    NORMAL = '\033[0m'

required = {"requests", "pyfiglet"}
missing = [pkg for pkg in required if importlib.util.find_spec(pkg) is None]

if missing:
    print(f"{Colors.YELLOW}{Colors.BOLD}Missing required packages: {', '.join(missing)}{Colors.NORMAL}")
    choice = input("Would you like to install them now? (Y/N): ").strip().lower()
    if choice in ("y", "yes"):
        subprocess.check_call([sys.executable, "-m", "pip", "install", *missing])
        print("Please restart the application.")
    else:
        print("Cannot continue without required packages. Exiting.")
    sys.exit(1)

# --- Atomic update check (before any other logic) ---
UPDATE_FLAG = Path(__file__).parent / "update_pending.json"
if UPDATE_FLAG.exists():
    try:
        with open(UPDATE_FLAG, "r") as f:
            update_info = json.load(f)
        new_script_path = Path(update_info["new_script_path"])
        current_script = Path(os.path.abspath(sys.argv[0]))
        backup_path = current_script.with_suffix('.bak')
        # Backup current script
        import shutil
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

# Third-party imports
try:
    import requests
    HAS_REQUESTS = True
except ImportError:
    HAS_REQUESTS = False
    print("Warning: Requests library not found. Install with 'pip install requests'")

try:
    import pyfiglet
    HAS_PYFIGLET = True
except ImportError:
    HAS_PYFIGLET = False

# Current version - Update this manually when releasing a new version
CURRENT_VERSION = "1.4.3"

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
        """Clear the terminal screen"""
        os.system('cls' if platform.system() == 'Windows' else 'clear')
    
    def _print_banner(self):
        """Print the application banner"""
        self._clear_screen()
        width = self._get_terminal_width()
        
        if HAS_PYFIGLET:
            # Use pyfiglet with a nice font if available
            banner = pyfiglet.figlet_format("malScraper", font="slant")
            for line in banner.splitlines():
                print(line[:width])
        else:
            # Fallback to a hand-crafted ASCII art
            ascii_art = r"""
  __  __   __ _    _       ___   ___  ____    __    ____  ____  ____ 
 |  \/  | / _` |  | |     / __| / __||  _ \  / _\  |  _ \|  __||  _ \
 | |\/| || (_| |  | |__  | (__ | (__ | |_) |/    \ | |_) )  _| | |_) )
 |_|  |_| \__,_|  |____| \___|  \___||____/ \_/\_/ |  __/|____||  __/
                                                    |_|         |_|    
            """
            for line in ascii_art.splitlines():
                print(line[:width])
        
        # Application info
        print(f"\t{Colors.PURPLE}Tool\t :: malScraper")
        print(f"\tAuthor\t :: Ryan Monaghan")
        print(f"\tTwitter\t :: @rynmonaghan")
        print(f"\tWebsite\t :: https://rynmon.ie")
        print(f"\tGithub\t :: https://github.com/Ryan-Monaghan/malScraper")
        print(f"\tBranch\t :: Stable")
        print(f"\tVersion\t :: {CURRENT_VERSION} (Python-compatible){Colors.NORMAL}\n")
    
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
        print(f"{Colors.BOLD}[*]{Colors.NORMAL} Perform {Colors.CYAN}Quick-Scan{Colors.NORMAL} (Most recent 100 Payload Domains)\t\t{Colors.YELLOW}QUICK,QUICK-SCAN,QSCAN{Colors.NORMAL}\n")

    def _print_directory_list(self):
        """Display the directory paths where reports are stored"""
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
        if not HAS_REQUESTS:
            print(f"{Colors.RED}Error: Requests library not installed. Cannot download files.{Colors.NORMAL}")
            return False
            
        try:
            if description:
                print(f"Downloading {description}...")
            
            response = requests.get(url, stream=True)
            response.raise_for_status()  # Raise exception for HTTP errors
            
            total_size = int(response.headers.get('content-length', 0))
            
            with open(output_path, 'wb') as f:
                if total_size > 0:
                    # If we know the size, show a progress bar
                    downloaded = 0
                    progress_chars = 20
                    for chunk in response.iter_content(chunk_size=8192):
                        if chunk:
                            f.write(chunk)
                            downloaded += len(chunk)
                            
                            # Update progress bar
                            percent = int(downloaded * 100 / total_size)
                            filled = int(progress_chars * downloaded / total_size)
                            bar = '█' * filled + '░' * (progress_chars - filled)
                            
                            # Clear line and rewrite progress
                            sys.stdout.write(f"\r{Colors.CYAN}Progress: {Colors.NORMAL}[{bar}] {percent}%")
                            sys.stdout.flush()
                    
                    sys.stdout.write("\n")  # Newline after progress bar
                else:
                    # If we don't know the size, just show activity
                    spinner = ['|', '/', '-', '\\']
                    idx = 0
                    for chunk in response.iter_content(chunk_size=8192):
                        if chunk:
                            f.write(chunk)
                            idx = (idx + 1) % len(spinner)
                            sys.stdout.write(f"\rDownloading... {spinner[idx]}")
                            sys.stdout.flush()
                    
                    sys.stdout.write("\n")  # Newline after spinner
            
            return True
            
        except requests.exceptions.RequestException as e:
            print(f"{Colors.RED}Error downloading file: {e}{Colors.NORMAL}")
            return False
        except Exception as e:
            print(f"{Colors.RED}Unexpected error during download: {e}{Colors.NORMAL}")
            return False
    
    def _process_payload_report(self):
        """Process the payload report to create AMP report"""
        try:
            # Strip domains for easy blacklisting
            import re
            
            with open(self.paths['payload_report'], 'r') as infile, \
                 open(self.paths['amp_report'], 'w') as outfile:
                
                for line in infile:
                    # Find domains like "http://domain.com/"
                    match = re.search(r'http://([^/]*)', line)
                    if match:
                        domain = match.group(1)
                        # Remove www. prefix
                        domain = domain.replace('www.', '')
                        outfile.write(f"{domain}\n")
            
            # Create Top 100 report
            with open(self.paths['payload_report'], 'r') as infile, \
                 open(self.paths['top_100'], 'w') as outfile:
                
                for i, line in enumerate(infile):
                    if i >= 100:
                        break
                    outfile.write(line)
                    
            return True
        
        except Exception as e:
            print(f"{Colors.RED}Error processing payload report: {e}{Colors.NORMAL}")
            return False
    
    def _check_for_updates(self):
        """Check for updates by querying the GitHub API"""
        if not HAS_REQUESTS:
            print(f"{Colors.YELLOW}Cannot check for updates: Requests library not installed{Colors.NORMAL}")
            time.sleep(1)
            return False, None, None, None
            
        try:
            print(f"{Colors.CYAN}Checking for updates...{Colors.NORMAL}")
            
            response = requests.get(RELEASE_URL, timeout=10)
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
                
                option = input(f"\n{Colors.GREEN}Would you like to update now? (Y/N): {Colors.NORMAL}").upper()
                
                if option in ['YES', 'Y']:
                    return True, latest_version, download_url, release_data
                else:
                    print("Continuing with current version...")
                    time.sleep(1)
                    return False, None, None, None
                
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
        option = input("Are you sure? (Y/N) ").upper()
        if option in ["Y", "YES"]:
            self._handle_exit()
    
    def full_scan(self):
        """Perform a full scan of all feeds"""
        self._clear_screen()
        print(datetime.datetime.now())
        print(random.choice(SPLASH_TEXTS))
        
        # Remove existing reports if they exist
        for path in self.paths.values():
            if isinstance(path, Path) and path.is_file():
                path.unlink()
        
        # C2 report header
        with open(self.paths['c2_report'], 'w') as f:
            f.write("#############################################\n")
            f.write("# C2 Servers Report sourced from http://cybercrime-tracker.net/ \n")
            f.write("#############################################\n")
        
        # Download reports from feeds
        downloads_successful = True
        
        if not self._download_file(FEEDS["c2_feed"], self.paths['c2_report'], "C2 servers report"):
            downloads_successful = False
        
        if not self._download_file(FEEDS["hex_feed"], self.paths['hex_report'], "Hex report"):
            downloads_successful = False
        
        if not self._download_file(FEEDS["payload_feed"], self.paths['payload_report'], "Payload domains"):
            downloads_successful = False
        
        if not self._download_file(FEEDS["haus_mal_down"], self.paths['haus_mal_down'], "URLHaus Malware downloads"):
            downloads_successful = False
        
        if not self._download_file(FEEDS["phish_tank"], self.paths['phish_tank'], "PhishTank data"):
            downloads_successful = False
        
        # Process payload report to create AMP report (strip domains)
        if self.paths['payload_report'].exists():
            if not self._process_payload_report():
                downloads_successful = False
        
        # Clear screen and show directory listing
        self._clear_screen()
        
        if not downloads_successful:
            print(f"{Colors.YELLOW}{Colors.BOLD}Warning: {Colors.NORMAL}Some downloads may have failed. Check the reports.{Colors.NORMAL}\n")
        
        self._print_directory_list()
        
        # Ask which feed to open
        self._prompt_open_report()
    
    def quick_scan(self):
        """Perform a quick scan - only download the most recent 100 payload domains"""
        self._clear_screen()
        print(datetime.datetime.now())
        print(random.choice(SPLASH_TEXTS))
        
        # Remove existing reports
        for path in [self.paths['payload_report'], self.paths['top_100']]:
            if isinstance(path, Path) and path.is_file():
                path.unlink()
        
        # Download payload report
        if self._download_file(FEEDS["payload_feed"], self.paths['payload_report'], "Payload domains"):
            self._process_payload_report()
            self._open_file(self.paths['top_100'])
        else:
            print(f"{Colors.RED}Failed to download payload report.{Colors.NORMAL}")
            time.sleep(2)
    
    def _prompt_open_report(self):
        """Prompt user to open a report"""
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
        elif option.lower() == "home":
            return
        else:
            self._clear_screen()
            self._print_directory_list()
            print(f"{Colors.RED}{Colors.BOLD}Error: {Colors.NORMAL}Invalid Option.")
            self._prompt_open_report()
    
    def reopen(self):
        """Allow user to reopen a previously downloaded report"""
        self._clear_screen()
        self._print_directory_list()
        self._prompt_open_report()
    
    def tutorial(self):
        """Display the tutorial text"""
        tut_text = f"\n{Colors.BOLD}MalScraper\n\n{Colors.BOLD}NAME\n - {Colors.NORMAL}malScraper.py - malScraper scrapes a list of Payload Domains, IOC's & C2 IPs from various feeds, for easy blacklisting.\n\n{Colors.BOLD}SYNOPSIS\n python malScraper.py {Colors.NORMAL}\n{Colors.BOLD}e.g. - {Colors.NORMAL}python malScraper.py \n\n{Colors.BOLD}DESCRIPTION\n - {Colors.NORMAL}A cross-platform tool for collecting malware information from various feeds.\n\n{Colors.BOLD}WORKFLOW\n - {Colors.NORMAL}1. Run Quick-Scan for a fast check of the most recent 100 domains\n - 2. Run Full-Scan to gather comprehensive data from all sources\n - 3. Use the numbered menu to open specific reports\n - 4. Reports are saved to your Desktop (Linux/Mac) or Documents (Windows) folder\n"
        self._clear_screen()
        print(tut_text)
    
    def install_update(self, version=None, download_url=None, release_data=None):
        """Install the downloaded update (atomic, on next launch)"""
        import zipfile
        import shutil
        import json
        
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
            current_script = Path(os.path.abspath(sys.argv[0]))
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
                command = input("malScraper> ")
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