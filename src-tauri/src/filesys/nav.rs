use std::fs;
use serde::Serialize;

#[derive(Serialize)]
pub struct FileItem {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: Option<u64>,
}

#[derive(Serialize)]
pub struct FileNode {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub children: Option<Vec<FileNode>>,
}

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

            Some(FileItem {
                name,
                path,
                is_dir,
                size,
            })
        })
        .collect();

    // Sort alphabetically with directories first
    items.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });

    Ok(items)
}

#[tauri::command]
pub fn get_directory_tree(path: &str, depth: usize) -> Result<FileNode, String> {
    fn build_tree(path: &str, depth: usize) -> Result<FileNode, String> {
        let metadata = fs::metadata(path)
            .map_err(|e| format!("Failed to read metadata for {}: {}", path, e))?;
        let is_dir = metadata.is_dir();
        let name = std::path::Path::new(path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let children = if is_dir && depth > 0 {
            let entries = fs::read_dir(path)
                .map_err(|e| format!("Failed to read directory {}: {}", path, e))?;
            let mut nodes: Vec<FileNode> = entries
                .filter_map(|entry| {
                    let entry = entry.ok()?;
                    let path = entry.path().to_string_lossy().to_string();
                    Some(build_tree(&path, depth - 1).ok()?)
                })
                .collect();

            // Sort directories first
            nodes.sort_by(|a, b| match (a.is_dir, b.is_dir) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
            });

            Some(nodes)
        } else {
            None
        };

        Ok(FileNode {
            name,
            path: path.to_string(),
            is_dir,
            children,
        })
    }

    build_tree(path, depth)
}