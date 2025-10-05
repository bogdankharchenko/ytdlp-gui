# Changelog

All notable changes to this project will be documented in this file.

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
