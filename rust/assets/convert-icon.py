#!/usr/bin/env python3
"""
Convert SVG to ICO and PNG formats for malScraper
Requires: pip install pillow cairosvg
"""

import sys
import os
from pathlib import Path

try:
    from PIL import Image
    import cairosvg
except ImportError:
    print("Error: Required packages not installed.")
    print("Install with: pip install pillow cairosvg")
    sys.exit(1)

def svg_to_png(svg_path, png_path, size):
    """Convert SVG to PNG at specified size"""
    png_data = cairosvg.svg2png(url=svg_path, output_width=size, output_height=size)
    with open(png_path, 'wb') as f:
        f.write(png_data)

def create_ico(svg_path, ico_path):
    """Create ICO file from SVG with multiple sizes"""
    sizes = [16, 32, 48, 64, 128, 256]
    images = []
    
    for size in sizes:
        png_data = cairosvg.svg2png(url=svg_path, output_width=size, output_height=size)
        img = Image.open(io.BytesIO(png_data))
        images.append(img)
    
    # Save as ICO with multiple sizes
    images[0].save(ico_path, format='ICO', sizes=[(s, s) for s in sizes])

def main():
    import io
    
    svg_file = "icon.svg"
    ico_file = "icon.ico"
    output_dir = Path("icons")
    
    if not os.path.exists(svg_file):
        print(f"Error: {svg_file} not found")
        sys.exit(1)
    
    output_dir.mkdir(exist_ok=True)
    
    print("Converting SVG to ICO and PNG formats...")
    
    # Create ICO file
    try:
        create_ico(svg_file, ico_file)
        print(f"✓ Created {ico_file}")
    except Exception as e:
        print(f"Error creating ICO: {e}")
        sys.exit(1)
    
    # Create PNG files at various sizes
    sizes = [16, 32, 48, 64, 128, 256, 512, 1024]
    for size in sizes:
        png_file = output_dir / f"icon-{size}x{size}.png"
        try:
            svg_to_png(svg_file, str(png_file), size)
            print(f"✓ Created {png_file}")
        except Exception as e:
            print(f"Error creating {png_file}: {e}")
    
    print("\nConversion complete!")
    print(f"ICO file: {ico_file}")
    print(f"PNG files: {output_dir}/")

if __name__ == "__main__":
    main()
