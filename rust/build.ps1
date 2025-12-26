# Build script for malScraper (Windows)
# Usage: .\build.ps1

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  malScraper Build Script (Windows)" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Check if cargo is available
try {
    $cargoVersion = cargo --version 2>&1
    Write-Host "Found Cargo: $cargoVersion" -ForegroundColor Green
} catch {
    Write-Host "ERROR: Cargo not found!" -ForegroundColor Red
    Write-Host ""
    Write-Host "Please install Rust first:" -ForegroundColor Yellow
    Write-Host "  1. Visit https://rustup.rs/" -ForegroundColor Yellow
    Write-Host "  2. Download and run rustup-init.exe" -ForegroundColor Yellow
    Write-Host "  3. Restart your terminal and try again" -ForegroundColor Yellow
    exit 1
}

Write-Host ""
Write-Host "Building malScraper in release mode..." -ForegroundColor Cyan
Write-Host ""

# Build the project
cargo build --release

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "========================================" -ForegroundColor Green
    Write-Host "  Build Successful!" -ForegroundColor Green
    Write-Host "========================================" -ForegroundColor Green
    Write-Host ""
    
    $exePath = "target\release\malscraper.exe"
    if (Test-Path $exePath) {
        $fileInfo = Get-Item $exePath
        $fileSize = [math]::Round($fileInfo.Length / 1MB, 2)
        
        Write-Host "Executable: $exePath" -ForegroundColor Yellow
        Write-Host "Size: $fileSize MB" -ForegroundColor Yellow
        Write-Host ""
        Write-Host "To run:" -ForegroundColor Cyan
        Write-Host "  .\$exePath" -ForegroundColor White
    }
} else {
    Write-Host ""
    Write-Host "========================================" -ForegroundColor Red
    Write-Host "  Build Failed!" -ForegroundColor Red
    Write-Host "========================================" -ForegroundColor Red
    Write-Host ""
    Write-Host "Check the error messages above for details." -ForegroundColor Yellow
    exit 1
}

