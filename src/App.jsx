import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { save } from "@tauri-apps/plugin-dialog";
import { Moon, Sun, Video, Music, Download, Loader2 } from "lucide-react";
import { Button } from "./components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "./components/ui/card";
import { Input } from "./components/ui/input";
import { Progress } from "./components/ui/progress";
import { Badge } from "./components/ui/badge";
import { useTheme } from "./components/theme-provider";
import { cn } from "./lib/utils";

function App() {
  const { theme, setTheme } = useTheme();
  const [url, setUrl] = useState("");
  const [videoInfo, setVideoInfo] = useState(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState("");
  const [downloadType, setDownloadType] = useState("video");
  const [selectedFormat, setSelectedFormat] = useState("");
  const [downloading, setDownloading] = useState(false);
  const [progress, setProgress] = useState({ percentage: 0, speed: null, eta: null, status: "idle" });
  const inputRef = useRef(null);

  const resetState = () => {
    setUrl("");
    setVideoInfo(null);
    setError("");
    setSelectedFormat("");
    setDownloadType("video");
    setLoading(false);
    setDownloading(false);
    setProgress({ percentage: 0, speed: null, eta: null, status: "idle" });
    inputRef.current?.focus();
  };

  useEffect(() => {
    const unlistenProgress = listen("download-progress", (event) => setProgress(event.payload));
    const unlistenFinished = listen("download-finished", () => setDownloading(false));
    const unlistenError = listen("download-error", (event) => {
      setError(`Download error: ${event.payload}`);
      setDownloading(false);
    });

    const handleKeyDown = (e) => {
      if ((e.metaKey || e.ctrlKey) && e.key === "n") {
        e.preventDefault();
        resetState();
      }
    };

    window.addEventListener("keydown", handleKeyDown);

    return () => {
      unlistenProgress.then((fn) => fn());
      unlistenFinished.then((fn) => fn());
      unlistenError.then((fn) => fn());
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, []);

  const fetchVideoInfo = async () => {
    setLoading(true);
    setError("");
    setVideoInfo(null);
    setSelectedFormat("");

    try {
      const info = await invoke("get_video_info", { url });
      setVideoInfo(info);
    } catch (err) {
      setError(err);
    } finally {
      setLoading(false);
    }
  };

  const getVideoFormats = () => {
    if (!videoInfo) return [];
    const formats = videoInfo.formats
      .filter((f) => f.vcodec !== "none" && f.acodec !== "none" && f.filesize)
      .map((f) => ({ ...f, height: f.resolution ? parseInt(f.resolution.split("x")[1]) : 0 }))
      .sort((a, b) => b.height - a.height);

    const seen = new Set();
    return formats.filter((f) => {
      const key = `${f.height}p`;
      if (seen.has(key)) return false;
      seen.add(key);
      return true;
    });
  };

  const getAudioFormats = () => {
    if (!videoInfo) return [];
    const formats = videoInfo.formats
      .filter((f) => f.acodec !== "none" && f.vcodec === "none")
      .map((f) => ({ ...f, bitrate: parseInt(f.format_note?.match(/(\d+)k/)?.[1] || "0") }))
      .sort((a, b) => b.bitrate - a.bitrate);

    const seen = new Set();
    return formats.filter((f) => {
      const key = `${f.ext}-${f.bitrate}`;
      if (seen.has(key)) return false;
      seen.add(key);
      return true;
    });
  };

  const startDownload = async () => {
    try {
      let defaultExt = downloadType === "audio" ? "m4a" : "mp4";
      let formatString = selectedFormat || (downloadType === "audio"
        ? "bestaudio[ext=m4a]/bestaudio"
        : "bestvideo[ext=mp4]+bestaudio[ext=m4a]/best[ext=mp4]/best");

      const filePath = await save({
        defaultPath: `${videoInfo.title}.${defaultExt}`,
        filters: [{
          name: downloadType === "audio" ? "Audio" : "Video",
          extensions: downloadType === "audio" ? ["m4a", "mp3", "opus", "webm"] : ["mp4", "mkv", "webm"],
        }],
      });

      if (!filePath) return;

      setDownloading(true);
      setProgress({ percentage: 0, speed: null, eta: null, status: "starting" });
      await invoke("download_video", { url, format: formatString, outputPath: filePath });
    } catch (err) {
      setError(err);
      setDownloading(false);
    }
  };

  const formatDuration = (seconds) => {
    if (!seconds) return "Unknown";
    const minutes = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${minutes}:${secs.toString().padStart(2, "0")}`;
  };

  const formatFileSize = (bytes) => {
    if (!bytes) return "";
    const mb = bytes / (1024 * 1024);
    return mb > 1000 ? `${(mb / 1024).toFixed(1)} GB` : `${mb.toFixed(0)} MB`;
  };

  const videoFormats = getVideoFormats();
  const audioFormats = getAudioFormats();

  return (
    <div className="min-h-screen bg-background">
      <div className="max-w-3xl mx-auto p-8 space-y-8">
        {/* Header */}
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-2xl font-semibold tracking-tight">YT-DLP</h1>
            <p className="text-sm text-muted-foreground">Download videos and audio from YouTube</p>
          </div>
          <Button
            variant="ghost"
            size="icon"
            onClick={() => setTheme(theme === "dark" ? "light" : "dark")}
          >
            {theme === "dark" ? <Sun className="h-4 w-4" /> : <Moon className="h-4 w-4" />}
          </Button>
        </div>

        {/* URL Input */}
        <div className="space-y-2">
          <label className="text-sm font-medium">Video URL</label>
          <div className="flex gap-2">
            <Input
              ref={inputRef}
              placeholder="https://youtube.com/watch?v=..."
              value={url}
              onChange={(e) => setUrl(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter" && url && !loading && !downloading) {
                  fetchVideoInfo();
                }
              }}
              disabled={loading || downloading}
              className="h-9"
            />
            <Button onClick={fetchVideoInfo} disabled={!url || loading || downloading} className="h-9 px-3">
              {loading ? <Loader2 className="h-4 w-4 animate-spin" /> : "Fetch"}
            </Button>
          </div>
          {error && <p className="text-sm text-destructive">{error}</p>}
        </div>

        {/* Video Info */}
        {videoInfo && (
          <>
            <div className="rounded-lg border bg-card">
              <div className="p-6">
                <div className="flex gap-4">
                  {videoInfo.thumbnail && (
                    <img src={videoInfo.thumbnail} alt="Thumbnail" className="w-32 h-20 object-cover rounded border" />
                  )}
                  <div className="flex-1 min-w-0">
                    <h3 className="font-medium leading-tight line-clamp-2 mb-2">{videoInfo.title}</h3>
                    <div className="flex gap-3 text-xs text-muted-foreground">
                      {videoInfo.uploader && <span>{videoInfo.uploader}</span>}
                      {videoInfo.duration && <span>{formatDuration(videoInfo.duration)}</span>}
                    </div>
                  </div>
                </div>
              </div>
            </div>

            {/* Quality Selection */}
            <div className="rounded-lg border bg-card">
              <div className="p-6">
                <div className="flex items-center justify-between mb-4">
                  <h3 className="text-sm font-medium">Quality</h3>
                  <div className="flex items-center gap-0.5 bg-muted rounded-lg p-0.5">
                    <Button
                      variant="ghost"
                      size="sm"
                      className={cn(
                        "h-7 px-3 text-xs hover:bg-background/60",
                        downloadType === "video" && "bg-background shadow-sm hover:bg-background"
                      )}
                      onClick={() => { setDownloadType("video"); setSelectedFormat(""); }}
                      disabled={downloading}
                    >
                      <Video className="h-3 w-3 mr-1.5" />
                      Video
                    </Button>
                    <Button
                      variant="ghost"
                      size="sm"
                      className={cn(
                        "h-7 px-3 text-xs hover:bg-background/60",
                        downloadType === "audio" && "bg-background shadow-sm hover:bg-background"
                      )}
                      onClick={() => { setDownloadType("audio"); setSelectedFormat(""); }}
                      disabled={downloading}
                    >
                      <Music className="h-3 w-3 mr-1.5" />
                      Audio
                    </Button>
                  </div>
                </div>

                <div className="grid grid-cols-4 gap-2">
                  <Button
                    variant={!selectedFormat ? "default" : "outline"}
                    className="h-auto flex-col gap-1 py-2.5 px-2"
                    onClick={() => setSelectedFormat("")}
                    disabled={downloading}
                  >
                    <span className="text-xs font-medium">BEST</span>
                    <span className="text-[10px] text-muted-foreground">Highest</span>
                  </Button>

                  {downloadType === "video" ? (
                    videoFormats.slice(0, 7).map((format) => (
                      <Button
                        key={format.format_id}
                        variant={selectedFormat === format.format_id ? "default" : "outline"}
                        className="h-auto flex-col gap-1 py-2.5 px-2"
                        onClick={() => setSelectedFormat(format.format_id)}
                        disabled={downloading}
                      >
                        <span className="text-xs font-medium">{format.height}p</span>
                        <span className="text-[10px] text-muted-foreground">{format.ext.toUpperCase()}</span>
                        {format.filesize && <span className="text-[10px] text-muted-foreground">{formatFileSize(format.filesize)}</span>}
                      </Button>
                    ))
                  ) : (
                    audioFormats.slice(0, 7).map((format) => (
                      <Button
                        key={format.format_id}
                        variant={selectedFormat === format.format_id ? "default" : "outline"}
                        className="h-auto flex-col gap-1 py-2.5 px-2"
                        onClick={() => setSelectedFormat(format.format_id)}
                        disabled={downloading}
                      >
                        <span className="text-xs font-medium">
                          {format.bitrate ? `${format.bitrate}k` : format.ext.toUpperCase()}
                        </span>
                        <span className="text-[10px] text-muted-foreground">{format.ext.toUpperCase()}</span>
                        {format.filesize && <span className="text-[10px] text-muted-foreground">{formatFileSize(format.filesize)}</span>}
                      </Button>
                    ))
                  )}
                </div>
              </div>
            </div>

            {/* Download Button */}
            <div className="space-y-3">
              <Button
                className="w-full h-9"
                onClick={startDownload}
                disabled={downloading}
              >
                {downloading ? (
                  <><Loader2 className="mr-2 h-4 w-4 animate-spin" /> Downloading...</>
                ) : (
                  <><Download className="mr-2 h-4 w-4" /> Download {downloadType === "video" ? "Video" : "Audio"}</>
                )}
              </Button>

              {downloading && (
                <div className="rounded-lg border bg-card p-4 space-y-2">
                  <Progress value={progress.percentage} className="h-1.5" />
                  <div className="flex justify-between text-xs text-muted-foreground">
                    <span>{progress.percentage.toFixed(1)}%</span>
                    <span>{progress.speed || ""}</span>
                    <span>{progress.eta ? `ETA ${progress.eta}` : ""}</span>
                  </div>
                </div>
              )}
            </div>
          </>
        )}
      </div>
    </div>
  );
}

export default App;
