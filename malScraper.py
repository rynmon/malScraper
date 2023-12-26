#imports
import os
import time
import sys
import random
import ctypes
from pyfiglet import Figlet
from colorama import Fore, Style

#constants
RED=Fore.RED
GRN=Fore.GREEN
NC=Style.RESET_ALL
BOLD=Style.BRIGHT
NORMAL=Style.NORMAL
PURPLE=Fore.MAGENTA
CYAN=Fore.CYAN
YELLOW=Fore.YELLOW

#variables
emojis=["😊", "😂", "😍", "👍", "🎉", "🚀", "💻", "🌈"]
randomEmoji=random.choice(emojis)

#functions
def displayCountdown(seconds):
    for i in range(seconds, 0, -1):
        print(f"Returning to Main Menu in: {i}{' ' if i > 9 else '  '}", end='\r')
        time.sleep(1)

def clearMenu():
    os.system('cls' if os.name == 'nt' else 'clear')

def fullScan():
    print("DEBUG")

def quickScan():
    print("DEBUG")

def exitApp():
    print("Closing the app...")
    sys.exit()

def helpFunc():
    clearMenu()
    helpText()

def tutorial():
    print("Tutorial is still on the to-do list.")
    displayCountdown(3)
    main()

def reopen():
    print("DEBUG")

def installUpdate():
    print("DEBUG")

def helpText():
    print(f"\n{CYAN}HELP MENU{NC} :: Available {YELLOW}options{NC} shown below:\n")
    print(f"{BOLD}[*]{NC} {CYAN}Tutorial{NC} of how to use this tool\t\t\t\t\t{YELLOW}TUTORIAL{NC}")
    print(f"[*] Show this {CYAN}Help{NC} Menu\t\t\t\t\t\t\t{YELLOW}HELP,GET-HELP,?,-?,/?,MENU{NC}")
    print(f"[*] {CYAN}Show options{NC} for this tool\t\t\t\t\t\t{YELLOW}SHOW OPTIONS,SHOW,OPTIONS{NC}")
    print(f"[*] {CYAN}Clear{NC} screen\t\t\t\t\t\t\t{YELLOW}CLEAR,CLEAR-HOST,CLS{NC}")
    print(f"[*] Return to {CYAN}Home{NC} Menu\t\t\t\t\t\t\t{YELLOW}HOME,BACK,CD ..{NC}")
    print(f"[*] {CYAN}Open{NC} an existing report\t\t\t\t\t\t{YELLOW}OPEN,REOPEN{NC}")
    print(f"[*] {CYAN}Quit{NC} malScraper\t\t\t\t\t\t\t{YELLOW}QUIT,EXIT{NC}")
    print(f"[*] {CYAN}Install{NC} the latest {CYAN}update{NC}\t\t\t\t\t\t{YELLOW}INSTALL,UPDATE{NC}")
    print(f"[*] Perform {CYAN}Full-Scan{NC} (Note this may take some time)\t\t\t{YELLOW}FULL,FULL-SCAN,FSCAN{NC}")
    print(f"[*] Perform {CYAN}Quick-Scan{NC} (Most recent 100 Payload Domains)\t\t{YELLOW}QUICK,QUICK-SCAN,QSCAN{NC}")
    userOptions()

def userOptions():
    option=input("\nmalscraper> ").upper()

    if option in ["FULL", "FULL-SCAN", "FSCAN"]:
        fullScan()
    elif option in ["QUICK", "QUICK-SCAN", "QSCAN"]:
        quickScan()
    elif option in ["QUIT", "EXIT"]:
        exitApp()
    elif option in ["CLEAR", "CLEAR-HOST", "CLS"]:
        clearMenu()
    elif option in ["HELP", "GET-HELP", "?", "-?", "/?", "MENU"]:
        helpFunc()
    elif option in ["BACK", "CD ..", "HOME"]:
        main()
    elif option == "TUTORIAL":
        tutorial()
    elif option in ["REOPEN", "OPEN"]:
        reopen()
    elif option in ["INSTALL", "UPDATE"]:
        installUpdate()
    else:
        clearMenu() 
        print(f"{RED}{BOLD}Error: {NORMAL}{NC}Invalid Option.")
        helpText()
        userOptions()

def main():
    clearMenu()
    f = Figlet(font='slant')
    print(f.renderText('malScraper.py'))
    print(f"\t{PURPLE}Tool\t :: malScraper{NC}")
    print(f"\t{PURPLE}Author\t :: Ryan Monaghan{NC}")
    print(f"\t{PURPLE}Email\t :: rynmon@pm.me{NC}")
    print(f"\t{PURPLE}GitHub\t :: https://github.com/Ryan-Monaghan/malScraper{NC}")
    print(f"\t{PURPLE}Branch\t :: dev-python{NC}")
    print(f"\t{PURPLE}Version\t :: 0.1{NC}")
    helpText()
    userOptions()

main()