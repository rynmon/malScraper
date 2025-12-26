#!/bin/bash
# Script to convert SVG icon to various formats
# Requires: Inkscape or ImageMagick

SVG_FILE="icon.svg"
MAIN_ICON="icon.svg"
OUTPUT_DIR="icons"

mkdir -p "$OUTPUT_DIR"

# Check for Inkscape
if command -v inkscape &> /dev/null; then
    echo "Using Inkscape..."
    
    # Create PNG files at various sizes
    for size in 16 32 48 64 128 256 512 1024; do
        inkscape "$SVG_FILE" --export-width=$size --export-filename="$OUTPUT_DIR/icon-${size}x${size}.png"
    done
    
    # Create ICO for Windows
    convert "$OUTPUT_DIR/icon-256x256.png" "$OUTPUT_DIR/icon-128x128.png" "$OUTPUT_DIR/icon-64x64.png" "$OUTPUT_DIR/icon-32x32.png" "$OUTPUT_DIR/icon-16x16.png" "$OUTPUT_DIR/icon.ico"
    
    echo "Icons created in $OUTPUT_DIR/"
    
# Check for ImageMagick
elif command -v convert &> /dev/null; then
    echo "Using ImageMagick..."
    
    for size in 16 32 48 64 128 256 512 1024; do
        convert -background none -resize "${size}x${size}" "$SVG_FILE" "$OUTPUT_DIR/icon-${size}x${size}.png"
    done
    
    convert "$OUTPUT_DIR/icon-256x256.png" "$OUTPUT_DIR/icon-128x128.png" "$OUTPUT_DIR/icon-64x64.png" "$OUTPUT_DIR/icon-32x32.png" "$OUTPUT_DIR/icon-16x16.png" "$OUTPUT_DIR/icon.ico"
    
    echo "Icons created in $OUTPUT_DIR/"
else
    echo "Error: Neither Inkscape nor ImageMagick found."
    echo "Please install one of them:"
    echo "  - Inkscape: https://inkscape.org/"
    echo "  - ImageMagick: https://imagemagick.org/"
    exit 1
fi

