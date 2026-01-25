fn main() {
    // Embed Windows icon and manifest if building for Windows
    #[cfg(target_os = "windows")]
    {
        let icon_path = std::path::Path::new("assets/icon.ico");
        let manifest_path = std::path::Path::new("assets/app.manifest");
        
        if icon_path.exists() {
            // Get absolute paths for the resource file
            let manifest = std::env::var("CARGO_MANIFEST_DIR").unwrap();
            let icon_full_path = std::path::Path::new(&manifest).join("assets").join("icon.ico");
            let manifest_full_path = std::path::Path::new(&manifest).join("assets").join("app.manifest");
            
            // Create resource file content with absolute paths
            let mut rc_content = format!("1 ICON \"{}\"\n", icon_full_path.to_string_lossy().replace('\\', "/"));
            
            // Add manifest if it exists (RT_MANIFEST = 24)
            if manifest_path.exists() {
                rc_content.push_str(&format!("1 24 \"{}\"\n", manifest_full_path.to_string_lossy().replace('\\', "/")));
            }
            
            // Write temporary resource file in assets directory
            let temp_rc = "assets/icon_temp.rc";
            if std::fs::write(temp_rc, rc_content).is_ok() {
                embed_resource::compile(temp_rc, std::iter::empty::<&str>());
                // Clean up temp file
                let _ = std::fs::remove_file(temp_rc);
            }
        }
        
        // Use CONSOLE subsystem to enable Windows Terminal support
        // The custom icon will still be embedded and shown in the taskbar
        // Windows Terminal will be used automatically if set as default terminal
        println!("cargo:rustc-link-arg=/SUBSYSTEM:CONSOLE");
    }
}
