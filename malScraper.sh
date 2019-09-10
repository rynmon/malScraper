#!/usr/bin/env bash
#global vars

#formatting
RED='\033[0;31m' #red
GRN='\033[0;32m' #green
NC='\033[0m' #no colour 
bold=$(tput bold) #bold
normal=$(tput sgr0) #normal
Purple='\033[1;35m' #Purple
Cyan='\033[1;36m' #Cyan
Yellow='\033[1;33m'

#splash-text arrays
arr[0]=$(echo -e "\U0001F50E Generating list...")
arr[1]=$(echo -e "\U0001F50E Scraping data...")
arr[2]=$(echo -e "\U0001F50E Spinning web...")

#time-on-run
timestamp=$(date +%Y-%m-%d-%H:%M)

#data locations
PayloadReport=/home/$USER/Desktop/malScraper/PayloadReport.txt
AMPReport=/home/$USER/Desktop/malScraper/AMPReport.txt
C2Report=/home/$USER/Desktop/malScraper/C2Report.txt

#pull random token from array
rand=$[$RANDOM % ${#arr[@]}]

#feed locations
PayloadFeed=$(base64 -d <<<"H4sIAHp5QF0AA8soKSkottLXLy3KyUgsLdZLTCotTtVLztBPyS/Py8lPTCnWL0mtKNHnAgCoVL2S
KQAAAA==" | gunzip)
C2Feed=$(base64 -d <<<"H4sIAMwQQ10AA0uuTEotSi7KzE3VLSlKTM5OLdLLSy3RT8zJ0SvIKOACAD03/gcfAAAA" | gunzip)
HexFeed=$(base64 -d <<<"H4sIACgpRF0AA8soKSmw0tcvKUpMzk4t0sswrtBLLdVPLMjUL84sSS2ON8zNzyvJ0CvIKOACAJtI
bs0rAAAA" | gunzip)
#Broken :( OpenPhish=$(base64 -d <<<"H4sIACJFXV0AA8soKSkottLXzy9IzSvIyCzO0EvOz9VPS01N0SupKOECAGyI4nkfAAAA" | gunzip)
PhishTank=$(base64 -d <<<"H4sIAHFCXV0AA8soKSmw0tdPSSxJ1CvIyCzOKEnMy9ZLzs8FC+nn5+Vk5qXqliXmZKboJReXcQEA
oid3DTAAAAA=" | gunzip)
HausMalDown=$(base64 -d <<<"H4sIAJqXXl0AA8soKSkottLXLy3KyUgsLdZLTCotTtVLztBPyS/Py8lPTCnWTy4u0+cCAON9198o
AAAA" | gunzip)

#functions
fullScan() {
	clear
	echo $(date)
	echo ${arr[$rand]}
	if test -f "$PayloadReport"; then
	rm /home/$USER/Desktop/malScraper/PayloadReport.txt
	#echo "Updating existing payload report..."
	fi
	if test -f "$AMPReport"; then
		rm /home/$USER/Desktop/malScraper/AMPReport.txt
		#echo "Updating existing AMP report..."
	fi
	if test -f "$C2Report"; then
		rm /home/$USER/Desktop/malScraper/C2Report.txt
		#echo "Updating existing AMP report..."
	fi
	if test -f "$HexReport"; then
		rm /home/$USER/Desktop/malScraper/Top100.txt
		#echo "Updating existing AMP report..."
	fi
	if test -f "$HexReport"; then
		rm /home/$USER/Desktop/malScraper/HexReport.csv
		#echo "Updating existing AMP report..."
	fi
	if test -f "$PhishTank"; then
		rm /home/$USER/Desktop/malScraper/Phishing/PhishTank.csv
		#echo "Updating existing AMP report..."
	fi
	if test -f "$HausMalDown"; then
		rm /home/$USER/Desktop/malScraper/HausMalDown.csv
		#echo "Updating existing AMP report..."
	fi

	printf "#############################################\n# C2 Servers Report sourced from http://cybercrime-tracker.net/ \n#############################################\n" >> /home/$USER/Desktop/malScraper/C2Report.txt
	#pull reports from feeds
	curl -s $C2Feed >> /home/$USER/Desktop/malScraper/C2Report.txt
	curl -s $HexFeed >> /home/$USER/Desktop/malScraper/HexReport.csv
	curl -s $PayloadFeed >> /home/$USER/Desktop/malScraper/PayloadReport.txt
	curl -# $HausMalDown >> /home/$USER/Desktop/malScraper/HausMalDown.csv
	wget -q -P /home/$USER/Desktop/malScraper/Phishing/ $PhishTank 
	cd /home/$USER/Desktop/malScraper/Phishing
	mv online-valid.csv PhishTank.csv
	cd /home/$USER/Desktop/malScraper/
	#strip domains of their http:// and www. headers for ez amp
	cat PayloadReport.txt | egrep -o "http://([^/]*)/" | sed -e 's/^http:\/\///g' | sed 's/www\./ /g' | sed 's/\/$/ /g' | sed 's/ //g' >> AMPReport.txt
	head -100 /home/$USER/Desktop/malScraper/PayloadReport.txt > /home/$USER/Desktop/malScraper/Top100.txt
	#sort temp1.txt | uniq > PayloadReport.txt
	#sort temp2.txt | uniq > AMPReport.txt
	#rm temp*
	#printf "\n"
	clear
	#presentation
	#figlet malScraper
	printf "${GRN}${bold}Success - Files written to:\n${normal}${NC}"
	printf "${GRN}${bold}1. ${normal}${RED}${bold}Payload Domains:${normal}${NC}"
	printf "/home/$USER/Desktop/malScraper/${bold}PayloadDomains.txt${normal}\n"
	printf "${GRN}${bold}2. ${normal}${RED}${bold}AMP Report:${normal}${NC}"
	printf "/home/$USER/Desktop/malScraper/${bold}AMPReport.txt${normal}\n"
	printf "${GRN}${bold}3. ${normal}${RED}${bold}C2 Servers:${normal}${NC}"
	printf "/home/$USER/Desktop/malScraper/${bold}C2Report.txt${normal}\n"
	printf "${GRN}${bold}4. ${normal}${RED}${bold}Hex Report:${normal}${NC}"
	printf "/home/$USER/Desktop/malScraper/${bold}HexReport.txt${normal}\n"
	printf "${GRN}${bold}5. ${normal}${RED}${bold}URLHaus Maldownloads:${normal}${NC}/home/$USER/Desktop/malScraper/${bold}HausMalDown.csv${normal}\n"
	printf "${GRN}${bold}6. ${normal}${RED}${bold}PhishTank Phishing Pages:${normal}${NC}/home/$USER/Desktop/malScraper/Phishing/${bold}PhishTank.csv${normal}\n"
	printf "${GRN}${bold}7. ${normal}${RED}${bold}Most Recent 100:${normal}${NC}"
	printf "/home/$USER/Desktop/malScraper/${bold}Top100.txt${normal}\n\n"
	#read -n 1 -s -r -p "Press any key to open Malware Domains..."
	#printf "\n"
	#prompt user & open report
	read -p "Which feed would you like to open?" option
	printf "\n"

	if [[ option == "1" ]]
	then
		xdg-open /home/$USER/Desktop/malScraper/PayloadDomains.txt
	elif [[ $option == "2" ]]
	then
		xdg-open /home/$USER/Desktop/malScraper/AMPReport.txt
	elif [[ $option == "3" ]]
	then
		xdg-open /home/$USER/Desktop/malScraper/C2Report.txt
	elif [[ $option == "4" ]]
	then
		xdg-open /home/$USER/Desktop/malScraper/HexReport.txt
	elif [[ $option == "5" ]]
	then
		xdg-open /home/$USER/Desktop/malScraper/HausMalDown.csv
	elif [[ $option == "6" ]]
	then
		xdg-open /home/$USER/Desktop/malScraper/Phishing/PhishTank.csv
	elif [[ $option == "7" ]]
	then
		xdg-open /home/$USER/Desktop/malScraper/Top100.txt
	else
		printf "${RED}${bold}Error: ${normal}${NC}Invalid Option.\n"
	fi
	exit 1
}

quickScan() {
	clear
	echo $(date)
	echo ${arr[$rand]}
	if test -f "$PayloadReport"; then
	rm /home/$USER/Desktop/malScraper/PayloadReport.txt
	#echo "Updating existing payload report..."
	fi
	if test -f "$AMPReport"; then
		rm /home/$USER/Desktop/malScraper/AMPReport.txt
		#echo "Updating existing AMP report..."
	fi
	if test -f "$C2Report"; then
		rm /home/$USER/Desktop/malScraper/C2Report.txt
		#echo "Updating existing AMP report..."
	fi
	if test -f "$HexReport"; then
		rm /home/$USER/Desktop/malScraper/Top100.txt
		#echo "Updating existing AMP report..."
	fi
	if test -f "$HexReport"; then
		rm /home/$USER/Desktop/malScraper/HexReport.csv
		#echo "Updating existing AMP report..."
	fi
	if test -f "$PhishTank"; then
		rm /home/$USER/Desktop/malScraper/Phishing/PhishTank.csv
		#echo "Updating existing AMP report..."
	fi
	if test -f "$HausMalDown"; then
		rm /home/$USER/Desktop/malScraper/HausMalDown.csv
		#echo "Updating existing AMP report..."
	fi

	curl -# $PayloadFeed >> /home/$USER/Desktop/malScraper/PayloadReport.txt
	head -100 /home/$USER/Desktop/malScraper/PayloadReport.txt > /home/$USER/Desktop/malScraper/Top100.txt 
	xdg-open /home/$USER/Desktop/malScraper/Top100.txt

}

exit() {
	#close terminal
	wmctrl -c :ACTIVE:
}

preInstall() {
	#wmctrl -r :ACTIVE: -b add,maximized_vert,maximized_horz
	#check if preRequisates are installed
	figlet=/usr/bin/figlet
	if test -f $figlet
	then
		#echo "Figlet is installed, continuing..."
		echo
	else 
		echo "Figlet not found on system, preparing to install...\n"
		sleep 2
		clear
		sudo apt-get install figlet
	fi
	wmctrl=/usr/bin/wmctrl
	if test -f $wmctrl
	then
		#echo "wmctrl is installed, continuing..."
		echo
	else 
		echo "\nwmctrl not found on system, preparing to install...\n"
		sleep 2
		clear
		sudo apt-get install wmctrl
	fi
	curlInst=/usr/bin/curl
	if test -f $curlInst
	then
		#echo "curl is installed, continuing..."
		echo
	else 
		echo "\ncurl not found on system, preparing to install...\n"
		sleep 2
		clear
		sudo apt-get install curl
	fi
	#echo "Pre-Requisates installed, loading UI!"
	#sleep 2
	#clear
	main
}

clearScreen() {
	printf "Clearing screen..."
	sleep 3
	clear
	main
}

help() {
	printf "This feature is currently unavailable, check back later.\n\nReturning to Home menu..."
	sleep 3
	main
}

main() {
	clear
	#force maximum window
	wmctrl -r :ACTIVE: -b add,maximized_vert,maximized_horz
	figlet malScraper

	printf "\t${Purple}Tool\t :: malScraper\n"
	printf "\tAuthor\t :: Ryan Monaghan\n"
	printf "\tTwitter\t :: @rynmonaghan\n"
	printf "\tGithub\t :: https://github.com/Ryan-Monaghan/malScraper\n"
	printf "\tVersion\t :: 1.0\n\n${NC}"

	printf "${Cyan}HELP MENU${NC} ${bold}::${normal} Available ${Yellow}options${NC} shown below:\n\n"
	
	printf "${bold}[*]${normal} ${Cyan}Tutorial${NC} of how to use this tool\t\t\t\t${Yellow}TUTORIAL${NC}\n"
	printf "${bold}[*]${normal} Show this ${Cyan}Help${NC} Menu\t\t\t\t\t\t${Yellow}HELP,GET-HELP,?,-?,/?,MENU${NC}\n"
	printf "${bold}[*]${normal} ${Cyan}Show options${NC} for this tool\t\t\t\t\t${Yellow}SHOW OPTIONS,SHOW,OPTIONS${NC}\n"
	printf "${bold}[*]${normal} ${Cyan}Clear${NC} screen\t\t\t\t\t\t${Yellow}CLEAR,CLEAR-HOST,CLS${NC}\n"
	printf "${bold}[*]${normal} Go ${Cyan}Back${NC} to previous menu\t\t\t\t\t${Yellow}BACK,CD ..${NC}\n"
	printf "${bold}[*]${normal} ${Cyan}Quit${NC} malScraper\t\t\t\t\t\t${Yellow}QUIT,EXIT${NC}\n"
	printf "${bold}[*]${normal} Perform ${Cyan}Full-Scan${NC} (Note this may take some time)\t\t${Yellow}FULL,FULL-SCAN,FSCAN${NC}\n"
	printf "${bold}[*]${normal} Perform ${Cyan}Quick-Scan${NC} Most recent 100 Payload Domains\t\t${Yellow}QUICK,QUICK-SCAN,QSCAN${NC}\n\n"

	read -p "malScraper>" option #store user input

	option=${option^^} #force user input to uppercase

	#track user input
	if [[ $option == "FULL" ]] || [[ $option == "FULL-SCAN" ]] || [[ $option == "FSCAN" ]]
	then
		fullScan
	elif [[ $option == "QUICK" ]] || [[ $option == "QUICK-SCAN" ]] || [[ $option == "QSCAN" ]]
		then
			quickScan
	elif [[ $option == "QUIT" ]] || [[ $option == "EXIT" ]]
		then
			exit
	elif [[ $option == "CLEAR" ]] || [[ $option == "CLEAR-HOST" ]] || [[ $option == "CLS" ]]
		then
			clearScreen
	elif [[ $option == "HELP" ]] || [[ $option == "GET-HELP" ]] || [[ $option == "?" ]] || [[ $option == "-?" ]] || [[ $option == "/?" ]] || [[ $option == "MENU" ]]
		then
			help
	else
		printf "${RED}${bold}Error - ${normal}${NC}invalid operation\n"
	fi
}

preInstall

#testing
#read -n 1 -s -r -p "Press any key to read report..."
#cat PayloadReport.txt
#was going to pull IP Addressed associated with domains, but made script runtime waaaay too long
#due to number of IPs that were being parsed through geoiplookup :(
#for domain in $(cat "AMPReport.txt"); do dig +short $domain | tr '\n' ' ' >> Blacklist.csv | geoiplookup $domain | awk 'NR==1{print ", " $5,$6,$7,$8}' >> Blacklist.csv ; done
