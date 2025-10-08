use std::fs;
use std::path::Path;
use serde::{Serialize, Deserialize};
use tauri::{AppHandle, Manager};

use crate::util::caches::{load_home_cache, save_home_cache};

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
pub fn resolve_user(handle: AppHandle) -> Result<String, String> {
    handle.path()
        .home_dir()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|_| "Failed to resolve home directory".to_string())
}