use std::{fs, path::Component};
use std::path::{Path, PathBuf};
use jwalk::WalkDir;
use serde::{Serialize, Deserialize};
use tauri::{AppHandle, Manager};

use crate::util::caches::{load_home_cache, save_home_cache, SharedHomeCache};

/// Represents a single file or directory entry.
#[derive(Serialize, Deserialize, Clone, Debug)]
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
/// Automatically de-duplicates and caps the length (50 files / 12 dirs).
#[tauri::command]
pub async fn register_recent_access(
    handle: AppHandle,
    state: tauri::State<'_, SharedHomeCache>,
    path: String,
) -> Result<(), String> {
    let shared_cache = state.inner();
    // Fetch metadata
    let metadata = fs::metadata(&path)
        .map_err(|e| format!("Failed to access metadata for {}: {}", path, e))?;
    let is_dir = metadata.is_dir();
    let size = if !is_dir { Some(metadata.len()) } else { None };
    let name = Path::new(&path)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let item = FileItem {
        name,
        path: path.clone(),
        is_dir,
        size,
    };

    // Push into appropriate recent list using the async home cache
    if is_dir {
        shared_cache.push_recent_dir(item).await;
    } else {
        shared_cache.push_recent_file(item).await;
    }

    // Persist to disk
    shared_cache.save(&handle).await;

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

#[tauri::command]
pub fn get_tree_from_root(target_path: &str) -> Result<FileNode, String> {
    // --- Normalize and canonicalize base path ---
    let mut normalized = target_path.to_string();

    if normalized.is_empty() {
        normalized = if cfg!(windows) { "C:\\".into() } else { "/".into() };
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