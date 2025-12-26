# Fixing Taskbar Icon in Windows

If the taskbar icon is still showing the Windows terminal icon instead of the malScraper icon, try these steps:

## 1. Rebuild the Application
```powershell
cd rust
cargo clean
cargo build --release
```

## 2. Clear Windows Icon Cache

Windows caches icons, so you may need to clear the cache:

### Method 1: Restart Windows Explorer
1. Press `Ctrl + Shift + Esc` to open Task Manager
2. Find "Windows Explorer" in the list
3. Right-click and select "Restart"

### Method 2: Clear Icon Cache (Advanced)
1. Open Command Prompt as Administrator
2. Run:
   ```cmd
   ie4uinit.exe -show
   ```
3. Or delete the icon cache:
   - Close all applications
   - Navigate to: `C:\Users\YourUsername\AppData\Local`
   - Delete the `IconCache.db` file (if it exists)
   - Restart Windows Explorer (see Method 1)

## 3. Verify Icon is Embedded

1. Navigate to `rust/target/release/`
2. Right-click `malscraper.exe`
3. Select "Properties"
4. Check the icon in the Properties dialog - it should show your icon
5. If the icon appears in Properties but not in the taskbar, it's a cache issue (see step 2)

## 4. Pin to Taskbar

After clearing the cache:
1. Run the application
2. Right-click the taskbar icon
3. Select "Pin to taskbar"
4. The pinned icon should show your custom icon

## Notes

- The icon is embedded in the executable during build
- Windows may take a moment to update the icon cache
- If the icon appears in File Explorer but not the taskbar, it's definitely a cache issue
- Restarting your computer will also clear the icon cache

