# Embedding Icon in malScraper Executable

## Setup Complete ✅

The build system is configured to automatically embed the icon when building for Windows.

## What You Need

1. **icon.ico file** - Place your converted icon file here:
   ```
   rust/assets/icon.ico
   ```

## How It Works

- The `build.rs` script automatically detects the icon file
- On Windows builds, it embeds the icon into the `.exe`
- The icon will appear in:
  - File Explorer
  - Task Manager
  - Application shortcuts
  - Alt+Tab switcher

## Building with Icon

Just build normally:
```bash
cargo build --release
```

The icon will be automatically embedded if `icon.ico` exists in `rust/assets/`.

## Verify Icon is Embedded

After building:
1. Navigate to `rust/target/release/`
2. Right-click `malscraper.exe`
3. Select "Properties"
4. Check the icon in the Properties dialog

## Troubleshooting

### Icon not showing?
- Make sure `icon.ico` is in `rust/assets/` directory
- Rebuild: `cargo clean && cargo build --release`
- Check that `icon.ico` is a valid ICO file

### Build errors?
- Make sure you have Visual Studio Build Tools installed (for Windows)
- The `embed-resource` crate requires the Windows Resource Compiler

## Alternative: Manual Embedding

If automatic embedding doesn't work, you can manually embed using:
- Resource Hacker (Windows)
- Or add to a `.res` file and link manually

