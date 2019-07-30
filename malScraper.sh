#!/usr/bin/env bash
RED='\033[0;31m'
GRN='\033[0;32m'
NC='\033[0m' # No Color
bold=$(tput bold) #variable to format text as bold
normal=$(tput sgr0) #variable to format text as normal
#feed='https://urlhaus.abuse.ch/downloads/text/'
feed=$(base64 -d <<<"H4sIAHp5QF0AA8soKSkottLXLy3KyUgsLdZLTCotTtVLztBPyS/Py8lPTCnWL0mtKNHnAgCoVL2S
KQAAAA==" | gunzip)
echo $feed
timestamp=$(date +%Y-%m-%d-%H:%M)
clear
mkdir -p /home/$USER/Desktop/malScraper
curl -# $feed >> /home/$USER/Desktop/malScraper/PayloadDomains.txt
#clear
cd /home/$USER/Desktop/malScraper/
cat PayloadDomains.txt | egrep -o "http://([^/]*)/" | sed -e 's/^http:\/\///g' | sed 's/www\./ /g' | sed 's/\/$/ /g' | sed 's/ //g' >> temp.txt
sort temp.txt | uniq > AMPReport.txt
rm temp.txt
printf "${GRN}${bold}Success - Files written to:\n${normal}${NC}"
printf "/home/$USER/Desktop/malScraper/PayloadDomains.txt \n"
printf "/home/$USER/Desktop/malScraper/AMPReport.txt \n"
#testing
#logo=$(figlet "eSentire")
#echo $logo