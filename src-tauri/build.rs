use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    // Download yt-dlp binary before building
    download_ytdlp().expect("Failed to download yt-dlp binary");

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
