use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use serde::{Serialize, Deserialize};
use serde_json::json;
use tauri::{AppHandle, Manager, Emitter};

/// Represents a single file or directory entry.
#[derive(Serialize, Deserialize, Clone)]
pub struct FileItem {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: Option<u64>,
}

#[derive(Serialize, Deserialize, Clone)]
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

/// Represents the user's cached "Home" view.
#[derive(Serialize, Deserialize, Default)]
pub struct HomeCache {
    pub recent_files: Vec<FileItem>,
    pub recent_dirs: Vec<FileItem>,
}

/// Location of the app cache directory in AppData.
fn get_cache_dir(handle: &AppHandle) -> PathBuf {
    let mut dir = handle.path()
        .app_data_dir()
        .unwrap();
    dir.push("Dagger");
    fs::create_dir_all(&dir).ok();
    dir
}

/// Location of the home cache JSON file.
fn get_home_cache_path(handle: &AppHandle) -> PathBuf {
    let mut path = get_cache_dir(handle);
    path.push("recent.json");
    path
}

/// Loads the cached recent items from disk, or creates an empty cache if missing.
fn load_home_cache(handle: &AppHandle) -> HomeCache {
    let path = get_home_cache_path(handle);
    if let Ok(mut file) = fs::File::open(&path) {
        let mut data = String::new();
        if file.read_to_string(&mut data).is_ok() {
            if let Ok(cache) = serde_json::from_str::<HomeCache>(&data) {
                return cache;
            }
        }
    }
    HomeCache::default()
}

/// Saves the home cache back to disk.
fn save_home_cache(handle: &AppHandle, cache: &HomeCache) {
    let path = get_home_cache_path(handle);
    if let Ok(mut file) = fs::File::create(&path) {
        let _ = file.write_all(serde_json::to_string_pretty(cache).unwrap().as_bytes());
    }
}

/// Adds an entry to the "recent" list when a file/folder is accessed.
/// Automatically de-duplicates and caps the length to 20 entries.
#[tauri::command]
pub fn register_recent_access(handle: AppHandle, path: &str) -> Result<(), String> {
    let metadata = fs::metadata(path)
        .map_err(|e| format!("Failed to access metadata for {}: {}", path, e))?;
    let is_dir = metadata.is_dir();
    let size = if !is_dir { Some(metadata.len()) } else { None };
    let name = Path::new(path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let mut cache = load_home_cache(&handle);

    let item = FileItem {
        name,
        path: path.to_string(),
        is_dir,
        size,
    };

    let target_list = if is_dir { &mut cache.recent_dirs } else { &mut cache.recent_files };

    // Remove duplicates
    target_list.retain(|f| f.path != path);

    // Insert at the top
    target_list.insert(0, item);

    // Cap list size
    if target_list.len() > 20 {
        target_list.truncate(20);
    }

    save_home_cache(&handle, &cache);
    Ok(())
}

/// Returns the current "Home" pseudo-directory contents.
/// This acts like a virtual folder showing recently accessed items.
// #[tauri::command]
// pub fn get_home_directory(handle: &AppHandle) -> Result<Vec<FileItem>, String> {
//     let cache = load_home_cache(&handle);

//     let mut items: Vec<FileItem> = Vec::new();
//     items.extend(cache.recent_dirs);
//     items.extend(cache.recent_files);

//     // Sort so that directories appear first
//     items.sort_by(|a, b| match (a.is_dir, b.is_dir) {
//         (true, false) => std::cmp::Ordering::Less,
//         (false, true) => std::cmp::Ordering::Greater,
//         _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
//     });

//     Ok(items)
// }

/// Helper to read immediate directory contents
// fn read_dir_safe(path: &str) -> Vec<FileNode> {
//     if let Ok(entries) = fs::read_dir(path) {
//         let mut nodes: Vec<FileNode> = entries
//             .filter_map(|entry| {
//                 let entry = entry.ok()?;
//                 let meta = entry.metadata().ok()?;
//                 let is_dir = meta.is_dir();
//                 let name = entry.file_name().to_string_lossy().to_string();
//                 let path = entry.path().to_string_lossy().to_string();
//                 Some(FileNode {
//                     name,
//                     path,
//                     is_dir,
//                     children: if is_dir { Some(vec![]) } else { None },
//                 })
//             })
//             .collect();

//         nodes.sort_by(|a, b| match (a.is_dir, b.is_dir) {
//             (true, false) => std::cmp::Ordering::Less,
//             (false, true) => std::cmp::Ordering::Greater,
//             _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
//         });

//         nodes
//     } else {
//         vec![]
//     }
// }

/// Recursively builds tree along a path from root to target
#[tauri::command]
pub fn get_tree_from_root(target_path: &str) -> Result<FileNode, String> {
    use std::path::{Path, PathBuf, Component};
    use std::fs;

    // Determine the system root
    let root_path = if cfg!(windows) {
        PathBuf::from("C:\\")
    } else {
        PathBuf::from("/") // linux, mac, etc
    };

    // Compute remaining components from root to target
    let target = Path::new(target_path);
    let relative = target.strip_prefix(&root_path).unwrap_or(target);
    let components: Vec<_> = relative.components().collect();

    // Recursive function: only expand the path along target_path
    fn build_tree_along_path(path: PathBuf, remaining: &[Component]) -> FileNode {
        let path_str = path.to_string_lossy().to_string();
        let name = path.file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| path_str.clone());

        let mut children: Vec<FileNode> = Vec::new();

        // Read immediate children for lazy loading
        if let Ok(entries) = fs::read_dir(&path) {
            for entry in entries.flatten() {
                let meta = entry.metadata().ok();
                if let Some(meta) = meta {
                    let is_dir = meta.is_dir();
                    let child_path = entry.path();
                    let child_name = entry.file_name().to_string_lossy().to_string();
                    children.push(FileNode {
                        name: child_name,
                        path: child_path.to_string_lossy().to_string(),
                        is_dir,
                        children: if is_dir { Some(Vec::new()) } else { None }, // lazy
                    });
                }
            }

            // Sort dirs first
            children.sort_by(|a, b| match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            });
        }

        // If there are remaining components, recurse into the next
        if !remaining.is_empty() {
            for child in &mut children {
                if child.name == remaining[0].as_os_str().to_string_lossy() {
                    *child = build_tree_along_path(PathBuf::from(&child.path), &remaining[1..]);
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
    let entries = fs::read_dir(path)
        .map_err(|e| format!("Failed to read directory: {}", e))?;

    let mut items: Vec<FileItem> = entries
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let metadata = entry.metadata().ok()?;
            let is_dir = metadata.is_dir();
            let size = if !is_dir { Some(metadata.len()) } else { None };
            let name = entry.file_name().to_string_lossy().to_string();
            let path = entry.path().to_string_lossy().to_string();
            Some(FileItem { name, path, is_dir, size })
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
pub async fn stream_directory_contents(
    handle: AppHandle,
    path: String,
    sort_key: String,        
    ascending: bool,       
) -> Result<(), String> {
    let entries = tokio::fs::read_dir(&path)
        .await
        .map_err(|e| e.to_string())?;

    let mut items = Vec::new();

    let mut dir_entries = entries;
    while let Ok(Some(entry)) = dir_entries.next_entry().await {
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

    // Sorting (same as before)
    items.sort_by(|a, b| {
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
        let ext = Path::new(&path_str)
            .extension()
            .map(|s| s.to_string_lossy().to_lowercase())
            .unwrap_or_default();

        let thumbnail = if !is_dir && ["png","jpg","jpeg","gif","bmp"].contains(&ext.as_str()) {
            tokio::fs::read(&path_str).await.ok().map(|bytes| base64::encode(&bytes))
        } else { None };

        if let Err(e) = handle.emit("file-chunk", serde_json::json!({
            "name": name,
            "path": path_str,
            "is_dir": is_dir,
            "size": size,
            "filetype": filetype,
            "thumbnail": thumbnail,
            "date_modified": _modified.map(|t| t.duration_since(std::time::UNIX_EPOCH).ok().map(|d| d.as_secs())).flatten(),
        })) {
            eprintln!("Failed to emit file chunk: {}", e);
        }
    }

    handle.emit("file-chunk-complete", serde_json::json!({ "path": path }))
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn resolve_user(handle: AppHandle) -> Result<String, String> {
    handle.path()
        .home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|_| "Failed to resolve home directory".to_string())
}