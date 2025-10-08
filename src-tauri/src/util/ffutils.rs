use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::{AppHandle, Manager};
use image::{DynamicImage, load_from_memory};

/// Minimal FFmpeg handler for bundled binaries
pub struct FFmpegHandler {
    ffmpeg_path: PathBuf,
    ffprobe_path: PathBuf,
}

impl FFmpegHandler {
    pub fn new(resource_dir: &Path) -> Self {
        let ffmpeg_path = resource_dir.join("binaries/windows/ffmpeg.exe");
        let ffprobe_path = resource_dir.join("binaries/windows/ffprobe.exe");

        if !ffmpeg_path.exists() {
            panic!("FFmpeg not found at {:?}", ffmpeg_path);
        }
        if !ffprobe_path.exists() {
            panic!("FFprobe not found at {:?}", ffprobe_path);
        }

        Self { ffmpeg_path, ffprobe_path }
    }

    /// Generate a thumbnail from a video and return it as a DynamicImage in memory
    pub fn generate_thumbnail(&self, video: &str, time_sec: f32) -> DynamicImage {
        let output = Command::new(&self.ffmpeg_path)
            .args(&[
                "-ss", &time_sec.to_string(), // seek to timestamp
                "-i", video,                  // input file
                "-frames:v", "1",             // only one frame
                "-f", "image2pipe",           // output to stdout
                "-vcodec", "png",             // output format PNG
                "pipe:1",                     // write to stdout
            ])
            .output()
            .unwrap_or_else(|e| panic!("Failed to execute FFmpeg: {}", e));

        if !output.status.success() {
            panic!("FFmpeg failed to generate thumbnail for video {}: {}", video, String::from_utf8_lossy(&output.stderr));
        }

        load_from_memory(&output.stdout)
            .unwrap_or_else(|e| panic!("Failed to decode image from FFmpeg output: {}", e))
    }

    /// Probe video metadata
    pub fn probe_video(&self, video: &str) -> String {
        let output = Command::new(&self.ffprobe_path)
            .args(&[
                "-v", "error",
                "-show_entries", "format=duration:stream=codec_name,width,height",
                "-of", "default=noprint_wrappers=1",
                video,
            ])
            .output()
            .unwrap_or_else(|e| panic!("Failed to execute FFprobe: {}", e));

        if !output.status.success() {
            panic!("FFprobe error: {}", String::from_utf8_lossy(&output.stderr));
        }

        String::from_utf8_lossy(&output.stdout).to_string()
    }
}

/// Initialize FFmpegHandler and log FFmpeg help to the console
pub fn ffmpeg_init(handle: &AppHandle) -> FFmpegHandler {
    let resource_dir = handle.path()
        .resource_dir()
        .unwrap_or_else(|_| panic!("Failed to get Tauri resource directory"));
    let handler = FFmpegHandler::new(&resource_dir);
    handler
}
