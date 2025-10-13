use crate::{
    filesys::nav::register_recent_access,
    util::{
        caches::{get_thumb, hash_path, open_thumb_db, set_thumb, SharedHomeCache},
        ffutils::{ffmpeg_init, FFmpegHandler},
    },
};
use base64::{engine::GeneralPurpose, Engine};
use image::ImageReader;
use jwalk::WalkDir;
use parselnk::Lnk;
use rayon::prelude::*;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter, State};

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

pub fn resolve_lnk_target(path: &str) -> Option<String> {
    let data = fs::read(path).ok()?;
    let lnk = Lnk::try_from(data).ok()?;

    // Local path
    if let Some(local_base) = &lnk.link_info.local_base_path {
        return Some(local_base.to_string());
    }

    // Skip network path for now (lnk crate limitation)
    // if let Some(_network) = &lnk.link_info.common_network_relative_link { }

    // StringData fields
    if let Some(relative_path) = &lnk.string_data.relative_path {
        return Some(relative_path.to_string_lossy().to_string());
    }
    if let Some(working_dir) = &lnk.string_data.working_dir {
        return Some(working_dir.to_string_lossy().to_string());
    }

    // fallback to original path
    Some(PathBuf::from(path).to_string_lossy().to_string())
}

pub fn get_thumbnail_for_path(
    handle: &tauri::AppHandle,
    ffmpeg: &FFmpegHandler,
    path: &str,
) -> Option<String> {
    // Resolve .lnk shortcuts on Windows
    let resolved_path = if cfg!(windows) && path.ends_with(".lnk") {
        crate::filesys::stream::resolve_lnk_target(path).unwrap_or_else(|| path.to_string())
    } else {
        path.to_string()
    };

    let ext = Path::new(&resolved_path)
        .extension()
        .map(|s| s.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let conn = open_thumb_db(handle).ok()?;
    let hash = hash_path(&resolved_path);
    let mtime = fs::metadata(&resolved_path)
        .ok()
        .and_then(|m| m.modified().ok())
        .map(|d| {
            d.duration_since(std::time::UNIX_EPOCH)
                .ok()
                .map(|d| d.as_secs() as i64)
        })
        .flatten()
        .unwrap_or(0);

    let encoder = GeneralPurpose::new(
        &base64::alphabet::STANDARD,
        base64::engine::general_purpose::PAD,
    );

    // Check cache first
    if let Ok(Some((thumb_bytes, _, _))) = get_thumb(&conn, hash, mtime) {
        return Some(encoder.encode(&thumb_bytes));
    }

    // Directory placeholder: return None or generate folder image if desired
    if Path::new(&resolved_path).is_dir() {
        return None;
    }

    // Image files
    if ["png", "jpg", "jpeg", "gif", "bmp"].contains(&ext.as_str()) {
        if let Ok(bytes) = fs::read(&resolved_path) {
            if let Ok(reader) = ImageReader::new(std::io::Cursor::new(&bytes)).with_guessed_format()
            {
                if let Ok(img) = reader.decode() {
                    let thumb = img.resize(128, 128, image::imageops::FilterType::Nearest);
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
                        return Some(encoder.encode(&buf));
                    }
                }
            }
        }
    }

    // Video files
    if ["mp4", "mkv", "mov", "avi", "flv"].contains(&ext.as_str()) {
        if let Some(buf) = std::panic::catch_unwind(|| {
            let img = ffmpeg.generate_thumbnail(&resolved_path, 1.0); // 1 second timestamp
            let thumb = img.resize(128, 128, image::imageops::FilterType::Nearest);
            let mut buf = Vec::new();
            thumb
                .write_to(
                    &mut std::io::Cursor::new(&mut buf),
                    image::ImageFormat::Jpeg,
                )
                .ok()
                .map(|_| buf)
        })
        .ok()
        .flatten()
        {
            let _ = set_thumb(&conn, hash, mtime, None, Some(&ext), &buf);
            return Some(encoder.encode(&buf));
        }
    }

    // 🔽 Windows-specific fallback: use system shell icon as last resort
    #[cfg(target_os = "windows")]
    {
        use image::{ImageBuffer, Rgba};

        if let Some(raw) = extract_shell_icon(Path::new(&resolved_path)) {
            // Convert BGRA -> RGBA
            let mut rgba = raw.clone();
            for px in rgba.chunks_exact_mut(4) {
                px.swap(0, 2);
            }

            let img: ImageBuffer<Rgba<u8>, _> = ImageBuffer::from_raw(64, 64, rgba)?;
            let thumb = image::DynamicImage::ImageRgba8(img).resize(
                128,
                128,
                image::imageops::FilterType::Nearest,
            );

            let mut buf = Vec::new();
            if thumb
                .write_to(
                    &mut std::io::Cursor::new(&mut buf),
                    image::ImageFormat::Png,
                )
                .is_ok()
            {
                let _ = set_thumb(&conn, hash, mtime, None, Some(&ext), &buf);
                return Some(encoder.encode(&buf));
            }
        }
    }

    // fallback: no thumbnail
    None
}

#[cfg(target_os = "windows")]
fn extract_shell_icon(path: &Path) -> Option<Vec<u8>> {
    use std::{ffi::OsStr, os::windows::ffi::OsStrExt, ptr};
    use windows::core::PCWSTR;
    use windows::Win32::Graphics::Gdi::{GetDC, ReleaseDC};
    use windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES;
    use windows::Win32::{
        Foundation::HWND,
        Graphics::Gdi::{
            CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetDIBits, SelectObject,
            BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
        },
        UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON},
        UI::WindowsAndMessaging::{DestroyIcon, DrawIconEx, DI_NORMAL, HICON},
    };

    // Convert path to UTF-16
    let wpath: Vec<u16> = OsStr::new(path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();

    unsafe {
        // Retrieve icon handle

        use windows::Win32::Graphics::Gdi::DIB_USAGE;
        let mut shinfo = SHFILEINFOW::default();
        let _res = SHGetFileInfoW(
            PCWSTR(wpath.as_ptr()),
            FILE_FLAGS_AND_ATTRIBUTES(0),
            Some(&mut shinfo),
            std::mem::size_of::<SHFILEINFOW>() as u32,
            SHGFI_ICON | SHGFI_LARGEICON,
        );

        let hicon: HICON = shinfo.hIcon;
        if hicon.0.is_null() {
            return None;
        }

        // Get device contexts
        let hdc = GetDC(Some(HWND(ptr::null_mut())));
        let memdc = CreateCompatibleDC(Some(hdc));

        // Create bitmap info
        let mut bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: 64,
                biHeight: -64, // top-down DIB
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };

        // Create DIB section
        let mut bits_ptr: *mut std::ffi::c_void = ptr::null_mut();
        let hbitmap =
            CreateDIBSection(Some(memdc), &bmi, DIB_USAGE(0), &mut bits_ptr, None, 0).unwrap();

        let old = SelectObject(memdc, hbitmap.into());
        let _ = DrawIconEx(memdc, 0, 0, hicon, 64, 64, 0, None, DI_NORMAL);

        // Copy pixels
        let mut buffer = vec![0u8; (64 * 64 * 4) as usize];
        GetDIBits(
            memdc,
            hbitmap,
            0,
            64,
            Some(buffer.as_mut_ptr() as *mut _),
            &mut bmi,
            DIB_USAGE(0),
        );

        // Cleanup
        let _ = SelectObject(memdc, old);
        let _ = DeleteObject(hbitmap.into());
        let _ = DeleteDC(memdc);
        let _ = ReleaseDC(Some(HWND(ptr::null_mut())), hdc);
        let _ = DestroyIcon(hicon);

        Some(buffer)
    }
}
