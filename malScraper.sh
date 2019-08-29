#!/usr/bin/env bash
RED='\033[0;31m' #red
GRN='\033[0;32m' #green
NC='\033[0m' # No Color
bold=$(tput bold) #variable to format text as bold
normal=$(tput sgr0) #variable to format text as normal
#create array to display splash text
arr[0]=$(echo -e "\U0001F50E Generating list...")
arr[1]=$(echo -e "\U0001F50E Scraping data...")
arr[2]=$(echo -e "\U0001F50E Spinning web...")
timestamp=$(date +%Y-%m-%d-%H:%M)
#variables for data locations
PayloadReport=/home/$USER/Desktop/malScraper/PayloadReport.txt
AMPReport=/home/$USER/Desktop/malScraper/AMPReport.txt
C2Report=/home/$USER/Desktop/malScraper/C2Report.txt
#variavle to pull random value from array
rand=$[$RANDOM % ${#arr[@]}]
echo $(date)
echo ${arr[$rand]}
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
clear
mkdir -p /home/$USER/Desktop/malScraper/Phishing
echo ${arr[$rand]}

#check for presence of existing reports, if found, delete them
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

#sleep 5

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
printf "${GRN}${bold}Success - Files written to:\n${normal}${NC}"
printf "${RED}${bold}Payload Domains:${normal}${NC}"
printf "/home/$USER/Desktop/malScraper/${bold}PayloadDomains.txt${normal}\n"
printf "${RED}${bold}AMP Report:${normal}${NC}"
printf "/home/$USER/Desktop/malScraper/${bold}AMPReport.txt${normal}\n"
printf "${RED}${bold}C2 Servers:${normal}${NC}"
printf "/home/$USER/Desktop/malScraper/${bold}C2Report.txt${normal}\n"
printf "${RED}${bold}Hex Report:${normal}${NC}"
printf "/home/$USER/Desktop/malScraper/${bold}HexReport.txt${normal}\n"
printf "${RED}${bold}PhishTank Phishing Pages:${normal}${NC}/home/$USER/Desktop/malScraper/${bold}HausMalDown.csv${normal}\n"
printf "${RED}${bold}PhishTank Phishing Pages:${normal}${NC}/home/$USER/Desktop/malScraper/Phishing/${bold}PhishTank.csv${normal}\n"
printf "${RED}${bold}Most Recent 100:${normal}${NC}"
printf "/home/$USER/Desktop/malScraper/${bold}Top100.txt${normal}\n\n"
read -n 1 -s -r -p "Press any key to open Malware Domains..."
printf "\n"
#prompt user & open report
xdg-open /home/$USER/Desktop/malScraper/HausMalDown.csv
exit 1

#testing
#read -n 1 -s -r -p "Press any key to read report..."
#cat PayloadReport.txt
#was going to pull IP Addressed associated with domains, but made script runtime waaaay tp long
#due to number of IPs that were being parsed through geoiplookup :(
#for domain in $(cat "AMPReport.txt"); do dig +short $domain | tr '\n' ' ' >> Blacklist.csv | geoiplookup $domain | awk 'NR==1{print ", " $5,$6,$7,$8}' >> Blacklist.csv ; done
#logo=$(figlet "eSentire")
#echo $logo.
