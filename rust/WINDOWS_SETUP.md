# Windows Build Tools Setup

## The Problem
Rust on Windows needs a C++ linker. You're seeing this error because `link.exe` (the MSVC linker) is not found.

## Solution Options

### Option 1: Install Visual Studio Build Tools (Recommended)

1. **Download Visual Studio Build Tools:**
   - Visit: https://visualstudio.microsoft.com/downloads/#build-tools-for-visual-studio-2022
   - Download "Build Tools for Visual Studio 2022"

2. **Install with C++ workload:**
   - Run the installer
   - Select "Desktop development with C++" workload
   - Make sure "MSVC v143 - VS 2022 C++ x64/x86 build tools" is checked
   - Click "Install"

3. **Restart your terminal** and try building again:
   ```powershell
   cargo build --release
   ```

### Option 2: Use GNU Toolchain (Alternative)

If you prefer not to install Visual Studio, you can use the GNU toolchain instead:

1. **Install MinGW-w64:**
   - Download from: https://www.mingw-w64.org/downloads/
   - Or use MSYS2: https://www.msys2.org/

2. **Configure Rust to use GNU toolchain:**
   ```powershell
   rustup toolchain install stable-x86_64-pc-windows-gnu
   rustup default stable-x86_64-pc-windows-gnu
   ```

3. **Build again:**
   ```powershell
   cargo build --release
   ```

### Option 3: Quick Install via Chocolatey (If you have it)

```powershell
choco install visualcppbuildtools
```

Then restart your terminal and build again.

## Verify Installation

After installing, verify Rust can find the linker:

```powershell
rustc --version
cargo --version
cargo build --release
```

## Which Option Should I Choose?

- **Option 1 (MSVC)**: Best compatibility, smaller binaries, recommended for Windows
- **Option 2 (GNU)**: No Visual Studio needed, but requires MinGW setup
- **Option 3**: Fastest if you already use Chocolatey

Most users should choose **Option 1** for the best experience.

