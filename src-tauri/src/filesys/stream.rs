use std::{path::Path, sync::{atomic::{AtomicBool, AtomicU64, Ordering}, Arc}};
use image::io::Reader as ImageReader;
use image::imageops::FilterType;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::RwLock;

pub struct StreamState {
    pub current_id: AtomicU64,
    pub cancelled: AtomicBool,
}

impl Default for StreamState {
    fn default() -> Self {
        Self {
            current_id: AtomicU64::new(0),
            cancelled: AtomicBool::new(false),
        }
    }
}

#[tauri::command]
pub async fn stream_directory_contents(
    handle: AppHandle,
    state: State<'_, Arc<StreamState>>,
    path: String,
    sort_key: String,
    ascending: bool,
    request_id: u64,
) -> Result<(), String> {
    // set active id + un-cancel
    state.current_id.store(request_id, Ordering::Relaxed);
    state.cancelled.store(false, Ordering::Relaxed);

    let mut entries = tokio::fs::read_dir(&path).await.map_err(|e| e.to_string())?;
    let mut items = Vec::new();

    // Collect entries (quick IO awaits here)
    while let Ok(Some(entry)) = entries.next_entry().await {
        // cheap atomic check — no await
        if state.cancelled.load(Ordering::Relaxed)
            || state.current_id.load(Ordering::Relaxed) != request_id
        {
            println!("Cancelled while collecting for {}", path);
            return Ok(());
        }

        if let Ok(meta) = entry.metadata().await {
            let is_dir = meta.is_dir();
            let size = if !is_dir { Some(meta.len()) } else { None };
            let name = entry.file_name().to_string_lossy().to_string();
            let path_str = entry.path().to_string_lossy().to_string();
            let filetype = entry.path().extension()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            let modified = meta.modified().ok();

            items.push((name, path_str, is_dir, size, filetype, modified));
        }
    }

    // sorting as before...
    items.sort_by(|a, b| {
        if a.2 != b.2 { return b.2.cmp(&a.2); }
        let ord = match sort_key.as_str() {
            "name" => a.0.to_lowercase().cmp(&b.0.to_lowercase()),
            "size" => a.3.cmp(&b.3),
            "filetype" => a.4.to_lowercase().cmp(&b.4.to_lowercase()),
            "date_modified" => a.5.cmp(&b.5),
            _ => a.0.to_lowercase().cmp(&b.0.to_lowercase()),
        };
        if ascending { ord } else { ord.reverse() }
    });

    for (name, path_str, is_dir, size, filetype, _modified) in items {
        // cheap atomic check before heavy work
        if state.cancelled.load(Ordering::Relaxed)
            || state.current_id.load(Ordering::Relaxed) != request_id
        {
            println!("Cancelled mid-stream for {}", path);
            return Ok(());
        }

        let ext = Path::new(&path_str)
            .extension()
            .map(|s| s.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        // Only perform expensive read/decode if still current
        let thumbnail = if !is_dir && ["png", "jpg", "jpeg", "gif", "bmp"].contains(&ext.as_str()) {
            // re-check before reading
            if state.cancelled.load(Ordering::Relaxed)
                || state.current_id.load(Ordering::Relaxed) != request_id
            {
                None
            } else {
                match tokio::fs::read(&path_str).await {
                    Ok(bytes) => {
                        if state.cancelled.load(Ordering::Relaxed)
                            || state.current_id.load(Ordering::Relaxed) != request_id
                        {
                            None
                        } else if let Ok(reader) = ImageReader::new(std::io::Cursor::new(&bytes)).with_guessed_format() {
                            match reader.decode() {
                                Ok(img) => {
                                    // run thumbnail quickly
                                    let thumb = img.thumbnail(128, 128);
                                    let mut buf = Vec::new();
                                    if thumb.write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Jpeg).is_ok() {
                                        Some(base64::encode(&buf))
                                    } else { None }
                                }
                                Err(_) => None,
                            }
                        } else { None }
                    }
                    Err(_) => None,
                }
            }
        } else {
            None
        };

        // final cheap check right before emit
        if state.cancelled.load(Ordering::Relaxed)
            || state.current_id.load(Ordering::Relaxed) != request_id
        {
            println!("Cancelled right before emit for {}", path);
            return Ok(());
        }

        // include request_id in the payload
        let _ = handle.emit("file-chunk", serde_json::json!({
            "request_id": request_id,
            "name": name,
            "path": path_str,
            "is_dir": is_dir,
            "size": size,
            "filetype": filetype,
            "thumbnail": thumbnail,
            "date_modified": _modified
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs()),
        }));
    }

    // completion only if still current
    if !state.cancelled.load(Ordering::Relaxed)
        && state.current_id.load(Ordering::Relaxed) == request_id
    {
        let _ = handle.emit("file-chunk-complete", serde_json::json!({ "request_id": request_id, "path": path }));
    }

    Ok(())
}

// #[tauri::command]
// pub async fn cancel_current_stream(state: State<'_, Arc<RwLock<StreamState>>>) -> Result<(), String> {
//     let s = state.read().await;
//     s.cancelled.store(true, Ordering::Relaxed);
//     Ok(())
// }
