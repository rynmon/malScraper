# malScraper Icon Assets

## Icon Design

**icon.svg** - Virus icon (black and cyan) - Perfect for a malware scraper tool!

The icon features a stylized virus design with a central circle and radiating spikes, using black and cyan (#2ca9bc) colors. This design is ideal for a security tool that deals with malware indicators.

### Alternative Icons
- **icon-alt1.svg** - Shield with "M" letter (green gradient)
- **icon-alt2.svg** - Network/web pattern (purple gradient)
- **icon-alt3.svg** - Magnifying glass with "M" (dark blue/gray)

## Converting to Icon Formats

### Windows (.ico)

Using ImageMagick:
```bash
magick convert icon.svg -resize 256x256 icon.ico
```

Or using online tools:
- https://convertio.co/svg-ico/
- https://cloudconvert.com/svg-to-ico

### macOS (.icns)

1. Create multiple sizes (16x16, 32x32, 64x64, 128x128, 256x256, 512x512, 1024x1024)
2. Use `iconutil`:
```bash
mkdir icon.iconset
# Copy PNG files at different sizes
iconutil -c icns icon.iconset
```

### Linux (.png)

```bash
# Using Inkscape
inkscape icon.svg --export-width=512 --export-filename=icon-512.png
inkscape icon.svg --export-width=256 --export-filename=icon-256.png
inkscape icon.svg --export-width=128 --export-filename=icon-128.png
inkscape icon.svg --export-width=64 --export-filename=icon-64.png
inkscape icon.svg --export-width=48 --export-filename=icon-48.png
inkscape icon.svg --export-width=32 --export-filename=icon-32.png
```

## Alternative: Simple Text-Based Icon

If you prefer a simpler approach, we could create a text-based icon using the "malScraper" text in a stylized font, or use a simple shield/magnifying glass combination.

## Online Icon Generators

- **Favicon.io**: https://favicon.io/ (text-based icons)
- **RealFaviconGenerator**: https://realfavicongenerator.net/
- **IconKitchen**: https://icon.kitchen/ (Android/iOS)

## Recommended Sizes

- **Windows**: 256x256, 128x128, 64x64, 48x48, 32x32, 16x16
- **macOS**: 1024x1024, 512x512, 256x256, 128x128, 64x64, 32x32, 16x16
- **Linux**: 512x512, 256x256, 128x128, 64x64, 48x48, 32x32

