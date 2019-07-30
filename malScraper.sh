#!/usr/bin/env bash
RED='\033[0;31m'
GRN='\033[0;32m'
NC='\033[0m' # No Color
bold=$(tput bold) #variable to format text as bold
normal=$(tput sgr0) #variable to format text as normal
arr[0]="Generating list..."
arr[1]="Scraping data..."
arr[2]="Removing system kernel..."
arr[3]="Erasing Disk..."
arr[4]="Piiiip Beeeep!!"
arr[5]="As seen on TV!"
arr[6]="Whoops! I'm out of memmory!"
arr[7]="Supercalifragilisticexpialidocious!"
arr[8]=$(echo -e "\U0001f602 \U0001F44C \U0001F525 Ayyy dis' sCriPt is litTy \U0001f602 \U0001F44C \U0001F525")
rand=$[$RANDOM % ${#arr[@]}]
echo $(date)
echo ${arr[$rand]}
feed=$(base64 -d <<<"H4sIAHp5QF0AA8soKSkottLXLy3KyUgsLdZLTCotTtVLztBPyS/Py8lPTCnWL0mtKNHnAgCoVL2S
KQAAAA==" | gunzip)
timestamp=$(date +%Y-%m-%d-%H:%M)
clear
mkdir -p /home/$USER/Desktop/malScraper
#printf "Generating list...\n"
echo ${arr[$rand]}
#printf '\xE2\x98\xA0'
#echo -e '\U0001f602'
curl -# $feed >> /home/$USER/Desktop/malScraper/temp1.txt
cd /home/$USER/Desktop/malScraper/
cat temp1.txt | egrep -o "http://([^/]*)/" | sed -e 's/^http:\/\///g' | sed 's/www\./ /g' | sed 's/\/$/ /g' | sed 's/ //g' >> temp2.txt
sort temp1.txt | uniq > PayloadReport.txt
sort temp2.txt | uniq > AMPReport.txt
rm temp*
#printf "\n"
clear
printf "${GRN}${bold}Success - Files written to:\n${normal}${NC}"
printf "${RED}${bold}Payload Domains:${normal}${NC}"
printf "/home/$USER/Desktop/malScraper/PayloadDomains.txt\n"
printf "${RED}${bold}AMP Report:${normal}${NC}"
printf "/home/$USER/Desktop/malScraper/AMPReport.txt\n\n"
exit 1
#testing
#read -n 1 -s -r -p "Press any key to read report..."
#cat PayloadReport.txt
#for domain in $(cat "AMPReport.txt"); do dig +short $domain | tr '\n' ' ' >> Blacklist.csv | geoiplookup $domain | awk 'NR==1{print ", " $5,$6,$7,$8}' >> Blacklist.csv ; done
#logo=$(figlet "eSentire")
#echo $logo.
