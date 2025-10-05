# YT-DLP GUI

A clean, modern desktop application for downloading videos and audio from YouTube and other platforms. Built with [Tauri](https://tauri.app/) and React, powered by [yt-dlp](https://github.com/yt-dlp/yt-dlp).

## Features

- 🎥 **Video & Audio Downloads** - Download videos or extract audio-only files
- 🎯 **Quality Selection** - Choose from available quality options or get the best quality automatically
- 🎨 **Clean Interface** - Modern UI built with shadcn/ui components
- 🌓 **Light/Dark Mode** - Automatic theme switching with system preferences
- ⚡ **Fast & Lightweight** - Native performance with Tauri's Rust backend
- 📊 **Progress Tracking** - Real-time download progress with speed and ETA
- ⌨️ **Keyboard Shortcuts** - Quick actions with CMD+N (reset) and Enter (fetch)
- 🔄 **Auto-Updates** - Built-in yt-dlp binary download and updates
- 🖥️ **Cross-Platform** - Available for macOS, Linux, and Windows

## Installation

Download the latest release for your platform from the [Releases](https://github.com/bogdankharchenko/ytdlp-gui/releases) page:

- **macOS**: Download the `.dmg` file (Apple Silicon or Intel)
- **Linux**: Download `.AppImage`, `.deb`, or `.rpm` package
- **Windows**: Download the `.msi` or `.exe` installer

### macOS Installation Notes

**Important**: The releases are currently unsigned. macOS will show a warning that the app is "damaged" or from an "unidentified developer."

To bypass this security warning, use one of these methods:

**Method 1: Remove Quarantine Attribute (Recommended)**
```bash
# After mounting the DMG
xattr -cr /Volumes/ytdlp-gui/ytdlp-gui.app

# Or after copying to Applications
xattr -cr /Applications/ytdlp-gui.app
```

**Method 2: System Settings**
1. Try to open the app (it will be blocked)
2. Go to System Settings → Privacy & Security
3. Scroll down to find "ytdlp-gui was blocked"
4. Click "Open Anyway"

**Method 3: Build Locally**
```bash
git clone https://github.com/bogdankharchenko/ytdlp-gui.git
cd ytdlp-gui
npm install
npm run tauri build
```
The locally built app in `src-tauri/target/release/bundle/` will work without any security warnings.

> **Note**: Proper code signing will be added in a future release. This requires an Apple Developer account.

## Usage

1. **Paste a URL** - Enter a YouTube or supported video URL
2. **Press Enter or click Fetch** - The app will retrieve video information
3. **Choose Format** - Select Video or Audio, then pick your preferred quality
4. **Download** - Click the download button and choose where to save

### Keyboard Shortcuts

- `Enter` - Fetch video information
- `CMD+N` (or `CTRL+N`) - Reset and start a new download

## Supported Platforms

YT-DLP GUI supports downloading from any platform that yt-dlp supports, including:

- YouTube
- Vimeo
- Twitter/X
- TikTok
- Reddit
- Facebook
- Instagram
- And [1000+ more sites](https://github.com/yt-dlp/yt-dlp/blob/master/supportedsites.md)

## Development

### Prerequisites

- [Node.js](https://nodejs.org/) (LTS version)
- [Rust](https://rustup.rs/)
- [Tauri prerequisites](https://tauri.app/v2/guides/prerequisites/) for your platform

### Setup

```bash
# Clone the repository
git clone https://github.com/bogdankharchenko/ytdlp-gui.git
cd ytdlp-gui

# Install dependencies
npm install

# Run in development mode
npm run tauri dev
```

### Build

```bash
# Build for production
npm run tauri build
```

The built application will be in `src-tauri/target/release/`.

## Technology Stack

- **Frontend**: React 18, Vite
- **UI Components**: shadcn/ui, Tailwind CSS, Lucide icons
- **Backend**: Tauri 2.x (Rust)
- **Downloader**: yt-dlp (automatically downloaded during build)
- **Icons**: Lucide React

## How It Works

1. **Binary Management**: The app automatically downloads the appropriate yt-dlp binary for your platform during build time
2. **Video Information**: Uses yt-dlp's JSON output to fetch video metadata and available formats
3. **Download Process**: Spawns yt-dlp as a subprocess and parses progress output in real-time
4. **Format Selection**: Filters and sorts available formats by quality and codec

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - see [LICENSE](LICENSE) for details

## Credits

- [yt-dlp](https://github.com/yt-dlp/yt-dlp) - The amazing command-line downloader
- [Tauri](https://tauri.app/) - Build smaller, faster desktop applications
- [shadcn/ui](https://ui.shadcn.com/) - Beautiful UI components

## Disclaimer

This tool is for personal use only. Please respect content creators and platform terms of service. Only download content you have permission to download.
