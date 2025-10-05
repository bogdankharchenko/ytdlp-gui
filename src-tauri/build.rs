use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Download yt-dlp and ffmpeg binaries before building
    download_ytdlp().expect("Failed to download yt-dlp binary");
    download_ffmpeg().expect("Failed to download ffmpeg binary");

    tauri_build::build()
}

fn download_ytdlp() -> Result<(), Box<dyn std::error::Error>> {
    let target_os = env::var("CARGO_CFG_TARGET_OS")?;
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH")?;

    // Determine the binary name and download URL based on platform
    let (binary_name, download_url) = match (target_os.as_str(), target_arch.as_str()) {
        ("macos", "aarch64") => (
            "yt-dlp-aarch64-apple-darwin",
            "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos"
        ),
        ("macos", "x86_64") => (
            "yt-dlp-x86_64-apple-darwin",
            "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_macos"
        ),
        ("windows", _) => (
            "yt-dlp-x86_64-pc-windows-msvc.exe",
            "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp.exe"
        ),
        ("linux", "x86_64") => (
            "yt-dlp-x86_64-unknown-linux-gnu",
            "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_linux"
        ),
        ("linux", "aarch64") => (
            "yt-dlp-aarch64-unknown-linux-gnu",
            "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp_linux_aarch64"
        ),
        _ => {
            println!("cargo:warning=Unsupported platform: {} {}", target_os, target_arch);
            return Ok(());
        }
    };

    // Create binaries directory if it doesn't exist
    let binaries_dir = PathBuf::from("binaries");
    fs::create_dir_all(&binaries_dir)?;

    let binary_path = binaries_dir.join(binary_name);

    // Check if binary already exists
    if binary_path.exists() {
        println!("cargo:warning=yt-dlp binary already exists at {:?}", binary_path);
        return Ok(());
    }

    println!("cargo:warning=Downloading yt-dlp from {}", download_url);

    // Download the binary
    let client = reqwest::blocking::Client::builder()
        .user_agent("ytdlp-gui-build-script")
        .timeout(std::time::Duration::from_secs(300))
        .build()?;

    let response = client.get(download_url).send()?;

    if !response.status().is_success() {
        return Err(format!("Failed to download yt-dlp: HTTP {}", response.status()).into());
    }

    let bytes = response.bytes()?;
    fs::write(&binary_path, bytes)?;

    // Set executable permissions on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&binary_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&binary_path, perms)?;
    }

    println!("cargo:warning=Successfully downloaded yt-dlp to {:?}", binary_path);
    println!("cargo:rerun-if-changed=binaries/{}", binary_name);

    Ok(())
}

fn download_ffmpeg() -> Result<(), Box<dyn std::error::Error>> {
    let target_os = env::var("CARGO_CFG_TARGET_OS")?;
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH")?;

    // Determine the binary name and download URL based on platform
    // Note: Official FFmpeg download page (https://www.ffmpeg.org/download.html) recommends:
    // - macOS: https://evermeet.cx/ffmpeg/ (x86_64 only, no ARM64 support)
    // - Windows: https://github.com/BtbN/FFmpeg-Builds/releases
    // - Linux: https://johnvansickle.com/ffmpeg/
    //
    // For macOS ARM64: evermeet.cx doesn't provide ARM64 binaries, and there are no official
    // FFmpeg ARM64 builds for macOS. We use osxexperts.net which provides working ARM64 builds.
    let (binary_name, download_url) = match (target_os.as_str(), target_arch.as_str()) {
        ("macos", "aarch64") => (
            "ffmpeg-aarch64-apple-darwin",
            "https://www.osxexperts.net/ffmpeg7arm.zip"  // ARM64 build (no official source available)
        ),
        ("macos", "x86_64") => (
            "ffmpeg-x86_64-apple-darwin",
            "https://evermeet.cx/ffmpeg/getrelease/ffmpeg/zip"  // Official recommended source
        ),
        ("windows", _) => (
            "ffmpeg-x86_64-pc-windows-msvc.exe",
            "https://github.com/BtbN/FFmpeg-Builds/releases/download/latest/ffmpeg-master-latest-win64-gpl.zip"
        ),
        ("linux", "x86_64") => (
            "ffmpeg-x86_64-unknown-linux-gnu",
            "https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-amd64-static.tar.xz"
        ),
        ("linux", "aarch64") => (
            "ffmpeg-aarch64-unknown-linux-gnu",
            "https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-arm64-static.tar.xz"
        ),
        _ => {
            println!("cargo:warning=Unsupported platform for ffmpeg: {} {}", target_os, target_arch);
            return Ok(());
        }
    };

    // Create binaries directory if it doesn't exist
    let binaries_dir = PathBuf::from("binaries");
    fs::create_dir_all(&binaries_dir)?;

    let binary_path = binaries_dir.join(binary_name);

    // Check if binary already exists
    if binary_path.exists() {
        println!("cargo:warning=ffmpeg binary already exists at {:?}", binary_path);
        return Ok(());
    }

    println!("cargo:warning=Downloading ffmpeg from {}", download_url);

    // Download the archive
    let client = reqwest::blocking::Client::builder()
        .user_agent("ytdlp-gui-build-script")
        .timeout(std::time::Duration::from_secs(600))
        .build()?;

    let response = client.get(download_url).send()?;

    if !response.status().is_success() {
        return Err(format!("Failed to download ffmpeg: HTTP {}", response.status()).into());
    }

    let bytes = response.bytes()?;

    // Extract ffmpeg binary from archive
    match target_os.as_str() {
        "macos" => {
            // macOS: ZIP file containing ffmpeg binary
            let temp_zip = binaries_dir.join("ffmpeg.zip");
            fs::write(&temp_zip, bytes)?;

            // Extract using unzip command
            std::process::Command::new("unzip")
                .args(&["-o", "-j", temp_zip.to_str().unwrap(), "-d", binaries_dir.to_str().unwrap()])
                .output()?;

            // Rename extracted binary
            let extracted = binaries_dir.join("ffmpeg");
            if extracted.exists() {
                fs::rename(&extracted, &binary_path)?;
            }

            // Clean up zip file
            let _ = fs::remove_file(temp_zip);
        },
        "windows" => {
            // Windows: ZIP file containing ffmpeg.exe in a subdirectory
            let temp_zip = binaries_dir.join("ffmpeg.zip");
            fs::write(&temp_zip, bytes)?;

            // Extract using PowerShell
            std::process::Command::new("powershell")
                .args(&["-Command", &format!("Expand-Archive -Path '{}' -DestinationPath '{}' -Force", temp_zip.display(), binaries_dir.display())])
                .output()?;

            // Find and rename ffmpeg.exe
            for entry in walkdir::WalkDir::new(&binaries_dir) {
                let entry = entry?;
                if entry.file_name() == "ffmpeg.exe" {
                    fs::rename(entry.path(), &binary_path)?;
                    break;
                }
            }

            // Clean up
            let _ = fs::remove_file(temp_zip);
        },
        "linux" => {
            // Linux: tar.xz archive
            let temp_tar = binaries_dir.join("ffmpeg.tar.xz");
            fs::write(&temp_tar, bytes)?;

            // Extract using tar command
            std::process::Command::new("tar")
                .args(&["-xJf", temp_tar.to_str().unwrap(), "-C", binaries_dir.to_str().unwrap(), "--strip-components=1", "--wildcards", "*/ffmpeg"])
                .output()?;

            // Rename extracted binary
            let extracted = binaries_dir.join("ffmpeg");
            if extracted.exists() {
                fs::rename(&extracted, &binary_path)?;
            }

            // Clean up tar file
            let _ = fs::remove_file(temp_tar);
        },
        _ => return Ok(()),
    }

    // Set executable permissions on Unix systems
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if binary_path.exists() {
            let mut perms = fs::metadata(&binary_path)?.permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&binary_path, perms)?;
        }
    }

    println!("cargo:warning=Successfully downloaded ffmpeg to {:?}", binary_path);
    println!("cargo:rerun-if-changed=binaries/{}", binary_name);

    Ok(())
}
