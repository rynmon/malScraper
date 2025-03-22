#!/usr/bin/env python3
"""
malScraper - Cross-platform tool to scrape malware domains, IOCs, and C2 IPs from various feeds
Converted from bash script to Python for Windows, Mac, and Linux compatibility
Original Author: Ryan Monaghan (@rynmonaghan)
"""

import os
import sys
import random
import base64
import gzip
import time
import datetime
import subprocess
import requests
import platform
import json
import re
import signal
from pathlib import Path

# Try to import pyfiglet for nice ASCII art headers
try:
    import pyfiglet
    HAS_PYFIGLET = True
except ImportError:
    HAS_PYFIGLET = False

# Current version - Update this manually when releasing a new version
CURRENT_VERSION = "1.4"

# Colors for terminal output (works on Windows, Mac, Linux)
class Colors:
    RED = '\033[0;31m'
    GREEN = '\033[0;32m'
    YELLOW = '\033[1;33m'
    PURPLE = '\033[1;35m'
    CYAN = '\033[1;36m'
    BOLD = '\033[1m'
    NORMAL = '\033[0m'

# Splash-text arrays (converted from bash)
SPLASH_TEXTS = [
    "🔎 Generating list...",
    "🔎 Scraping data...",
    "🔎 Spinning web..."
]

# Get current timestamp
TIMESTAMP = datetime.datetime.now().strftime("%Y-%m-%d-%H:%M")

# Setup base directory based on platform
def get_base_dir():
    """Determine the appropriate base directory for the platform"""
    home = str(Path.home())
    
    if platform.system() == "Windows":
        return os.path.join(home, "Documents", "malScraper")
    else:  # For Mac and Linux
        return os.path.join(home, "Desktop", "malScraper")

# Create base directory
BASE_DIR = get_base_dir()

# Data file paths
def setup_paths():
    """Setup file paths based on the base directory"""
    paths = {
        "payload_report": os.path.join(BASE_DIR, "PayloadReport.txt"),
        "amp_report": os.path.join(BASE_DIR, "AMPReport.txt"),
        "c2_report": os.path.join(BASE_DIR, "C2Report.txt"),
        "top_100": os.path.join(BASE_DIR, "Top100.txt"),
        "hex_report": os.path.join(BASE_DIR, "HexReport.csv"),
        "haus_mal_down": os.path.join(BASE_DIR, "HausMalDown.csv"),
        "phish_tank": os.path.join(BASE_DIR, "Phishing", "PhishTank.csv")
    }
    return paths

# Feed locations (decoded from base64)
FEEDS = {
    "payload_feed": "https://urlhaus.abuse.ch/downloads/text/",
    "c2_feed": "http://cybercrime-tracker.net/all.php",
    "hex_feed": "https://raw.githubusercontent.com/Neo23x0/signature-base/master/iocs/hash-iocs.txt",
    "phish_tank": "https://data.phishtank.com/data/online-valid.csv",
    "haus_mal_down": "https://urlhaus.abuse.ch/downloads/csv/"
}

# GitHub CodeLoad API for version checking
RELEASE_URL = "https://api.github.com/repos/Ryan-Monaghan/malScraper/releases/latest"

def ensure_directories():
    """Ensure all required directories exist"""
    os.makedirs(BASE_DIR, exist_ok=True)
    os.makedirs(os.path.join(BASE_DIR, "Phishing"), exist_ok=True)
    os.makedirs(os.path.join(BASE_DIR, "Updates"), exist_ok=True)
    
def print_banner():
    """Print a nice ASCII art banner for malScraper"""
    if HAS_PYFIGLET:
        # Use pyfiglet with a nice font if available
        print(pyfiglet.figlet_format("malScraper", font="slant"))
    else:
        # Fallback to a hand-crafted ASCII art
        print(r"""
  __  __   __ _    _       ___   ___  ____    __    ____  ____  ____ 
 |  \/  | / _` |  | |     / __| / __||  _ \  / _\  |  _ \|  __||  _ \
 | |\/| || (_| |  | |__  | (__ | (__ | |_) |/    \ | |_) )  _| | |_) )
 |_|  |_| \__,_|  |____| \___|  \___||____/ \_/\_/ |  __/|____||  __/
                                                    |_|         |_|    
        """)

def random_splash_text():
    """Return a random splash text"""
    return random.choice(SPLASH_TEXTS)

def open_file(filepath):
    """Open a file with the default application for the platform"""
    if platform.system() == "Windows":
        os.startfile(filepath)
    elif platform.system() == "Darwin":  # macOS
        subprocess.run(["open", filepath], check=True)
    else:  # Linux
        subprocess.run(["xdg-open", filepath], check=True)

def dir_list(paths):
    """Display the directory paths where reports are stored"""
    print(f"{Colors.GREEN}{Colors.BOLD}Success - Files written to:{Colors.NORMAL}{Colors.NORMAL}")
    print(f"{Colors.GREEN}{Colors.BOLD}1. {Colors.NORMAL}{Colors.RED}{Colors.BOLD}Payload Domains:{Colors.NORMAL}{Colors.NORMAL} {paths['payload_report']}")
    print(f"{Colors.GREEN}{Colors.BOLD}2. {Colors.NORMAL}{Colors.RED}{Colors.BOLD}AMP Report:{Colors.NORMAL}{Colors.NORMAL} {paths['amp_report']}")
    print(f"{Colors.GREEN}{Colors.BOLD}3. {Colors.NORMAL}{Colors.RED}{Colors.BOLD}C2 Servers:{Colors.NORMAL}{Colors.NORMAL} {paths['c2_report']}")
    print(f"{Colors.GREEN}{Colors.BOLD}4. {Colors.NORMAL}{Colors.RED}{Colors.BOLD}Hex Report:{Colors.NORMAL}{Colors.NORMAL} {paths['hex_report']}")
    print(f"{Colors.GREEN}{Colors.BOLD}5. {Colors.NORMAL}{Colors.RED}{Colors.BOLD}URLHaus Maldownloads:{Colors.NORMAL}{Colors.NORMAL} {paths['haus_mal_down']}")
    print(f"{Colors.GREEN}{Colors.BOLD}6. {Colors.NORMAL}{Colors.RED}{Colors.BOLD}PhishTank Phishing Pages:{Colors.NORMAL}{Colors.NORMAL} {paths['phish_tank']}")
    print(f"{Colors.GREEN}{Colors.BOLD}7. {Colors.NORMAL}{Colors.RED}{Colors.BOLD}Most Recent 100:{Colors.NORMAL}{Colors.NORMAL} {paths['top_100']}\n")

def full_scan(paths):
    """Perform a full scan of all feeds"""
    # Clear screen
    os.system('cls' if platform.system() == 'Windows' else 'clear')
    
    print(datetime.datetime.now())
    print(random_splash_text())
    
    # Remove existing reports if they exist
    for path in paths.values():
        if os.path.exists(path):
            os.remove(path)
    
    # C2 report header
    with open(paths['c2_report'], 'w') as f:
        f.write("#############################################\n")
        f.write("# C2 Servers Report sourced from http://cybercrime-tracker.net/ \n")
        f.write("#############################################\n")
    
    # Download reports from feeds
    print("Downloading C2 servers report...")
    response = requests.get(FEEDS["c2_feed"])
    with open(paths['c2_report'], 'a') as f:
        f.write(response.text)
    
    print("Downloading Hex report...")
    response = requests.get(FEEDS["hex_feed"])
    with open(paths['hex_report'], 'w') as f:
        f.write(response.text)
    
    print("Downloading Payload domains...")
    response = requests.get(FEEDS["payload_feed"])
    with open(paths['payload_report'], 'w') as f:
        f.write(response.text)
    
    print("Downloading URLHaus Malware downloads...")
    response = requests.get(FEEDS["haus_mal_down"])
    with open(paths['haus_mal_down'], 'w') as f:
        f.write(response.text)
    
    print("Downloading PhishTank data...")
    response = requests.get(FEEDS["phish_tank"])
    with open(paths['phish_tank'], 'w') as f:
        f.write(response.text)
    
    # Process payload report to create AMP report (strip domains)
    with open(paths['payload_report'], 'r') as f:
        payload_lines = f.readlines()
    
    import re
    with open(paths['amp_report'], 'w') as f:
        for line in payload_lines:
            # Find domains like "http://domain.com/"
            match = re.search(r'http://([^/]*)', line)
            if match:
                domain = match.group(1)
                # Remove www. prefix
                domain = domain.replace('www.', '')
                f.write(f"{domain}\n")
    
    # Create Top 100 report
    with open(paths['top_100'], 'w') as f:
        for i, line in enumerate(payload_lines):
            if i >= 100:
                break
            f.write(line)
    
    # Clear screen and show directory listing
    os.system('cls' if platform.system() == 'Windows' else 'clear')
    dir_list(paths)
    
    # Ask which feed to open
    option = input("Which feed would you like to open? ")
    print()
    
    while True:
        if option == "1":
            open_file(paths['payload_report'])
            return
        elif option == "2":
            open_file(paths['amp_report'])
            return
        elif option == "3":
            open_file(paths['c2_report'])
            return
        elif option == "4":
            open_file(paths['hex_report'])
            return
        elif option == "5":
            open_file(paths['haus_mal_down'])
            return
        elif option == "6":
            open_file(paths['phish_tank'])
            return
        elif option == "7":
            open_file(paths['top_100'])
            return
        else:
            os.system('cls' if platform.system() == 'Windows' else 'clear')
            dir_list(paths)
            print(f"{Colors.RED}{Colors.BOLD}Error: {Colors.NORMAL}{Colors.NORMAL}Invalid Option.")
            option = input("Which feed would you like to open? ")

def quick_scan(paths):
    """Perform a quick scan - only download the most recent 100 payload domains"""
    os.system('cls' if platform.system() == 'Windows' else 'clear')
    print(datetime.datetime.now())
    print(random_splash_text())
    
    # Remove existing reports if they exist
    for path in [paths['payload_report'], paths['top_100']]:
        if os.path.exists(path):
            os.remove(path)
    
    # Download payload report
    print("Downloading Payload domains...")
    response = requests.get(FEEDS["payload_feed"])
    with open(paths['payload_report'], 'w') as f:
        f.write(response.text)
    
    # Create Top 100 report
    with open(paths['payload_report'], 'r') as f:
        payload_lines = f.readlines()
    
    with open(paths['top_100'], 'w') as f:
        for i, line in enumerate(payload_lines):
            if i >= 100:
                break
            f.write(line)
    
    # Open the Top 100 report
    open_file(paths['top_100'])
    time.sleep(1)
    print()

def reopen(paths):
    """Allow user to reopen a previously downloaded report"""
    os.system('cls' if platform.system() == 'Windows' else 'clear')
    dir_list(paths)
    
    option = input("Which feed would you like to open? ")
    print()
    
    while True:
        if option == "1":
            open_file(paths['payload_report'])
            return
        elif option == "2":
            open_file(paths['amp_report'])
            return
        elif option == "3":
            open_file(paths['c2_report'])
            return
        elif option == "4":
            open_file(paths['hex_report'])
            return
        elif option == "5":
            open_file(paths['haus_mal_down'])
            return
        elif option == "6":
            open_file(paths['phish_tank'])
            return
        elif option == "7":
            open_file(paths['top_100'])
            return
        elif option.lower() == "home":
            return
        else:
            os.system('cls' if platform.system() == 'Windows' else 'clear')
            dir_list(paths)
            print(f"{Colors.RED}{Colors.BOLD}Error: {Colors.NORMAL}{Colors.NORMAL}Invalid Option.")
            option = input("Which feed would you like to open? ")

def version_check():
    """Check for updates by querying the GitHub API"""
    try:
        print(f"{Colors.CYAN}Checking for updates...{Colors.NORMAL}")
        response = requests.get(RELEASE_URL, timeout=10)
        
        if response.status_code == 200:
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
            
            # Compare versions
            if CURRENT_VERSION == latest_version:
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
                elif option in ['NO', 'N']:
                    print("Continuing with current version...")
                    time.sleep(1)
                    return False, None, None, None
                else:
                    print(f"{Colors.RED}{Colors.BOLD}Error: {Colors.NORMAL}Invalid Option.")
                    time.sleep(1)
                    return False, None, None, None
        else:
            print(f"{Colors.YELLOW}Could not check for updates (HTTP {response.status_code}){Colors.NORMAL}")
            time.sleep(1)
            return False, None, None, None
            
    except requests.exceptions.ConnectionError:
        print(f"{Colors.YELLOW}Could not check for updates: No internet connection{Colors.NORMAL}")
        time.sleep(1)
        return False, None, None, None
    except requests.exceptions.Timeout:
        print(f"{Colors.YELLOW}Update check timed out{Colors.NORMAL}")
        time.sleep(1)
        return False, None, None, None
    except Exception as e:
        print(f"{Colors.YELLOW}Error checking for updates: {e}{Colors.NORMAL}")
        time.sleep(1)
        return False, None, None, None

def download_update(version, download_url):
    """Download the update from GitHub"""
    try:
        update_dir = os.path.join(BASE_DIR, "Updates")
        os.makedirs(update_dir, exist_ok=True)
        
        update_file = os.path.join(update_dir, f"malScraper-{version}.zip")
        
        print(f"{Colors.CYAN}Downloading update...{Colors.NORMAL}")
        
        # Show a simple progress indicator
        response = requests.get(download_url, stream=True)
        total_size = int(response.headers.get('content-length', 0))
        
        if total_size > 0:
            # If we know the size, show a progress bar
            downloaded = 0
            progress_chars = 20
            with open(update_file, 'wb') as f:
                for chunk in response.iter_content(chunk_size=8192):
                    if chunk:
                        f.write(chunk)
                        downloaded += len(chunk)
                        
                        # Update progress bar
                        percent = int(downloaded * 100 / total_size)
                        filled = int(progress_chars * downloaded / total_size)
                        bar = '█' * filled + '░' * (progress_chars - filled)
                        
                        sys.stdout.write(f"\r{Colors.CYAN}Progress: {Colors.NORMAL}[{bar}] {percent}%")
                        sys.stdout.flush()
            
            print("\n")  # Newline after progress bar
        else:
            # If we don't know the size, just download without progress
            with open(update_file, 'wb') as f:
                for chunk in response.iter_content(chunk_size=8192):
                    if chunk:
                        f.write(chunk)
                        sys.stdout.write(".")
                        sys.stdout.flush()
            
            print("\n")  # Newline after dots
        
        return update_file
    
    except Exception as e:
        print(f"{Colors.RED}{Colors.BOLD}Error downloading update: {e}{Colors.NORMAL}")
        return None

def install_update():
    """Install the downloaded update"""
    import zipfile
    import shutil
    
    try:
        # First check for updates
        update_available, version, download_url, release_data = version_check()
        
        if not update_available:
            return
        
        # Download the update
        update_file = download_update(version, download_url)
        if not update_file:
            print(f"{Colors.RED}Update download failed.{Colors.NORMAL}")
            time.sleep(2)
            return
        
        print(f"{Colors.CYAN}Installing update...{Colors.NORMAL}")
        
        # Get the current script path
        current_script = os.path.abspath(sys.argv[0])
        script_dir = os.path.dirname(current_script)
        script_name = os.path.basename(current_script)
        
        # Create a temporary directory for extraction
        temp_dir = os.path.join(BASE_DIR, "Updates", "temp")
        if os.path.exists(temp_dir):
            shutil.rmtree(temp_dir)
        os.makedirs(temp_dir, exist_ok=True)
        
        # Extract the update
        with zipfile.ZipFile(update_file, 'r') as zip_ref:
            zip_ref.extractall(temp_dir)
        
        # Find the updated script (it might be in a subdirectory after extraction)
        new_script = None
        for root, dirs, files in os.walk(temp_dir):
            for file in files:
                if file.endswith('.py') and (file == script_name or file == 'malScraper.py'):
                    new_script = os.path.join(root, file)
                    break
            if new_script:
                break
        
        if not new_script:
            print(f"{Colors.RED}Could not find the updated script in the package.{Colors.NORMAL}")
            time.sleep(2)
            return
        
        # Create backup of current script
        backup_path = current_script + '.bak'
        shutil.copy2(current_script, backup_path)
        
        # Copy new script over current script
        shutil.copy2(new_script, current_script)
        
        print(f"{Colors.GREEN}{Colors.BOLD}Update successfully installed!{Colors.NORMAL}")
        print(f"{Colors.CYAN}A backup of your previous version was saved to:{Colors.NORMAL} {backup_path}")
        print(f"{Colors.CYAN}Restart the application to use the new version.{Colors.NORMAL}")
        
        time.sleep(3)
        
        # Ask if user wants to restart now
        restart = input(f"{Colors.GREEN}Would you like to restart now? (Y/N): {Colors.NORMAL}").upper()
        if restart in ['Y', 'YES']:
            print(f"{Colors.CYAN}Restarting...{Colors.NORMAL}")
            time.sleep(1)
            # Restart the script
            python = sys.executable
            os.execl(python, python, current_script)
        
    except Exception as e:
        print(f"{Colors.RED}{Colors.BOLD}Error installing update: {e}{Colors.NORMAL}")
        time.sleep(2)


def handle_exit(signal_received=None, frame=None):
    """Handle exit gracefully, whether from Ctrl+C or regular exit command"""
    exit_messages = [
        "Bye... 👋😢",
        "Cya... 👋😢", 
        "Byeeeeeeeeeeee... 👋😢"
    ]
    
    # Clear line in case we're interrupting a prompt
    print("\r" + " " * 80 + "\r", end="")
    
    # Print goodbye message
    print(f"\n{random.choice(exit_messages)}")
    
    # Force exit to avoid any hanging threads
    sys.exit(0)

def help_text():
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

def tutorial():
    """Display the tutorial text"""
    tut_text = f"\n{Colors.BOLD}MalScraper\n\n{Colors.BOLD}NAME\n - {Colors.NORMAL}malScraper.py - malScraper scrapes a list of Payload Domains, IOC's & C2 IPs from from various feeds, for easy blacklisting.\n\n{Colors.BOLD}SYNOPSIS\n python malScraper.py {Colors.NORMAL}\n{Colors.BOLD}e.g. - {Colors.NORMAL}python malScraper.py \n\n{Colors.BOLD}DESCRIPTION\n - {Colors.NORMAL}A cross-platform tool for collecting malware information from various feeds.\n"
    os.system('cls' if platform.system() == 'Windows' else 'clear')
    print(tut_text)

def user_options(paths):
    """Handle user input and execute corresponding functions"""
    try:
        option = input("malScraper> ").upper()
        
        if option in ["FULL", "FULL-SCAN", "FSCAN"]:
            full_scan(paths)
        elif option in ["QUICK", "QUICK-SCAN", "QSCAN"]:
            quick_scan(paths)
        elif option in ["QUIT", "EXIT"]:
            close_conf = input("Are you sure? (Y/N) ").upper()
            if close_conf in ["Y", "YES"]:
                handle_exit()
            # If not confirmed, just return to continue the loop
        elif option in ["CLEAR", "CLEAR-HOST", "CLS"]:
            os.system('cls' if platform.system() == 'Windows' else 'clear')
        elif option in ["HELP", "GET-HELP", "?", "-?", "/?", "MENU"]:
            help_text()
        elif option in ["BACK", "CD ..", "HOME"]:
            # Just continue main loop
            pass
        elif option == "TUTORIAL":
            tutorial()
        elif option in ["REOPEN", "OPEN"]:
            reopen(paths)
        elif option in ["INSTALL", "UPDATE"]:
            install_update()
        else:
            os.system('cls' if platform.system() == 'Windows' else 'clear')
            print(f"{Colors.RED}{Colors.BOLD}Error - {Colors.NORMAL}{Colors.NORMAL}invalid operation\n")
            help_text()
    except EOFError:  # Handle Ctrl+D
        handle_exit()
    except KeyboardInterrupt:  # Extra handler for Ctrl+C inside input()
        handle_exit()
    except Exception as e:
        # Catch all other exceptions to prevent crashes
        print(f"\n{Colors.RED}{Colors.BOLD}Error: {Colors.NORMAL}{str(e)}")
        print(f"If this error persists, please report it to the developers.")
        time.sleep(2)

def main():
    """Main function that handles the user interface"""
    # Setup signal handlers for graceful exit
    signal.signal(signal.SIGINT, handle_exit)  # Handle Ctrl+C
    if platform.system() != "Windows":
        signal.signal(signal.SIGTERM, handle_exit)  # Handle termination signal (not available on Windows)
    
    # Setup required directories
    ensure_directories()
    
    # Setup file paths
    paths = setup_paths()
    
    # Check for updates
    update_available, version, download_url, release_data = version_check()
    
    # Clear screen
    os.system('cls' if platform.system() == 'Windows' else 'clear')
    
    # Print application header
    print_banner()
    
    print(f"\t{Colors.PURPLE}Tool\t :: malScraper")
    print(f"\tAuthor\t :: Ryan Monaghan")
    print(f"\tTwitter\t :: @rynmonaghan")
    print(f"\tWebsite\t :: https://rynmon.ie")
    print(f"\tGithub\t :: https://github.com/Ryan-Monaghan/malScraper")
    print(f"\tBranch\t :: Stable")
    print(f"\tVersion\t :: {CURRENT_VERSION} (Python-compatible){Colors.NORMAL}\n")
    
    help_text()
    
    # Main application loop
    while True:
        try:
            user_options(paths)
        except Exception as e:
            print(f"\n{Colors.RED}{Colors.BOLD}Unexpected error: {Colors.NORMAL}{str(e)}")
            print(f"The application will continue running. If this error persists, please restart.")
            time.sleep(2)

if __name__ == "__main__":
    main()
