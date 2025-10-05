use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter, Manager};
use std::process::Child;
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

// Store the current download process
struct DownloadState {
    process: Option<Child>,
}

impl DownloadState {
    fn new() -> Self {
        Self { process: None }
    }
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

    // Get ffmpeg path - we need to construct it from the resource directory
    // Tauri places sidecar binaries in the resource directory
    let resource_dir = app.path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource dir: {}", e))?;

    let ffmpeg_name = if cfg!(target_os = "windows") {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    };

    let ffmpeg_path = resource_dir.join(ffmpeg_name);

    args.push("--ffmpeg-location".to_string());
    args.push(ffmpeg_path.to_string_lossy().to_string());

    // Always use the provided format string
    args.push("-f".to_string());
    args.push(format.clone());

    // Extract the file extension from output_path
    let path = std::path::Path::new(&output_path);
    if let Some(ext) = path.extension() {
        if let Some(ext_str) = ext.to_str() {
            // Only use merge-output-format for video containers (mp4, mkv, webm)
            // Audio formats (m4a, mp3, opus) should use --audio-format and --extract-audio
            match ext_str {
                "mp4" | "mkv" | "webm" => {
                    args.push("--merge-output-format".to_string());
                    args.push(ext_str.to_string());
                }
                "m4a" | "mp3" | "opus" => {
                    // For audio extraction, let yt-dlp handle it naturally
                    // The format selector will already specify audio-only
                }
                _ => {}
            }
        }
    }

    args.push("-o".to_string());
    args.push(output_path.clone());
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
        .setup(|app| {
            app.manage(Arc::new(Mutex::new(DownloadState::new())));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_video_info,
            list_formats,
            download_video
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
