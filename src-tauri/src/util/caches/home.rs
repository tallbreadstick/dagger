use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::{collections::VecDeque, fs, io::Read, path::PathBuf};
use tauri::AppHandle;
use tokio::sync::RwLock;

use crate::util::caches::get_cache_dir;
use crate::{filesys::nav::FileItem};

const MAX_RECENT_FILES: usize = 50;
const MAX_RECENT_DIRS: usize = 12;

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct HomeCache {
    pub recent_files: VecDeque<FileItem>,
    pub recent_dirs: VecDeque<FileItem>,
    pub pinned_items: Vec<FileItem>,
}

#[derive(Clone, Default)]
pub struct SharedHomeCache(Arc<RwLock<HomeCache>>);

impl SharedHomeCache {
    pub fn new(cache: HomeCache) -> Self {
        Self(Arc::new(RwLock::new(cache)))
    }

    pub async fn load(handle: &AppHandle) -> Self {
        let cache = load_home_cache(handle);
        Self::new(cache)
    }

    pub async fn save(&self, handle: &AppHandle) {
        let cache = self.0.read().await;
        save_home_cache(handle, &cache);
    }

    /// Add a recent file, deduplicate, and cap the deque
    pub async fn push_recent_file(&self, item: FileItem) {
        let mut cache = self.0.write().await;
        cache.recent_files.retain(|x| x.path != item.path);
        cache.recent_files.push_front(item);
        while cache.recent_files.len() > MAX_RECENT_FILES {
            cache.recent_files.pop_back();
        }
    }

    /// Add a recent directory, deduplicate, and cap the deque
    pub async fn push_recent_dir(&self, item: FileItem) {
        let mut cache = self.0.write().await;
        cache.recent_dirs.retain(|x| x.path != item.path);
        cache.recent_dirs.push_front(item);
        while cache.recent_dirs.len() > MAX_RECENT_DIRS {
            cache.recent_dirs.pop_back();
        }
    }
}

/// Location of the home cache JSON file
fn get_home_cache_path(handle: &AppHandle) -> PathBuf {
    let mut path = get_cache_dir(handle);
    path.push("recent.json");
    path
}

/// Loads the cached recent items from disk or creates an empty cache if missing
pub fn load_home_cache(handle: &AppHandle) -> HomeCache {
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

/// Saves the home cache to disk atomically
pub fn save_home_cache(handle: &AppHandle, cache: &HomeCache) {
    let path = get_home_cache_path(handle);
    let tmp_path = path.with_extension("tmp");

    let serialized = serde_json::to_string_pretty(cache).unwrap();

    fs::write(&tmp_path, serialized).unwrap_or_else(|_| panic!("Failed to write temp home cache"));
    fs::rename(&tmp_path, &path).unwrap_or_else(|_| panic!("Failed to rename temp cache file"));
}
