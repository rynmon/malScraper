#!/usr/bin/env bash
RED='\033[0;31m'
GRN='\033[0;32m'
NC='\033[0m' # No Color
bold=$(tput bold) #variable to format text as bold
normal=$(tput sgr0) #variable to format text as normal
arr[0]=$(echo -e "\U0001F50E Generating list...")
arr[1]=$(echo -e "\U0001F50E Scraping data...")
arr[2]=$(echo -e "\U0001F50E Spinning web...")
timestamp=$(date +%Y-%m-%d-%H:%M)
PayloadReport=/home/$USER/Desktop/malScraper/PayloadReport.txt
AMPReport=/home/$USER/Desktop/malScraper/AMPReport.txt
C2Report=/home/$USER/Desktop/malScraper/C2Report.txt
rand=$[$RANDOM % ${#arr[@]}]
echo $(date)
echo ${arr[$rand]}
PayloadFeed=$(base64 -d <<<"H4sIAHp5QF0AA8soKSkottLXLy3KyUgsLdZLTCotTtVLztBPyS/Py8lPTCnWL0mtKNHnAgCoVL2S
KQAAAA==" | gunzip)
C2Feed=$(base64 -d <<<"H4sIAMwQQ10AA0uuTEotSi7KzE3VLSlKTM5OLdLLSy3RT8zJ0SvIKOACAD03/gcfAAAA" | gunzip)
HexFeed=$(base64 -d <<<"H4sIACgpRF0AA8soKSmw0tcvKUpMzk4t0sswrtBLLdVPLMjUL84sSS2ON8zNzyvJ0CvIKOACAJtI
bs0rAAAA" | gunzip)
clear
mkdir -p /home/$USER/Desktop/malScraper
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
	rm /home/$USER/Desktop/malScraper/HexReport.csv
	#echo "Updating existing AMP report..."
fi

#sleep 5

printf "#############################################\n C2 Servers Report sourced from http://cybercrime-tracker.net/ \n#############################################\n" >> /home/$USER/Desktop/malScraper/C2Report.txt
curl -# $C2Feed >> /home/$USER/Desktop/malScraper/C2Report.txt
curl -# $HexFeed >> /home/$USER/Desktop/malScraper/HexReport.csv
curl -# $PayloadFeed >> /home/$USER/Desktop/malScraper/PayloadReport.txt
cd /home/$USER/Desktop/malScraper/
cat PayloadReport.txt | egrep -o "http://([^/]*)/" | sed -e 's/^http:\/\///g' | sed 's/www\./ /g' | sed 's/\/$/ /g' | sed 's/ //g' >> AMPReport.txt
head -100 /home/$USER/Desktop/malScraper/PayloadReport.txt > /home/$USER/Desktop/malScraper/Top100.txt
#sort temp1.txt | uniq > PayloadReport.txt
#sort temp2.txt | uniq > AMPReport.txt
#rm temp*
#printf "\n"
clear
printf "${GRN}${bold}Success - Files written to:\n${normal}${NC}"
printf "${RED}${bold}Payload Domains:${normal}${NC}"
printf "/home/$USER/Desktop/malScraper/${bold}PayloadDomains.txt${normal}\n"
printf "${RED}${bold}AMP Report:${normal}${NC}"
printf "/home/$USER/Desktop/malScraper/${bold}AMPReport.txt${normal}\n"
printf "${RED}${bold}C2 Servers:${normal}${NC}"
printf "/home/$USER/Desktop/malScraper/${bold}C2Report.txt${normal}\n"
printf "${RED}${bold}Hex Report:${normal}${NC}"
printf "/home/$USER/Desktop/malScraper/${bold}HexReport.txt${normal}\n"
printf "${RED}${bold}Most Recent 100:${normal}${NC}"
printf "/home/$USER/Desktop/malScraper/${bold}Top100.txt${normal}\n\n"
read -n 1 -s -r -p "Press any key to open HexReport..."
printf "\n"
xdg-open /home/$USER/Desktop/malScraper/HexReport.csv
exit 1
#testing
#read -n 1 -s -r -p "Press any key to read report..."
#cat PayloadReport.txt
#for domain in $(cat "AMPReport.txt"); do dig +short $domain | tr '\n' ' ' >> Blacklist.csv | geoiplookup $domain | awk 'NR==1{print ", " $5,$6,$7,$8}' >> Blacklist.csv ; done
#logo=$(figlet "eSentire")
#echo $logo.
