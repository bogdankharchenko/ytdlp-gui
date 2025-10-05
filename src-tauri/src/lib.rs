use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_shell::ShellExt;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VideoFormat {
    pub format_id: String,
    pub ext: String,
    pub resolution: Option<String>,
    pub filesize: Option<u64>,
    pub format_note: Option<String>,
    pub vcodec: Option<String>,
    pub acodec: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoInfo {
    pub title: String,
    pub thumbnail: Option<String>,
    pub duration: Option<f64>,
    pub uploader: Option<String>,
    pub description: Option<String>,
    pub formats: Vec<VideoFormat>,
}

#[derive(Debug, Serialize, Clone)]
pub struct DownloadProgress {
    pub percentage: f64,
    pub speed: Option<String>,
    pub eta: Option<String>,
    pub status: String,
}

#[tauri::command]
async fn get_video_info(url: String, app: AppHandle) -> Result<VideoInfo, String> {
    let sidecar_command = app.shell()
        .sidecar("yt-dlp")
        .map_err(|e| format!("Failed to create sidecar command: {}", e))?;

    let output = sidecar_command
        .args(["--dump-json", "--no-playlist", &url])
        .output()
        .await
        .map_err(|e| format!("Failed to execute yt-dlp: {}", e))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("yt-dlp error: {}", error));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json_value: serde_json::Value = serde_json::from_str(&stdout)
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;

    let formats: Vec<VideoFormat> = json_value["formats"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|f| {
            Some(VideoFormat {
                format_id: f["format_id"].as_str()?.to_string(),
                ext: f["ext"].as_str()?.to_string(),
                resolution: f["resolution"].as_str().map(|s| s.to_string()),
                filesize: f["filesize"].as_u64(),
                format_note: f["format_note"].as_str().map(|s| s.to_string()),
                vcodec: f["vcodec"].as_str().map(|s| s.to_string()),
                acodec: f["acodec"].as_str().map(|s| s.to_string()),
            })
        })
        .collect();

    Ok(VideoInfo {
        title: json_value["title"]
            .as_str()
            .unwrap_or("Unknown")
            .to_string(),
        thumbnail: json_value["thumbnail"].as_str().map(|s| s.to_string()),
        duration: json_value["duration"].as_f64(),
        uploader: json_value["uploader"].as_str().map(|s| s.to_string()),
        description: json_value["description"].as_str().map(|s| s.to_string()),
        formats,
    })
}

#[tauri::command]
async fn list_formats(url: String, app: AppHandle) -> Result<String, String> {
    let sidecar_command = app.shell()
        .sidecar("yt-dlp")
        .map_err(|e| format!("Failed to create sidecar command: {}", e))?;

    let output = sidecar_command
        .args(["-F", &url])
        .output()
        .await
        .map_err(|e| format!("Failed to execute yt-dlp: {}", e))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("yt-dlp error: {}", error));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[tauri::command]
async fn download_video(
    url: String,
    format: String,
    output_path: String,
    app: AppHandle,
) -> Result<String, String> {
    let mut args = vec![];

    // Get ffmpeg path - different in dev vs production
    let ffmpeg_path = if cfg!(debug_assertions) {
        // Development mode: use binaries from binaries/ directory relative to cargo manifest
        let target_os = std::env::consts::OS;
        let target_arch = std::env::consts::ARCH;

        let ffmpeg_name = match (target_os, target_arch) {
            ("macos", "aarch64") => "ffmpeg-aarch64-apple-darwin",
            ("macos", "x86_64") => "ffmpeg-x86_64-apple-darwin",
            ("windows", _) => "ffmpeg-x86_64-pc-windows-msvc.exe",
            ("linux", "x86_64") => "ffmpeg-x86_64-unknown-linux-gnu",
            ("linux", "aarch64") => "ffmpeg-aarch64-unknown-linux-gnu",
            _ => "ffmpeg",
        };

        // Use CARGO_MANIFEST_DIR which points to src-tauri directory
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
            .unwrap_or_else(|_| ".".to_string());
        std::path::PathBuf::from(manifest_dir)
            .join("binaries")
            .join(ffmpeg_name)
    } else {
        // Production mode: Tauri v2 places sidecar binaries with the platform-specific suffix removed
        // They end up in Contents/MacOS/ on macOS, next to the main binary
        let resource_dir = app.path()
            .resource_dir()
            .map_err(|e| format!("Failed to get resource dir: {}", e))?;

        // On macOS, binaries are in ../MacOS relative to Resources
        // On other platforms, they're directly in the resource dir
        let bin_dir = if cfg!(target_os = "macos") {
            resource_dir.parent().unwrap().join("MacOS")
        } else {
            resource_dir.clone()
        };

        // Tauri strips the platform suffix from sidecar binaries, so it's just "ffmpeg" (or "ffmpeg.exe" on Windows)
        let ffmpeg_name = if cfg!(target_os = "windows") {
            "ffmpeg.exe"
        } else {
            "ffmpeg"
        };

        bin_dir.join(ffmpeg_name)
    };

    // Only add ffmpeg-location if the file exists
    if ffmpeg_path.exists() {
        args.push("--ffmpeg-location".to_string());
        args.push(ffmpeg_path.to_string_lossy().to_string());
    }

    // Always use the provided format string
    args.push("-f".to_string());
    args.push(format.clone());

    // Extract the file extension from output_path and handle format accordingly
    let path = std::path::Path::new(&output_path);
    let is_audio_format = format.contains("bestaudio") || format.contains("audio");

    if let Some(ext) = path.extension() {
        if let Some(ext_str) = ext.to_str() {
            match ext_str {
                "mp4" | "mkv" | "webm" if !is_audio_format => {
                    // Video download - use merge output format
                    args.push("--merge-output-format".to_string());
                    args.push(ext_str.to_string());
                }
                "m4a" | "mp3" | "opus" => {
                    // Audio download - use extract-audio and audio-format
                    args.push("-x".to_string()); // --extract-audio
                    args.push("--audio-format".to_string());
                    args.push(ext_str.to_string());
                }
                _ => {}
            }
        }
    }

    args.push("-o".to_string());
    args.push(output_path.clone());
    args.push("--force-overwrites".to_string());
    args.push("--newline".to_string());
    args.push("--no-playlist".to_string());
    args.push(url.clone());

    let sidecar_command = app.shell()
        .sidecar("yt-dlp")
        .map_err(|e| format!("Failed to create sidecar command: {}", e))?;

    let (mut rx, _child) = sidecar_command
        .args(&args)
        .spawn()
        .map_err(|e| format!("Failed to spawn yt-dlp: {}", e))?;

    let app_clone = app.clone();

    tauri::async_runtime::spawn(async move {
        use regex::Regex;
        let progress_regex = Regex::new(r"\[download\]\s+(\d+\.?\d*)%").unwrap();
        let speed_regex = Regex::new(r"at\s+([^\s]+/s)").unwrap();
        let eta_regex = Regex::new(r"ETA\s+(\d+:\d+)").unwrap();

        while let Some(event) = rx.recv().await {
            if let tauri_plugin_shell::process::CommandEvent::Stdout(line) = event {
                let line_str = String::from_utf8_lossy(&line);

                let mut progress = DownloadProgress {
                    percentage: 0.0,
                    speed: None,
                    eta: None,
                    status: "downloading".to_string(),
                };

                if let Some(caps) = progress_regex.captures(&line_str) {
                    if let Some(pct) = caps.get(1) {
                        progress.percentage = pct.as_str().parse().unwrap_or(0.0);
                    }
                }

                if let Some(caps) = speed_regex.captures(&line_str) {
                    if let Some(spd) = caps.get(1) {
                        progress.speed = Some(spd.as_str().to_string());
                    }
                }

                if let Some(caps) = eta_regex.captures(&line_str) {
                    if let Some(eta) = caps.get(1) {
                        progress.eta = Some(eta.as_str().to_string());
                    }
                }

                if line_str.contains("[download] 100%") {
                    progress.status = "completed".to_string();
                    progress.percentage = 100.0;
                }

                let _ = app_clone.emit("download-progress", progress);
            } else if let tauri_plugin_shell::process::CommandEvent::Stderr(line) = event {
                let error = String::from_utf8_lossy(&line);
                let _ = app_clone.emit("download-error", error.to_string());
            }
        }

        let _ = app_clone.emit("download-finished", ());
    });

    Ok("Download started".to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            get_video_info,
            list_formats,
            download_video
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
