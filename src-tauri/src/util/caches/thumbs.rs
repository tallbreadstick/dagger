use rusqlite::{params, Connection, OptionalExtension, Result};
use std::{fs, path::PathBuf};
use tauri::AppHandle;
use xxhash_rust::xxh3::xxh3_64;

use crate::util::get_cache_dir;

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
        PRAGMA synchronous=NORMAL;",
    )?;

    Ok(conn)
}

/// Compute a 64-bit xxHash of a file path.
pub fn hash_path(path: &str) -> u64 {
    xxh3_64(path.as_bytes())
}

/// Reads a thumbnail and optional metadata from the cache.
/// Returns None if missing or stale.
pub fn get_thumb(
    conn: &Connection,
    hash: u64,
    mtime: i64,
) -> Result<Option<(Vec<u8>, Option<i64>, Option<String>)>> {
    let row: Option<(i64, i64, Option<String>, Vec<u8>)> = conn
        .query_row(
            "SELECT mtime, size, filetype, thumb FROM thumbs WHERE hash = ?1",
            [hash],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .optional()?;

    if let Some((cached_mtime, size, filetype, thumb)) = row {
        if cached_mtime == mtime {
            return Ok(Some((thumb, Some(size), filetype)));
        }
    }
    Ok(None)
}

/// Inserts or updates a thumbnail and optional metadata in the cache.
pub fn set_thumb(
    conn: &Connection,
    hash: u64,
    mtime: i64,
    size: Option<i64>,
    filetype: Option<&str>,
    thumb: &[u8],
) -> Result<()> {
    conn.execute(
        "INSERT INTO thumbs (hash, mtime, size, filetype, thumb) 
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(hash) DO UPDATE SET 
             mtime = excluded.mtime,
             size = excluded.size,
             filetype = excluded.filetype,
             thumb = excluded.thumb;",
        params![hash, mtime, size, filetype, thumb],
    )?;
    Ok(())
}

/// Optional: remove thumbnails older than a certain mtime (cleanup).
pub fn prune_thumbs(conn: &Connection, min_mtime: i64) -> Result<()> {
    conn.execute("DELETE FROM thumbs WHERE mtime < ?1;", [min_mtime])?;
    Ok(())
}
