use std::{fs, path::PathBuf};
use tauri::{AppHandle, Manager};

pub mod home;
pub mod layouts;
pub mod thumbs;

pub use home::{load_home_cache, save_home_cache, HomeCache, SharedHomeCache};
pub use layouts::{
    fetch_layout_settings, load_layout_cache, save_layout_cache, update_layout_settings,
    LayoutCache, SharedLayoutCache,
};
pub use thumbs::{get_thumb, hash_path, open_thumb_db, prune_thumbs, set_thumb};

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
