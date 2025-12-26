# PowerShell script to convert SVG to ICO for Windows
# Requires: ImageMagick or online conversion

$svgFile = "icon.svg"
$mainIcon = "icon.svg"
$outputDir = "icons"

if (-not (Test-Path $outputDir)) {
    New-Item -ItemType Directory -Path $outputDir
}

# Check for ImageMagick
if (Get-Command magick -ErrorAction SilentlyContinue) {
    Write-Host "Using ImageMagick..." -ForegroundColor Green
    
    # Create PNG files at various sizes
    $sizes = @(16, 32, 48, 64, 128, 256, 512, 1024)
    foreach ($size in $sizes) {
        magick convert -background none -resize "${size}x${size}" $svgFile "$outputDir\icon-${size}x${size}.png"
    }
    
    # Create ICO file
    magick convert "$outputDir\icon-256x256.png" "$outputDir\icon-128x128.png" "$outputDir\icon-64x64.png" "$outputDir\icon-32x32.png" "$outputDir\icon-16x16.png" "$outputDir\icon.ico"
    
    Write-Host "Icons created in $outputDir\" -ForegroundColor Green
} else {
    Write-Host "ImageMagick not found. Please install it or use an online converter:" -ForegroundColor Yellow
    Write-Host "  - Download: https://imagemagick.org/script/download.php" -ForegroundColor Cyan
    Write-Host "  - Or use: https://convertio.co/svg-ico/" -ForegroundColor Cyan
}

