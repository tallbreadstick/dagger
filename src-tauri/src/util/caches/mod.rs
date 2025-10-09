use std::{fs, path::PathBuf};
use tauri::{AppHandle, Manager};

pub mod home;
pub mod thumbs;

pub use home::{load_home_cache, save_home_cache, HomeCache, SharedHomeCache};
pub use thumbs::{open_thumb_db, hash_path, get_thumb, set_thumb, prune_thumbs};

/// Location of the app cache directory
fn get_cache_dir(handle: &AppHandle) -> PathBuf {
    let mut dir = handle
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| panic!("Failed to get app data dir"));
    dir.push("caches");
    fs::create_dir_all(&dir).unwrap_or_else(|_| panic!("Failed to create cache directory"));
    dir
}
