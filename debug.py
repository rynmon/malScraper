import os
import time
import sys
import random
import ctypes
import base64
import zlib
import requests
from pyfiglet import Figlet
from colorama import Fore, Style
from datetime import datetime

# Define the list of download sources using the variables
#encoded sources
encPayloadFeed="H4sIAHp5QF0AA8soKSkottLXLy3KyUgsLdZLTCotTtVLztBPyS/Py8lPTCnWL0mtKNHnAgCoVL2SKQAAAA=="
encC2Feed="H4sIAMwQQ10AA0uuTEotSi7KzE3VLSlKTM5OLdLLSy3RT8zJ0SvIKOACAD03/gcfAAAA"
encHexFeed="H4sIACgpRF0AA8soKSmw0tcvKUpMzk4t0sswrtBLLdVPLMjUL84sSS2ON8zNzyvJ0CvIKOACAJtIbs0rAAAA"
encPhishTank="H4sIAHFCXV0AA8soKSmw0tdPSSxJ1CvIyCzOKEnMy9ZLzs8FC+nn5+Vk5qXqliXmZKboJReXcQEAoid3DTAAAAA="
encHausMalDown="H4sIAJqXXl0AA8soKSkottLXLy3KyUgsLdZLTCotTtVLztBPyS/Py8lPTCnWTy4u0+cCAON9198oAAAA"

PayloadReport="C:\\temp\\malScraper\\PayloadReport.txt"
AMPReport="C:\\temp\\malScraper\\AMPReport.txt"
C2Report="C:\\temp\\malScraper\\C2Report.txt"
Top100="C:\\temp\\malScraper\\Top100.txt"
HexReport="C:\\temp\\malScraper\\HexReport.csv"
HausMalDown="C:\\temp\\malScraper\\HausMalDown.csv"
PhishTank="C:\\temp\\malScraper\\PhishTank.csv"

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

download_sources = [
    (PayloadFeed, PayloadReport)
]

try:
    for url, local_path in download_sources:
        # Perform a GET request to download the file
        response = requests.get(url)
        
        # Check if the request was successful (status code 200)
        if response.status_code == 200:
            # Save the content to the local file
            with open(local_path, 'wb') as file:
                file.write(response.content)
            print(f"File downloaded successfully to '{local_path}'.")
        else:
            print(f"Failed to download the file '{url}'. Status code: {response.status_code}")

except Exception as e:
    print(f"Error: {e}")