use std::{collections::VecDeque, fs, io::Read, path::PathBuf};
use rusqlite::{params, Connection, OptionalExtension, Result};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tokio::sync::RwLock;
use xxhash_rust::xxh3::xxh3_64;
use std::sync::Arc;

use crate::filesys::nav::FileItem;

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

/// Location of the app cache directory
fn get_cache_dir(handle: &AppHandle) -> PathBuf {
    let dir = handle.path().app_data_dir().unwrap_or_else(|_| panic!("Failed to get app data dir"));
    fs::create_dir_all(&dir).unwrap_or_else(|_| panic!("Failed to create cache directory"));
    dir
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

/// Location of the thumbnail cache DB at `%APPDATA%\dagger\caches\thumbs.db`
fn get_thumb_db_path(handle: &AppHandle) -> PathBuf {
    let mut path = get_cache_dir(handle);
    path.push("caches");
    fs::create_dir_all(&path).ok();
    path.push("thumbs.db");
    path
}

/// Opens (or creates) the SQLite thumbnail cache.
pub fn open_thumb_db(handle: &AppHandle) -> Result<Connection> {
    let path = get_thumb_db_path(handle);
    let conn = Connection::open(path)?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS thumbs (
            hash INTEGER PRIMARY KEY,
            mtime INTEGER NOT NULL,
            size INTEGER,
            filetype TEXT,
            thumb BLOB NOT NULL
        );
        PRAGMA journal_mode=WAL;
        PRAGMA synchronous=NORMAL;"
    )?;

    Ok(conn)
}

/// Compute a 64-bit xxHash of a file path.
pub fn hash_path(path: &str) -> u64 {
    xxh3_64(path.as_bytes())
}

/// Reads a thumbnail and optional metadata from the cache.
/// Returns None if missing or stale.
pub fn get_thumb(conn: &Connection, hash: u64, mtime: i64) -> Result<Option<(Vec<u8>, Option<i64>, Option<String>)>> {
    let row: Option<(i64, i64, Option<String>, Vec<u8>)> = conn.query_row(
        "SELECT mtime, size, filetype, thumb FROM thumbs WHERE hash = ?1",
        [hash],
        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
    ).optional()?;

    if let Some((cached_mtime, size, filetype, thumb)) = row {
        if cached_mtime == mtime {
            return Ok(Some((thumb, Some(size), filetype)));
        }
    }
    Ok(None)
}

/// Inserts or updates a thumbnail and optional metadata in the cache.
pub fn set_thumb(conn: &Connection, hash: u64, mtime: i64, size: Option<i64>, filetype: Option<&str>, thumb: &[u8]) -> Result<()> {
    conn.execute(
        "INSERT INTO thumbs (hash, mtime, size, filetype, thumb) 
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(hash) DO UPDATE SET 
             mtime = excluded.mtime,
             size = excluded.size,
             filetype = excluded.filetype,
             thumb = excluded.thumb;",
        params![hash, mtime, size, filetype, thumb]
    )?;
    Ok(())
}

/// Optional: remove thumbnails older than a certain mtime (cleanup).
pub fn prune_thumbs(conn: &Connection, min_mtime: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM thumbs WHERE mtime < ?1;",
        [min_mtime]
    )?;
    Ok(())
}
