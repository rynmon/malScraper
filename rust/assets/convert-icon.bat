@echo off
REM Simple batch script to convert SVG to ICO using online converter
REM This opens the SVG file and provides instructions for conversion

echo.
echo ========================================
echo malScraper Icon Converter
echo ========================================
echo.
echo The icon.svg file has been updated with the new virus icon.
echo.
echo To convert to ICO format, you have several options:
echo.
echo Option 1: Online Converter (Easiest)
echo   1. Open https://convertio.co/svg-ico/ in your browser
echo   2. Upload icon.svg
echo   3. Download the converted icon.ico
echo   4. Place icon.ico in this directory (rust/assets/)
echo.
echo Option 2: Python Script
echo   1. Install: pip install pillow cairosvg
echo   2. Run: python convert-icon.py
echo.
echo Option 3: ImageMagick
echo   1. Install ImageMagick from https://imagemagick.org/
echo   2. Run: .\create-icons.ps1
echo.
echo ========================================
echo.
echo Opening icon.svg in default browser for preview...
start icon.svg
echo.
pause
