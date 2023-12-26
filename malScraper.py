import os
import time

def clearScreen():
    # Clear the screen
    os.system('cls')

def fullScan():
    print("DEBUG")

def quickScan():
    print("DEBUG")

def exitApp():
    print("DEBUG")

def helpFunc():
    print("DEBUG")

def tutorial():
    print("DEBUG")

def reopen():
    print("DEBUG")

def installUpdate():
    print("DEBUG")

"""def DEBUGhelpText():
    print("DEBUG - helpTextMenu")

def DEBUGuserOptions():
    print("\nDEBUG - userOptionsMenu")"""

def helpText():
    print("\nHELP MENU :: Available options shown below:\n")
    print("[*] Tutorial of how to use this tool\t\t\t\t\tTUTORIAL")
    print("[*] Show this Help Menu\t\t\t\t\t\t\tHELP,GET-HELP,?,-?,/?,MENU")
    print("[*] Show options for this tool\t\t\t\t\t\tSHOW OPTIONS,SHOW,OPTIONS")
    print("[*] Clear screen\t\t\t\t\t\t\tCLEAR,CLEAR-HOST,CLS")
    print("[*] Return to Home Menu\t\t\t\t\t\t\tHOME,BACK,CD ..")
    print("[*] Open an existing report\t\t\t\t\t\tOPEN,REOPEN")
    print("[*] Quit malScraper\t\t\t\t\t\t\tQUIT,EXIT")
    print("[*] Install the latest update\t\t\t\t\t\tINSTALL,UPDATE")
    print("[*] Perform Full-Scan (Note this may take some time)\t\t\tFULL,FULL-SCAN,FSCAN")
    print("[*] Perform }Quick-Scan (Most recent 100 Payload Domains)\t\tQUICK,QUICK-SCAN,QSCAN")

def userOptions():
    option=input("malscraper> ").upper()

    if option in ["FULL", "FULL-SCAN", "FSCAN"]:
        fullScan()
    elif option in ["QUICK", "QUICK-SCAN", "QSCAN"]:
        quickScan()
    elif option in ["QUIT", "EXIT"]:
        exitApp()
    elif option in ["CLEAR", "CLEAR-HOST", "CLS"]:
        clearScreen()
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
        clearScreen() 
        print("Error - invalid operation")
        helpText()
        userOptions()

def main():
    clearScreen()
    print ("---MALSCRAPER PYTHON PORT---")
    print("\tTool\t :: malScraper")
    print("\tAuthor\t :: Ryan Monaghan")
    print("\tEmail\t :: rynmon@pm.me")
    print("\tGitHub\t :: https://github.com/Ryan-Monaghan/malScraper")
    print("\tBranch\t :: dev-python")
    print("\tVersion\t :: 0.1")
    helpText()
    userOptions()

main()
    
"""helpText() {
	printf "${Cyan}HELP MENU${NC} ${bold}::${normal} Available ${Yellow}options${NC} shown below:\n\n"
	printf "${bold}[*]${normal} ${Cyan}Tutorial${NC} of how to use this tool\t\t\t\t\t${Yellow}TUTORIAL${NC}\n"
	printf "${bold}[*]${normal} Show this ${Cyan}Help${NC} Menu\t\t\t\t\t\t\t${Yellow}HELP,GET-HELP,?,-?,/?,MENU${NC}\n"
	printf "${bold}[*]${normal} ${Cyan}Show options${NC} for this tool\t\t\t\t\t\t${Yellow}SHOW OPTIONS,SHOW,OPTIONS${NC}\n"
	printf "${bold}[*]${normal} ${Cyan}Clear${NC} screen\t\t\t\t\t\t\t${Yellow}CLEAR,CLEAR-HOST,CLS${NC}\n"
	printf "${bold}[*]${normal} Return to ${Cyan}Home${NC} Menu\t\t\t\t\t\t\t${Yellow}HOME,BACK,CD ..${NC}\n"
	printf "${bold}[*]${normal} ${Cyan}Open${NC} an existing report\t\t\t\t\t\t${Yellow}OPEN,REOPEN${NC}\n"
	printf "${bold}[*]${normal} ${Cyan}Quit${NC} malScraper\t\t\t\t\t\t\t${Yellow}QUIT,EXIT${NC}\n"
	printf "${bold}[*]${normal} ${Cyan}Install${NC} the latest ${Cyan}update${NC}\t\t\t\t\t\t${Yellow}INSTALL,UPDATE${NC}\n"
	printf "${bold}[*]${normal} Perform ${Cyan}Full-Scan${NC} (Note this may take some time)\t\t\t${Yellow}FULL,FULL-SCAN,FSCAN${NC}\n"
	printf "${bold}[*]${normal} Perform ${Cyan}Quick-Scan${NC} (Most recent 100 Payload Domains)\t\t${Yellow}QUICK,QUICK-SCAN,QSCAN${NC}\n\n"
}"""