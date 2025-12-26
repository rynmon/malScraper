# PowerShell script to help install Visual Studio Build Tools
# This script opens the download page and provides instructions

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Visual Studio Build Tools Setup" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

Write-Host "Rust needs the Microsoft Visual C++ linker to compile on Windows." -ForegroundColor Yellow
Write-Host ""

Write-Host "Option 1: Install Visual Studio Build Tools (Recommended)" -ForegroundColor Green
Write-Host "  1. Download from: https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022" -ForegroundColor White
Write-Host "  2. Run the installer" -ForegroundColor White
Write-Host "  3. Select 'Desktop development with C++' workload" -ForegroundColor White
Write-Host "  4. Make sure 'MSVC v143 - VS 2022 C++ x64/x86 build tools' is checked" -ForegroundColor White
Write-Host "  5. Click Install" -ForegroundColor White
Write-Host "  6. Restart your terminal and run: cargo build --release" -ForegroundColor White
Write-Host ""

Write-Host "Option 2: Use GNU Toolchain (Alternative)" -ForegroundColor Green
Write-Host "  If you prefer not to install Visual Studio:" -ForegroundColor White
Write-Host "  1. Install MinGW-w64 or MSYS2" -ForegroundColor White
Write-Host "  2. Run: rustup toolchain install stable-x86_64-pc-windows-gnu" -ForegroundColor White
Write-Host "  3. Run: rustup default stable-x86_64-pc-windows-gnu" -ForegroundColor White
Write-Host ""

$response = Read-Host "Would you like to open the Visual Studio Build Tools download page? (Y/N)"

if ($response -eq 'Y' -or $response -eq 'y') {
    Start-Process "https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022"
    Write-Host ""
    Write-Host "Download page opened in your browser." -ForegroundColor Green
    Write-Host "After installation, restart your terminal and run: cargo build --release" -ForegroundColor Yellow
} else {
    Write-Host ""
    Write-Host "You can manually visit: https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022" -ForegroundColor Cyan
}

Write-Host ""

