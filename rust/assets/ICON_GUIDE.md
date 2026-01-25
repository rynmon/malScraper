# malScraper Icon - Quick Conversion Guide

## Current Icon
**icon.svg** - Virus icon (black and cyan) - perfect for malware scraper tool

## Quick Conversion Methods

### Method 1: Online (Easiest)
1. Open `icon.svg` in your browser to preview
2. Use one of these online converters:
   - **ICO**: https://convertio.co/svg-ico/
   - **PNG**: https://cloudconvert.com/svg-to-png
   - **All formats**: https://realfavicongenerator.net/

### Method 2: PowerShell Script (Windows)
```powershell
cd rust/assets
.\create-icons.ps1
```
Requires ImageMagick installed.

### Method 3: Bash Script (Linux/macOS)
```bash
cd rust/assets
chmod +x create-icons.sh
./create-icons.sh
```
Requires Inkscape or ImageMagick.

## Recommended Sizes for Windows
- 256x256 (main)
- 128x128
- 64x64
- 48x48
- 32x32
- 16x16

## Using the Icon

### Windows
- Convert to `.ico` format
- Use in application manifest or resource file
- Or simply rename executable and Windows will use it from the folder

### macOS
- Convert to `.icns` format
- Use with `iconutil` or online converter

### Linux
- Use PNG files at various sizes
- Place in appropriate icon theme directory

## Preview
Open `icon.svg` in any modern browser or image viewer to see the design.

