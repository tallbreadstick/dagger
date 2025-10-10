use crate::{filesys::nav::register_recent_access, util::{
    caches::{get_thumb, hash_path, open_thumb_db, set_thumb, SharedHomeCache},
    ffutils::ffmpeg_init,
}};
use base64::{engine::GeneralPurpose, Engine};
use image::ImageReader;
use jwalk::WalkDir;
use rayon::prelude::*;
use std::{
    path::Path,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};
use tauri::{AppHandle, Emitter, State};

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
    pool: State<'_, Arc<rayon::ThreadPool>>,
    cache_state: State<'_, SharedHomeCache>,
    mut path: String,
    sort_key: String,
    ascending: bool,
    show_hidden: bool,
    request_id: u64,
) -> Result<(), String> {

    if path.is_empty() {
        // Default to root depending on OS
        path = if cfg!(windows) {
            "C:\\".to_string()
        } else {
            "/".to_string()
        };
    } else if cfg!(windows) {
        // Handle "C:" or "D:" without trailing slash
        if path.ends_with(':') {
            path.push('\\');
        }

        // Also normalize forward slashes to backslashes (in case frontend sent them)
        path = path.replace('/', "\\");
    }

    // Verify the directory is valid
    if !Path::new(&path).is_dir() {
        return Err(format!("Path is not a valid directory: {}", path));
    }

    // ✅ Register the access in recents
    if let Err(e) = register_recent_access(&handle, &cache_state, path.clone()).await {
        eprintln!("Failed to register recent access: {}", e);
    }

    state.current_id.store(request_id, Ordering::Relaxed);
    state.cancelled.store(false, Ordering::Relaxed);

    let encoder = GeneralPurpose::new(
        &base64::alphabet::STANDARD,
        base64::engine::general_purpose::PAD,
    );
    let pool_ref = pool.inner().clone();

    let walker = WalkDir::new(&path)
        .max_depth(1)
        .follow_links(false)
        .skip_hidden(!show_hidden)
        .parallelism(jwalk::Parallelism::RayonExistingPool {
            pool: pool_ref,
            busy_timeout: Some(Duration::from_millis(20)),
        });

    // Phase 1: Collect metadata only
    let mut items: Vec<_> = walker
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path() != Path::new(&path))
        .filter_map(|entry| {
            if state.cancelled.load(Ordering::Relaxed)
                || state.current_id.load(Ordering::Relaxed) != request_id
            {
                return None;
            }

            let meta = entry.metadata().ok()?;
            let is_dir = meta.is_dir();
            let size = if !is_dir { Some(meta.len()) } else { None };
            let name = entry.file_name.to_string_lossy().to_string();
            let path_str = entry.path().to_string_lossy().to_string();
            let filetype = entry
                .path()
                .extension()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            let modified = meta.modified().ok();

            Some((name, path_str, is_dir, size, filetype, modified))
        })
        .collect();

    // Sort files
    items.sort_by(|a, b| {
        if a.2 != b.2 {
            return b.2.cmp(&a.2);
        }
        let ord = match sort_key.as_str() {
            "name" => a.0.to_lowercase().cmp(&b.0.to_lowercase()),
            "size" => a.3.cmp(&b.3),
            "filetype" => a.4.to_lowercase().cmp(&b.4.to_lowercase()),
            "date_modified" => a.5.cmp(&b.5),
            _ => a.0.to_lowercase().cmp(&b.0.to_lowercase()),
        };
        if ascending {
            ord
        } else {
            ord.reverse()
        }
    });

    // Phase 1 emit: metadata only
    for (name, path_str, is_dir, size, filetype, modified) in &items {
        if state.cancelled.load(Ordering::Relaxed)
            || state.current_id.load(Ordering::Relaxed) != request_id
        {
            return Ok(());
        }

        let _ = handle.emit(
            "file-metadata",
            serde_json::json!({
                "request_id": request_id,
                "name": name,
                "path": path_str,
                "is_dir": is_dir,
                "size": size,
                "filetype": filetype,
                "date_modified": modified
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs()),
            }),
        );
    }

    let _ = handle.emit(
        "file-metadata-complete",
        serde_json::json!({
            "request_id": request_id,
            "path": path
        }),
    );

    let ffmpeg_handler = ffmpeg_init(&handle);

    // Phase 2: generate/fetch thumbnails in parallel
    items
        .into_par_iter()
        .for_each(|(_name, path_str, is_dir, _size, _filetype, modified)| {
            if state.cancelled.load(Ordering::Relaxed)
                || state.current_id.load(Ordering::Relaxed) != request_id
            {
                return;
            }

            let ext = Path::new(&path_str)
                .extension()
                .map(|s| s.to_string_lossy().to_lowercase())
                .unwrap_or_default();

            let conn = match open_thumb_db(&handle) {
                Ok(c) => c,
                Err(_) => return,
            };

            let hash = hash_path(&path_str);
            let mtime = modified
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            let thumbnail = if let Ok(Some((thumb_bytes, _, _))) = get_thumb(&conn, hash, mtime) {
                Some(encoder.encode(&thumb_bytes))
            } else if is_dir || ["png", "jpg", "jpeg", "gif", "bmp"].contains(&ext.as_str()) {
                // Existing image thumbnail logic
                match std::fs::read(&path_str) {
                    Ok(bytes) => {
                        if let Ok(reader) =
                            ImageReader::new(std::io::Cursor::new(&bytes)).with_guessed_format()
                        {
                            if let Ok(img) = reader.decode() {
                                let thumb =
                                    img.resize(128, 128, image::imageops::FilterType::Nearest);
                                let mut buf = Vec::new();
                                if thumb
                                    .write_to(
                                        &mut std::io::Cursor::new(&mut buf),
                                        image::ImageFormat::Jpeg,
                                    )
                                    .is_ok()
                                {
                                    let _ = set_thumb(
                                        &conn,
                                        hash,
                                        mtime,
                                        Some(bytes.len() as i64),
                                        Some(&ext),
                                        &buf,
                                    );
                                    Some(encoder.encode(&buf))
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    Err(_) => None,
                }
            } else if ["mp4", "mkv", "mov", "avi", "flv"].contains(&ext.as_str()) {
                // Video thumbnail via FFmpeg
                match std::panic::catch_unwind(|| {
                    let img = ffmpeg_handler.generate_thumbnail(&path_str, 1.0); // 1 second timestamp
                    let thumb = img.resize(128, 128, image::imageops::FilterType::Nearest);
                    let mut buf = Vec::new();
                    thumb
                        .write_to(
                            &mut std::io::Cursor::new(&mut buf),
                            image::ImageFormat::Jpeg,
                        )
                        .ok()
                        .map(|_| buf)
                }) {
                    Ok(Some(buf)) => {
                        let _ = set_thumb(&conn, hash, mtime, None, Some(&ext), &buf);
                        Some(encoder.encode(&buf))
                    }
                    _ => None,
                }
            } else {
                None
            };

            let _ = handle.emit(
                "file-thumbnail",
                serde_json::json!({
                    "request_id": request_id,
                    "path": path_str,
                    "thumbnail": thumbnail,
                }),
            );
        });

    // Phase 3: complete
    if !state.cancelled.load(Ordering::Relaxed)
        && state.current_id.load(Ordering::Relaxed) == request_id
    {
        let _ = handle.emit(
            "file-stream-complete",
            serde_json::json!({ "request_id": request_id, "path": path }),
        );
    }

    Ok(())
}