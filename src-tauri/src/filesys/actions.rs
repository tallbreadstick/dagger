// src/tauri/actions.rs
use std::future::Future;
use std::path::Path;
use std::pin::Pin;
use tokio::fs;

/// Create a new empty file asynchronously
#[tauri::command]
pub async fn create_new_file(path: String) -> Result<(), String> {
    fs::File::create(&path)
        .await
        .map(|_| ())
        .map_err(|e| format!("Failed to create file: {}", e))
}

/// Create a new directory asynchronously
#[tauri::command]
pub async fn create_new_directory(path: String) -> Result<(), String> {
    fs::create_dir_all(&path)
        .await
        .map_err(|e| format!("Failed to create directory: {}", e))
}

/// Async recursive directory copy with boxed future
fn copy_dir_recursive<'a>(
    src: &'a Path,
    dst: &'a Path,
) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + 'a>> {
    Box::pin(async move {
        fs::create_dir_all(dst)
            .await
            .map_err(|e| format!("Failed to create destination dir: {}", e))?;

        let mut entries = fs::read_dir(src)
            .await
            .map_err(|e| format!("Failed to read directory {}: {}", src.display(), e))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| format!("Failed to read entry: {}", e))?
        {
            let file_type = entry
                .file_type()
                .await
                .map_err(|e| format!("Failed to get file type: {}", e))?;
            let dest_path = dst.join(entry.file_name());

            if file_type.is_dir() {
                copy_dir_recursive(&entry.path(), &dest_path).await?;
            } else if file_type.is_file() {
                fs::copy(entry.path(), &dest_path).await.map_err(|e| {
                    format!("Failed to copy file {}: {}", entry.path().display(), e)
                })?;
            }
        }

        Ok(())
    })
}

/// Copy a file or directory asynchronously
#[tauri::command]
pub async fn copy_item(src: String, dest: String) -> Result<(), String> {
    let src_path = Path::new(&src);
    let dest_path = Path::new(&dest);

    if !src_path.exists() {
        return Err("Source path does not exist".into());
    }

    if src_path.is_file() {
        fs::copy(src_path, dest_path)
            .await
            .map(|_| ())
            .map_err(|e| format!("Failed to copy file: {}", e))
    } else if src_path.is_dir() {
        copy_dir_recursive(src_path, dest_path).await
    } else {
        Err("Source path is neither file nor directory".into())
    }
}

/// Move a file or directory asynchronously
#[tauri::command]
pub async fn move_item(src: String, dest: String) -> Result<(), String> {
    let src_path = Path::new(&src);
    let dest_path = Path::new(&dest);

    if !src_path.exists() {
        return Err("Source path does not exist".into());
    }

    fs::rename(src_path, dest_path)
        .await
        .map_err(|e| format!("Failed to move item: {}", e))
}

/// Async recursive delete with boxed future
fn delete_dir_recursive(
    path: &Path,
) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send + '_>> {
    Box::pin(async move {
        let mut entries = fs::read_dir(path)
            .await
            .map_err(|e| format!("Failed to read directory {}: {}", path.display(), e))?;

        while let Some(entry) = entries
            .next_entry()
            .await
            .map_err(|e| format!("Failed to read entry: {}", e))?
        {
            let file_type = entry
                .file_type()
                .await
                .map_err(|e| format!("Failed to get file type: {}", e))?;
            let entry_path = entry.path();

            if file_type.is_dir() {
                delete_dir_recursive(&entry_path).await?;
                fs::remove_dir(&entry_path)
                    .await
                    .map_err(|e| format!("Failed to remove dir {}: {}", entry_path.display(), e))?;
            } else if file_type.is_file() {
                fs::remove_file(&entry_path).await.map_err(|e| {
                    format!("Failed to remove file {}: {}", entry_path.display(), e)
                })?;
            }
        }

        Ok(())
    })
}

/// Delete a file or directory asynchronously
#[tauri::command]
pub async fn delete_item(path: String) -> Result<(), String> {
    let path = Path::new(&path);

    if !path.exists() {
        return Err("Path does not exist".into());
    }

    if path.is_file() {
        fs::remove_file(path)
            .await
            .map_err(|e| format!("Failed to delete file: {}", e))
    } else if path.is_dir() {
        delete_dir_recursive(path).await?;
        fs::remove_dir(path)
            .await
            .map_err(|e| format!("Failed to delete directory: {}", e))
    } else {
        Err("Path is neither file nor directory".into())
    }
}

/// Paste items based on frontend-provided paths
#[tauri::command]
pub async fn paste_item_from_paths(dest: String, paths: Vec<String>) -> Result<(), String> {
    let dest_path = Path::new(&dest);

    if !dest_path.exists() || !dest_path.is_dir() {
        return Err("Destination path does not exist or is not a directory".into());
    }

    for path_str in paths {
        let src_path = Path::new(&path_str);
        if !src_path.exists() {
            continue;
        }

        let dest_file_path = dest_path.join(src_path.file_name().unwrap());

        if src_path.is_file() {
            fs::copy(src_path, &dest_file_path)
                .await
                .map(|_| ())
                .map_err(|e| format!("Failed to copy file {}: {}", src_path.display(), e))?;
        } else if src_path.is_dir() {
            copy_dir_recursive(src_path, &dest_file_path).await?;
        }
    }

    Ok(())
}

/// Rename a file or directory asynchronously
#[tauri::command]
pub async fn rename_item(path: String, new_name: String) -> Result<(), String> {
    let path = Path::new(&path);

    if !path.exists() {
        return Err("Path does not exist".into());
    }

    let parent = path.parent().ok_or("Failed to get parent directory")?;
    let new_path = parent.join(new_name);

    fs::rename(path, new_path)
        .await
        .map_err(|e| format!("Failed to rename item: {}", e))
}
