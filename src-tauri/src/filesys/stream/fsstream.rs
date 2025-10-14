use std::{
    fs,
    path::Path,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, UNIX_EPOCH},
};

use jwalk::WalkDir;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use tauri::{AppHandle, Emitter, State};

use crate::{
    filesys::{nav::register_recent_access, stream::thumbs::get_thumbnail_for_path},
    util::{caches::SharedHomeCache, ffutils::ffmpeg_init},
};

pub struct FileStreamState {
    pub current_id: AtomicU64,
    pub cancelled: AtomicBool,
}

impl Default for FileStreamState {
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
    state: State<'_, Arc<FileStreamState>>,
    pool: State<'_, Arc<rayon::ThreadPool>>,
    cache_state: State<'_, SharedHomeCache>,
    mut path: String,
    sort_key: String,
    ascending: bool,
    show_hidden: bool,
    request_id: u64,
) -> Result<(), String> {
    if path == "Home" {
        return stream_home_directory(handle, cache_state, request_id).await;
    }

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
                "pinned": false
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
        .for_each(|(_name, path_str, _is_dir, _size, _filetype, _modified)| {
            if state.cancelled.load(Ordering::Relaxed)
                || state.current_id.load(Ordering::Relaxed) != request_id
            {
                return;
            }

            // Use the helper to get a base64 thumbnail
            let thumbnail = get_thumbnail_for_path(&handle, &ffmpeg_handler, &path_str);

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

pub async fn stream_home_directory(
    handle: AppHandle,
    cache_state: State<'_, SharedHomeCache>,
    request_id: u64,
) -> Result<(), String> {
    let cache = cache_state.0.read().await;
    let path = "Home".to_string();

    // --- Phase 1: emit metadata for cached files ---
    for item in cache.recent_files.iter() {
        let modified = fs::metadata(&item.path)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs());
        let filetype = item.name.rsplit('.').next().unwrap_or("").to_string();
        let _ = handle.emit(
            "file-metadata",
            serde_json::json!({
                "request_id": request_id,
                "name": item.name,
                "path": item.path,
                "is_dir": false,
                "size": item.size,
                "filetype": filetype,
                "date_modified": modified,
                "pinned": false
            }),
        );
    }

    for item in cache.recent_dirs.iter() {
        let modified = fs::metadata(&item.path)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs());
        let filetype = item.name.rsplit('.').next().unwrap_or("").to_string();
        let _ = handle.emit(
            "file-metadata",
            serde_json::json!({
                "request_id": request_id,
                "name": item.name,
                "path": item.path,
                "is_dir": true,
                "size": item.size,
                "filetype": filetype,
                "date_modified": modified,
                "pinned": false
            }),
        );
    }

    for item in cache.pinned_items.iter() {
        let modified = fs::metadata(&item.path)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_secs());
        let filetype = item.name.rsplit('.').next().unwrap_or("").to_string();
        let _ = handle.emit(
            "file-metadata",
            serde_json::json!({
                "request_id": request_id,
                "name": item.name,
                "path": item.path,
                "is_dir": false,
                "size": item.size,
                "filetype": filetype,
                "date_modified": modified,
                "pinned": true
            }),
        );
    }

    let _ = handle.emit(
        "file-metadata-complete",
        serde_json::json!({
            "request_id": request_id,
            "path": path,
        }),
    );

    // --- Phase 2: emit thumbnails using helper ---
    let ffmpeg_handler = ffmpeg_init(&handle);

    for item in cache.recent_files.iter() {
        if let Some(thumb) = get_thumbnail_for_path(&handle, &ffmpeg_handler, &item.path) {
            let _ = handle.emit(
                "file-thumbnail",
                serde_json::json!({
                    "request_id": request_id,
                    "path": item.path,
                    "thumbnail": thumb,
                }),
            );
        }
    }

    for item in cache.pinned_items.iter() {
        if let Some(thumb) = get_thumbnail_for_path(&handle, &ffmpeg_handler, &item.path) {
            let _ = handle.emit(
                "file-thumbnail",
                serde_json::json!({
                    "request_id": request_id,
                    "path": item.path,
                    "thumbnail": thumb,
                }),
            );
        }
    }

    // --- Phase 3: signal completion ---
    let _ = handle.emit(
        "file-stream-complete",
        serde_json::json!({
            "request_id": request_id,
            "path": path,
        }),
    );

    Ok(())
}
