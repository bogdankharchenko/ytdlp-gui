# Release Process

This project uses GitHub Actions to automatically build and release the application for multiple platforms.

## Workflows

### CI Workflow (`ci.yml`)
- **Triggers**: Push to `main` branch or pull requests
- **Purpose**: Test builds across all platforms (macOS, Linux, Windows)
- **What it does**: Validates that the app builds successfully on all platforms

### Release Workflow (`release.yml`)
- **Triggers**: Git tags starting with `v` (e.g., `v1.0.0`) or manual trigger
- **Purpose**: Build and publish releases
- **Platforms**:
  - macOS (Apple Silicon - aarch64)
  - macOS (Intel - x86_64)
  - Linux (x86_64)
  - Windows (x86_64)

## Creating a Release

### Option 1: Tag-based Release (Recommended)

1. Update the version in `src-tauri/Cargo.toml` and `package.json`
2. Commit your changes:
   ```bash
   git add .
   git commit -m "Bump version to 1.0.0"
   ```

3. Create and push a git tag:
   ```bash
   git tag v1.0.0
   git push origin main
   git push origin v1.0.0
   ```

4. GitHub Actions will automatically:
   - Build the app for all platforms
   - Create a new GitHub Release
   - Upload installers/packages to the release

### Option 2: Manual Release

1. Go to the "Actions" tab in your GitHub repository
2. Select the "Release" workflow
3. Click "Run workflow"
4. Select the branch and click "Run workflow"

## Release Artifacts

The workflow will create platform-specific installers:

- **macOS**:
  - `.dmg` installer (Apple Silicon)
  - `.dmg` installer (Intel)
  - `.app.tar.gz` archive

- **Linux**:
  - `.AppImage` (portable)
  - `.deb` package (Debian/Ubuntu)
  - `.rpm` package (Fedora/RedHat)

- **Windows**:
  - `.msi` installer
  - `.exe` setup file

## First-Time Setup

Before your first release, make sure:

1. **Repository Settings**: Enable "Read and write permissions" for GitHub Actions
   - Go to Settings → Actions → General → Workflow permissions
   - Select "Read and write permissions"
   - Check "Allow GitHub Actions to create and approve pull requests"

2. **Code Signing** (Optional but recommended for production):

   **macOS**:
   - Add `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_ID`, and `APPLE_PASSWORD` secrets

   **Windows**:
   - Add code signing certificate to secrets

   See [Tauri documentation](https://tauri.app/v1/guides/distribution/sign-macos/) for details.

## Version Bumping

Remember to update versions in both files:

1. `src-tauri/Cargo.toml`:
   ```toml
   [package]
   version = "1.0.0"
   ```

2. `package.json`:
   ```json
   {
     "version": "1.0.0"
   }
   ```

## Troubleshooting

- **Build fails on a specific platform**: Check the Actions logs for that platform
- **Release not created**: Ensure the tag starts with `v` (e.g., `v1.0.0`, not `1.0.0`)
- **Permission denied**: Check workflow permissions in repository settings
