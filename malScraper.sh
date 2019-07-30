#!/usr/bin/env bash
RED='\033[0;31m'
GRN='\033[0;32m'
NC='\033[0m' # No Color
bold=$(tput bold) #variable to format text as bold
normal=$(tput sgr0) #variable to format text as normal
feed=$(base64 -d <<<"H4sIAHp5QF0AA8soKSkottLXLy3KyUgsLdZLTCotTtVLztBPyS/Py8lPTCnWL0mtKNHnAgCoVL2S
KQAAAA==" | gunzip)
timestamp=$(date +%Y-%m-%d-%H:%M)
clear
mkdir -p /home/$USER/Desktop/malScraper
curl -# $feed >> /home/$USER/Desktop/malScraper/temp1.txt
#clear
cd /home/$USER/Desktop/malScraper/
cat temp1.txt | egrep -o "http://([^/]*)/" | sed -e 's/^http:\/\///g' | sed 's/www\./ /g' | sed 's/\/$/ /g' | sed 's/ //g' >> temp2.txt
sort temp1.txt | uniq > PayloadReport.txt
sort temp2.txt | uniq > AMPReport.txt
rm temp*
printf "${GRN}${bold}Success - Files written to:\n${normal}${NC}"
printf "/home/$USER/Desktop/malScraper/PayloadDomains.txt \n"
printf "/home/$USER/Desktop/malScraper/AMPReport.txt \n"
#testing
#for domain in $(cat "AMPReport.txt"); do dig +short $domain | tr '\n' ' ' >> Blacklist.csv | geoiplookup $domain | awk 'NR==1{print ", " $5,$6,$7,$8}' >> Blacklist.csv ; done
#logo=$(figlet "eSentire")
#echo $logo
