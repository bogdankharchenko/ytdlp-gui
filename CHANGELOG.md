# Changelog

All notable changes to this project will be documented in this file.

## [0.1.3] - 2025-10-04

### New Features

- **Warm Earth Theme**: New warm, earthy color palette for both light and dark modes
  - Dark mode: Rich terracotta accent (#D97757) on warm dark backgrounds
  - Light mode: Soft beige backgrounds with terracotta accents
  - Easy on the eyes with warm tones throughout
- **Download Success UI**: Clickable file path displayed after download completes
  - Opens file with default app or reveals in Finder/Explorer
  - Subtle muted text with truncation for long paths
- **Progress Indicator**: Download progress now fills the button from left to right
  - No separate progress bar - cleaner UI
  - Visual feedback integrated into download button
- **File Overwrite**: Downloads now properly overwrite existing files with `--force-overwrites`
- **Compact UI**: Reduced vertical spacing and window height (700x480)
  - Reorganized format selection into cleaner toggle-style layout
  - Type and Quality selectors combined in video info card
  - Resizable window with minimum dimensions

### Bug Fixes

- **Development Mode FFmpeg**: Fixed ffmpeg not found in dev mode by using CARGO_MANIFEST_DIR
- **ARM64 FFmpeg**: Fixed architecture mismatch - now downloads correct ARM64 binary for Apple Silicon
  - Documented official FFmpeg sources and ARM64 limitation
- **Opener Permissions**: Added proper permissions for opening downloaded files in user directories

### Improvements

- Simplified download button text to just "Download" (no Video/Audio suffix)
- Better hover states in light theme with readable text colors
- More compact window size while maintaining usability
- Official FFmpeg download sources documented in build.rs

## [0.1.2] - 2025-10-04

### Bug Fixes

- **FFmpeg Path Resolution**: Fixed ffmpeg not being found in bundled macOS apps
  - Corrected path to use `Contents/MacOS/` instead of `Contents/Resources/`
  - Now properly resolves platform-specific binary locations
- **Audio Extraction**: Added `-x` (extract-audio) and `--audio-format` flags for audio downloads
  - Properly handles audio format conversion (m4a, mp3, opus)
  - Fixed issues with audio file extension handling
- **Format Selection**: Added fallback options for format selection
  - When specific format is unavailable, falls back to best quality
  - Prevents "Requested format is not available" errors

### Improvements

- Fixed bundle identifier warning (changed from `com.ytdlp-gui.app` to `com.ytdlp.gui`)
- Better error handling for unavailable formats
- More robust format string construction with fallbacks

## [0.1.1] - 2025-10-04

### Bug Fixes

- **Audio Downloads**: Fixed `'NoneType' object has no attribute 'lower'` error when downloading audio
  - Ensured format parameter is always provided as a required string
  - Added proper output format handling based on file extension
  - Only use `--merge-output-format` for video containers (mp4, mkv, webm)
  - Audio formats (m4a, mp3, opus) now download correctly without merge format errors

### Improvements

- Simplified format string logic in download function
- Better error handling for audio/video format selection

## [0.1.0] - 2025-10-04

### Initial Release

A clean, modern desktop application for downloading videos and audio from YouTube using yt-dlp.

### Features

- **Clean UI**: Modern interface built with React, Tauri v2, and shadcn/ui components
- **Light/Dark Mode**: System-aware theme switching with grayscale color scheme
- **Video Downloads**:
  - Support for multiple quality options (360p, 480p, 720p, 1080p, 4K, etc.)
  - Automatic video+audio merging for high-quality streams
  - Format selection with file size information
- **Audio Downloads**:
  - Extract audio-only formats
  - Multiple quality options (bitrate-based)
  - Support for m4a, mp3, opus, webm formats
- **Real-time Progress**:
  - Live download progress with percentage
  - Download speed indicator
  - ETA (estimated time remaining)
- **Keyboard Shortcuts**:
  - `Cmd+N` / `Ctrl+N`: Reset state and focus input
  - `Enter`: Fetch video information
- **Automatic Dependencies**:
  - yt-dlp binary automatically downloaded during build
  - ffmpeg binary automatically downloaded and bundled
  - No manual installation required

### Technical Details

- **Frontend**: React 18 + Vite + Tailwind CSS
- **Backend**: Rust + Tauri v2
- **UI Components**: shadcn/ui (Radix UI primitives)
- **Video Downloader**: yt-dlp (bundled)
- **Video Processing**: ffmpeg (bundled)

### Platform Support

- macOS (Apple Silicon & Intel)
- Windows (x64)
- Linux (x64)

### Fixes

- Fixed video format filtering to show all available quality options
- Fixed hover states on audio/video toggle buttons
- Bundled ffmpeg to support automatic video+audio stream merging

### Known Issues

- macOS builds are unsigned (requires Apple Developer account)
  - Workaround: Use `xattr -cr /path/to/app.app` after download
  - Or build locally with `npm run tauri build`
