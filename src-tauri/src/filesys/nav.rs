use base64::engine::GeneralPurpose;
use base64::Engine;
use image::ImageReader;
use jwalk::WalkDir;
use serde::{Deserialize, Serialize};
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::{fs, path::Component};
use tauri::{AppHandle, Manager, State};

use crate::util::caches::{get_thumb, hash_path, open_thumb_db, set_thumb, SharedHomeCache};
use crate::util::ffutils::ffmpeg_init;

/// Represents a single file or directory entry.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileItem {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileItemWithThumbnail {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: Option<u64>,
    pub thumbnail: Option<String>, // base64 PNG data for images, None for non-images
}

/// Represents a folder and its children recursively.
#[derive(Serialize)]
pub struct FileNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub children: Option<Vec<FileNode>>,
}

/// Adds an entry to the "recent" list when a file/folder is accessed.
/// Files are saved as FileItemWithThumbnail with cached or generated thumbnail.
/// Directories remain as FileItem.
/// Automatically deduplicates and caps (50 files / 12 dirs).
pub async fn register_recent_access(
    handle: &AppHandle,
    state: &State<'_, SharedHomeCache>,
    path: String,
) -> Result<(), String> {
    let shared_cache = state.inner();
    let path_obj = Path::new(&path);

    // Validate target
    if !path_obj.exists() {
        return Err(format!("Path does not exist: {}", path));
    }

    let metadata = fs::metadata(&path)
        .map_err(|e| format!("Failed to access metadata for {}: {}", path, e))?;
    let is_dir = metadata.is_dir();
    let size = if !is_dir { Some(metadata.len()) } else { None };
    let name = path_obj
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // Handle directories (simple)
    if is_dir {
        let item = FileItem {
            name,
            path: path.clone(),
            is_dir: true,
            size: None,
        };
        shared_cache.push_recent_dir(item).await;
    } else {
        // Handle files with thumbnail caching
        let ext = path_obj
            .extension()
            .map(|s| s.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        let encoder = GeneralPurpose::new(
            &base64::alphabet::STANDARD,
            base64::engine::general_purpose::PAD,
        );
        let conn = open_thumb_db(handle).map_err(|e| format!("Failed to open thumb DB: {}", e))?;
        let hash = hash_path(&path);
        let mtime = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let thumbnail = if let Ok(Some((thumb_bytes, _, _))) = get_thumb(&conn, hash, mtime) {
            Some(encoder.encode(&thumb_bytes))
        } else if ["png", "jpg", "jpeg", "gif", "bmp"].contains(&ext.as_str()) {
            match fs::read(&path) {
                Ok(bytes) => {
                    if let Ok(reader) = ImageReader::new(Cursor::new(&bytes)).with_guessed_format()
                    {
                        if let Ok(img) = reader.decode() {
                            let thumb = img.resize(128, 128, image::imageops::FilterType::Nearest);
                            let mut buf = Vec::new();
                            if thumb
                                .write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Jpeg)
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
            let ffmpeg_handler = ffmpeg_init(handle);
            match std::panic::catch_unwind(|| {
                let img = ffmpeg_handler.generate_thumbnail(&path, 1.0);
                let thumb = img.resize(128, 128, image::imageops::FilterType::Nearest);
                let mut buf = Vec::new();
                thumb
                    .write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Jpeg)
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

        let item = FileItemWithThumbnail {
            name,
            path: path.clone(),
            is_dir: false,
            size,
            thumbnail,
        };

        shared_cache.push_recent_file(item).await;
    }

    // Persist to disk
    shared_cache.save(handle).await;

    Ok(())
}

#[tauri::command]
pub async fn open_from_path(
    handle: AppHandle,
    state: State<'_, SharedHomeCache>,
    path: &str,
) -> Result<(), String> {
    if path.is_empty() {
        return Err("Path is empty".into());
    }
    let path = PathBuf::from(path);
    if !path.exists() {
        return Err(format!("Path does not exist: {}", path.display()));
    }
    opener::open(&path).map_err(|e| format!("Failed to open {}: {}", path.display(), e))?;
    register_recent_access(&handle, &state, path.to_string_lossy().to_string())
        .await
        .map_err(|e| format!("Failed to register recent access: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn get_tree_from_root(target_path: &str) -> Result<FileNode, String> {
    // --- Normalize and canonicalize base path ---
    let mut normalized = target_path.to_string();

    if normalized.is_empty() {
        normalized = if cfg!(windows) {
            "C:\\".into()
        } else {
            "/".into()
        };
    } else if cfg!(windows) {
        if normalized.ends_with(':') {
            normalized.push('\\');
        }
        normalized = normalized.replace('/', "\\");
    }

    // Canonicalize with dunce (removes \\?\ and resolves symlinks)
    let target = dunce::canonicalize(&normalized).unwrap_or_else(|_| PathBuf::from(&normalized));

    // --- Determine drive root (C:\, D:\, etc.) ---
    #[cfg(windows)]
    let root_path: PathBuf = {
        if let Some(Component::Prefix(prefix)) = target.components().next() {
            let drive = prefix.as_os_str().to_string_lossy();
            PathBuf::from(format!("{}\\", drive.trim_end_matches('\\')))
        } else {
            PathBuf::from("C:\\")
        }
    };

    #[cfg(not(windows))]
    let root_path = PathBuf::from("/");

    let root_path = dunce::canonicalize(&root_path).unwrap_or(root_path.clone());

    // --- Compute relative path from root to target ---
    let relative = target.strip_prefix(&root_path).unwrap_or(target.as_path());
    let components: Vec<_> = relative.components().collect();

    // --- Helper to clean \\?\ prefixes ---
    fn normalize_path(p: &Path) -> String {
        let s = p.to_string_lossy();
        s.strip_prefix(r"\\?\").unwrap_or(&s).to_string()
    }

    // --- Recursive tree builder ---
    fn build_tree_along_path(path: PathBuf, remaining: &[Component]) -> FileNode {
        let path_str = normalize_path(&path);
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| path_str.clone());

        let mut children = Vec::new();

        for entry in WalkDir::new(&path)
            .max_depth(1)
            .skip_hidden(false)
            .into_iter()
            .flatten()
        {
            if entry.path() == path {
                continue;
            }

            let is_dir = entry.file_type().is_dir();
            let child_path = entry.path();
            let child_name = entry.file_name().to_string_lossy().to_string();

            children.push(FileNode {
                name: child_name,
                path: normalize_path(&child_path),
                is_dir,
                children: if is_dir { Some(Vec::new()) } else { None },
            });
        }

        // Sort: directories first, then alphabetically
        children.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });

        // Recurse along target subpath
        if let Some((first, rest)) = remaining.split_first() {
            let next_name = first.as_os_str().to_string_lossy();
            for child in &mut children {
                if child.name.eq_ignore_ascii_case(&next_name) {
                    let mut next_path = path.clone();
                    next_path.push(&child.name);
                    *child = build_tree_along_path(next_path, rest);
                    break;
                }
            }
        }

        FileNode {
            name,
            path: path_str,
            is_dir: true,
            children: Some(children),
        }
    }

    Ok(build_tree_along_path(root_path, &components))
}

/// Returns immediate directory contents (non-recursive)

#[tauri::command]

pub fn list_directory_contents(path: &str) -> Result<Vec<FileItem>, String> {
    let entries = fs::read_dir(path).map_err(|e| format!("Failed to read directory: {}", e))?;

    let mut items: Vec<FileItem> = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;

            let metadata = entry.metadata().ok()?;

            let is_dir = metadata.is_dir();

            let size = if !is_dir { Some(metadata.len()) } else { None };

            let name = entry.file_name().to_string_lossy().to_string();

            let path = entry.path().to_string_lossy().to_string();

            Some(FileItem {
                name,
                path,
                is_dir,
                size,
            })
        })
        .collect();

    items.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,

        (false, true) => std::cmp::Ordering::Greater,

        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    Ok(items)
}

#[tauri::command]
pub fn resolve_user(handle: AppHandle) -> Result<String, String> {
    handle
        .path()
        .home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|_| "Failed to resolve home directory".to_string())
}
