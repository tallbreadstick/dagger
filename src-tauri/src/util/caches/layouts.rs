use serde::{Serialize, Deserialize};
use std::{fs, io::Read, path::PathBuf, sync::Arc};
use tauri::{AppHandle, State};
use tokio::sync::RwLock;

use crate::util::caches::get_cache_dir;

// ===============================
// LayoutCache Structure
// ===============================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutCache {
    // Sorting
    pub sort_key: SortKey,
    pub ascending: bool,

    // Viewing
    pub view_mode: ViewMode,
    pub show_hidden: bool,
    pub show_extensions: bool,
    pub icon_size: IconSize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortKey {
    Name,
    Size,
    Filetype,
    DateModified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ViewMode {
    Grid,
    List,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IconSize {
    Small,
    Medium,
    Large
}

impl Default for LayoutCache {
    fn default() -> Self {
        Self {
            sort_key: SortKey::Name,
            ascending: true,
            view_mode: ViewMode::Grid,
            show_hidden: false,
            show_extensions: true,
            icon_size: IconSize::Small,
        }
    }
}

// ===============================
// SharedLayoutCache Wrapper
// ===============================

#[derive(Clone, Default)]
pub struct SharedLayoutCache(pub Arc<RwLock<LayoutCache>>);

impl SharedLayoutCache {
    pub fn new(cache: LayoutCache) -> Self {
        Self(Arc::new(RwLock::new(cache)))
    }

    /// Load layout cache from disk (or default)
    pub async fn load(handle: &AppHandle) -> Self {
        let cache = load_layout_cache(handle);
        Self::new(cache)
    }

    /// Save current layout cache state to disk
    pub async fn save(&self, handle: &AppHandle) {
        let cache = self.0.read().await;
        save_layout_cache(handle, &cache);
    }
}

// ===============================
// Internal Helpers
// ===============================

fn get_layout_cache_path(handle: &AppHandle) -> PathBuf {
    let mut path = get_cache_dir(handle);
    path.push("layout.json");
    path
}

/// Loads layout settings from disk, or defaults if missing
pub fn load_layout_cache(handle: &AppHandle) -> LayoutCache {
    let path = get_layout_cache_path(handle);

    if let Ok(mut file) = fs::File::open(&path) {
        let mut data = String::new();
        if file.read_to_string(&mut data).is_ok() {
            if let Ok(cache) = serde_json::from_str::<LayoutCache>(&data) {
                return cache;
            }
        }
    }

    LayoutCache::default()
}

/// Saves layout settings to disk atomically
pub fn save_layout_cache(handle: &AppHandle, cache: &LayoutCache) {
    let path = get_layout_cache_path(handle);
    let tmp_path = path.with_extension("tmp");

    let serialized = serde_json::to_string_pretty(cache).unwrap();

    fs::write(&tmp_path, serialized)
        .unwrap_or_else(|_| panic!("Failed to write temp layout cache"));
    fs::rename(&tmp_path, &path)
        .unwrap_or_else(|_| panic!("Failed to rename temp layout cache"));
}

#[tauri::command]
pub async fn fetch_layout_settings(
    layout_cache: State<'_, SharedLayoutCache>,
) -> Result<LayoutCache, String> {
    let cache = layout_cache.0.read().await.clone();
    Ok(cache)
}

#[tauri::command]
pub async fn update_layout_settings(
    handle: AppHandle,
    layout_cache: State<'_, SharedLayoutCache>,
    new_settings: LayoutCache,
) -> Result<(), String> {
    {
        let mut cache = layout_cache.0.write().await;
        *cache = new_settings.clone();
    }

    // persist changes
    layout_cache.save(&handle).await;
    Ok(())
}