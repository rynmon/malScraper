#imports
import os
import time
import sys
import random
import ctypes
import base64
import zlib
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

#data locations
#stores the default location of the download path of the various feeds - these can be modified to custom locations
PayloadReport="C:\\temp\malScraper\PayloadReport.txt"
AMPReport="C:\\temp\malScraper\AMPReport.txt"
C2Report="C:\\temp\malScraper\C2Report.txt"
Top100="C:\\temp\malScraper\Top100.txt"
HexReport="C:\\temp\malScraper\HexReport.csv"
HausMalDown="C:\\temp\malScraper\HausMalDown.csv"
PhishTank="C:\\temp\malScraper\PhishTank.csv"

#encoded sources
encPayloadFeed="H4sIAHp5QF0AA8soKSkottLXLy3KyUgsLdZLTCotTtVLztBPyS/Py8lPTCnWL0mtKNHnAgCoVL2SKQAAAA=="
encC2Feed="H4sIAMwQQ10AA0uuTEotSi7KzE3VLSlKTM5OLdLLSy3RT8zJ0SvIKOACAD03/gcfAAAA"
encHexFeed="H4sIACgpRF0AA8soKSmw0tcvKUpMzk4t0sswrtBLLdVPLMjUL84sSS2ON8zNzyvJ0CvIKOACAJtIbs0rAAAA"
encPhishTank="H4sIAHFCXV0AA8soKSmw0tdPSSxJ1CvIyCzOKEnMy9ZLzs8FC+nn5+Vk5qXqliXmZKboJReXcQEAoid3DTAAAAA="
encHausMalDown="H4sIAJqXXl0AA8soKSkottLXLy3KyUgsLdZLTCotTtVLztBPyS/Py8lPTCnWTy4u0+cCAON9198oAAAA"

#decode sources
PayloadFeed=zlib.decompress(base64.b64decode(encPayloadFeed), 16+zlib.MAX_WBITS).decode('utf-8')
#debug print(PayloadFeed)
C2Feed=zlib.decompress(base64.b64decode(encC2Feed), 16+zlib.MAX_WBITS).decode('utf-8')
#debug print(C2Feed)
HexFeed=zlib.decompress(base64.b64decode(encHexFeed), 16+zlib.MAX_WBITS).decode('utf-8')
#debug print(HexFeed)
PhishTank=zlib.decompress(base64.b64decode(encPhishTank), 16+zlib.MAX_WBITS).decode('utf-8')
#debug print(PhishTank)
HausMalDown=zlib.decompress(base64.b64decode(encHausMalDown), 16+zlib.MAX_WBITS).decode('utf-8')
#debug print(HausMalDown)

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