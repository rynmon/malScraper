# Building malScraper on Windows

## Step 1: Install Rust

1. **Download and run the Rust installer:**
   - Visit: https://rustup.rs/
   - Download `rustup-init.exe`
   - Run the installer and follow the prompts
   - Choose the default installation (recommended)

2. **Restart your terminal/PowerShell** after installation completes

3. **Verify installation:**
   ```powershell
   rustc --version
   cargo --version
   ```

## Step 2: Build the Project

1. **Navigate to the rust directory:**
   ```powershell
   cd rust
   ```

2. **Build in release mode (optimized):**
   ```powershell
   cargo build --release
   ```

   This will:
   - Download all dependencies (first time only)
   - Compile the project
   - Create an optimized executable

3. **The executable will be located at:**
   ```
   rust\target\release\malscraper.exe
   ```

## Step 3: Run the Executable

You can run it directly:
```powershell
.\target\release\malscraper.exe
```

Or copy it to a more convenient location:
```powershell
# Copy to current directory
Copy-Item .\target\release\malscraper.exe .\

# Or add to PATH for global access
```

## Quick Build Script

You can also create a simple build script. Save this as `build.ps1` in the `rust` directory:

```powershell
# build.ps1
Write-Host "Building malScraper..." -ForegroundColor Cyan
cargo build --release

if ($LASTEXITCODE -eq 0) {
    Write-Host "`nBuild successful!" -ForegroundColor Green
    Write-Host "Executable: target\release\malscraper.exe" -ForegroundColor Yellow
} else {
    Write-Host "`nBuild failed!" -ForegroundColor Red
    exit 1
}
```

Then run:
```powershell
.\build.ps1
```

## Troubleshooting

### "cargo is not recognized"
- Make sure Rust is installed and you've restarted your terminal
- Check that `C:\Users\YourUsername\.cargo\bin` is in your PATH

### Build errors
- Make sure you're in the `rust` directory
- Check that you have an internet connection (first build downloads dependencies)
- Try running `cargo clean` and then `cargo build --release` again

### Antivirus warnings
- Some antivirus software may flag the executable during compilation
- This is a false positive - the tool is for security research
- You may need to add an exception for the `target` directory

## Build Time

- First build: ~5-10 minutes (downloads dependencies)
- Subsequent builds: ~1-2 minutes
- Clean rebuild: ~2-3 minutes

## File Size

The resulting `malscraper.exe` will be approximately:
- **5-10 MB** (release build with optimizations)
- Much smaller than Python + all dependencies!

