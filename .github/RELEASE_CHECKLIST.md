# Release Checklist

## Pre-Release

- [ ] Test the application locally
  ```bash
  cd rust
  cargo build --release
  cargo test
  ```

- [ ] Verify version number in `rust/Cargo.toml`
- [ ] Update `CHANGELOG.md` (if you create one)
- [ ] Commit all changes
- [ ] Push to repository

## Creating a Release

1. **Create and push a version tag:**
   ```bash
   git tag v1.5.0
   git push origin v1.5.0
   ```

2. **GitHub Actions will automatically:**
   - Build binaries for all platforms
   - Create a GitHub Release
   - Upload all binaries

3. **Verify the release:**
   - Go to GitHub Releases page
   - Check that all binaries are uploaded
   - Test downloading and running a binary

## Post-Release

- [ ] Update any external documentation
- [ ] Announce the release (if desired)
- [ ] Monitor for any issues

## Manual Release (if needed)

If GitHub Actions fails, you can manually build:

```bash
# Windows
cargo build --release --target x86_64-pc-windows-msvc

# macOS (Intel)
cargo build --release --target x86_64-apple-darwin

# macOS (Apple Silicon)
cargo build --release --target aarch64-apple-darwin

# Linux (x86_64)
cargo build --release --target x86_64-unknown-linux-gnu

# Linux (ARM64) - requires cross-compilation setup
cargo build --release --target aarch64-unknown-linux-gnu
```

