use std::{
    fs,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering}, Arc, Condvar, Mutex
    },
};

use jwalk::WalkDir;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

use crate::filesys::{
    os::windows::{get_system_clipboard, set_system_clipboard},
};

/// How to resolve a single conflict
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum DuplicateStrategy {
    Ignore,
    Replace,
    Index,
}

/// A request describing the conflict the UI must resolve.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConflictRequest {
    /// The request id from the calling paste operation
    pub request_id: u64,
    /// Source path being copied
    pub src: String,
    /// Intended destination path (where the conflict occurred)
    pub dest: String,
    /// Display-friendly filename (optional)
    pub name: String,
}

/// The user's response to the conflict (filled by resolve_copy_conflict)
#[derive(Clone, Debug)]
pub struct ConflictResponse {
    pub strategy: DuplicateStrategy,
    pub repeat_for_all: bool,
}

/// Shared state for the copy/paste stream.
/// - `conflict_lock` + `conflict_cv` are used to wait/notify for UI decisions.
pub struct CopyStreamState {
    pub current_id: AtomicU64,
    pub cancelled: AtomicBool,

    // conflict synchronization
    // If there's a pending request, conflict_request will be Some(request)
    // and conflict_response will be Some(response) once the user answered.
    conflict_lock: Mutex<ConflictSlot>,
    conflict_cv: Condvar,
}

struct ConflictSlot {
    request: Option<ConflictRequest>,
    response: Option<ConflictResponse>,
}

impl CopyStreamState {
    pub fn new() -> Self {
        Self {
            current_id: AtomicU64::new(0),
            cancelled: AtomicBool::new(false),
            conflict_lock: Mutex::new(ConflictSlot {
                request: None,
                response: None,
            }),
            conflict_cv: Condvar::new(),
        }
    }

    /// Called by the copy stream to post a conflict request and block until UI answers.
    /// Returns the chosen strategy and whether it should repeat for all remaining conflicts.
    /// The call will return early if the stream is cancelled or request_id doesn't match.
    pub fn request_conflict_decision(
        &self,
        request: ConflictRequest,
    ) -> Result<ConflictResponse, String> {
        // lock and set request; clear previous response
        let mut slot = self.conflict_lock.lock().unwrap();
        slot.request = Some(request);
        slot.response = None;

        // notify UI that a request is available (front-end should be listening for "clipboard-paste-conflict")
        // The emitter is on your copy loop side (you emit the event there). Here we only block.

        // wait for a response (blocking)
        loop {
            // Wait on condvar until response is set
            slot = self.conflict_cv.wait(slot).unwrap();

            // If response available, take it and return
            if let Some(resp) = slot.response.take() {
                // clear request after reading
                slot.request = None;
                return Ok(resp);
            }

            // Otherwise, spurious wake — continue loop
        }
    }

    /// Called by the UI command to submit decision and wake the blocked thread.
    pub fn submit_conflict_response(
        &self,
        request_id: u64,
        response: ConflictResponse,
    ) -> Result<(), String> {
        let mut slot = self.conflict_lock.lock().unwrap();

        // Optionally verify that the request_id matches the pending request
        if let Some(req) = &slot.request {
            if req.request_id != request_id {
                return Err("mismatched request id".into());
            }
        } else {
            // no pending request to answer
            return Err("no pending conflict request".into());
        }

        // set response and notify
        slot.response = Some(response);
        self.conflict_cv.notify_one();
        Ok(())
    }

    /// Helper for UI to peek current request (non-blocking). Useful if you want a route to fetch the current
    /// request details for rendering in the portal. Returns Some(ConflictRequest) if pending.
    pub fn take_pending_request(&self) -> Option<ConflictRequest> {
        let slot = self.conflict_lock.lock().unwrap();
        slot.request.clone()
    }
}

#[tauri::command]
pub fn copy_items_to_clipboard(paths: Vec<String>) -> Result<(), String> {
    set_system_clipboard(paths)
}

#[tauri::command]
pub async fn paste_items_from_clipboard(
    handle: AppHandle,
    state: State<'_, Arc<CopyStreamState>>,
    working_dir: String,
    request_id: u64,
) -> Result<(), String> {
    // mark this request id active and reset cancelled flag
    state.current_id.store(request_id, Ordering::Relaxed);
    state.cancelled.store(false, Ordering::Relaxed);

    // 1) Get clipboard paths (CF_HDROP etc.)
    let clipboard_paths = match get_system_clipboard() {
        Ok(v) => v,
        Err(e) => {
            return Err(format!("Failed to read clipboard: {}", e));
        }
    };

    if clipboard_paths.is_empty() {
        return Err("Clipboard does not contain file paths".into());
    }

    // Normalize working dir
    let dest_root = {
        let p = PathBuf::from(&working_dir);
        #[cfg(not(target_os = "windows"))]
        {
            p = PathBuf::from(p.to_string_lossy().to_string().replace("\\", "/"));
        }
        p
    };

    if !dest_root.is_dir() {
        return Err(format!(
            "Working dir is not a directory: {}",
            dest_root.display()
        ));
    }

    // Phase 1: scan -> build list of files to copy (src_path, rel_path, size)
    // We will preserve directory structure:
    // - if clipboard item is a file: rel_path = file_name
    // - if clipboard item is a directory: rel_path = path relative to that directory for its files
    let mut entries: Vec<(PathBuf, PathBuf, u64)> = Vec::new(); // (src, rel, size)
    let mut total_size: u64 = 0;

    for root in &clipboard_paths {
        // cancellation check
        if state.cancelled.load(Ordering::Relaxed)
            || state.current_id.load(Ordering::Relaxed) != request_id
        {
            let _ = handle.emit(
                "clipboard-paste-cancelled",
                serde_json::json!({ "request_id": request_id }),
            );
            return Ok(());
        }

        let root_path = root;
        if root_path.is_file() {
            let size = fs::metadata(root_path).map(|m| m.len()).unwrap_or(0);
            let rel = root_path
                .file_name()
                .map(|n| PathBuf::from(n))
                .unwrap_or_else(|| PathBuf::from("unknown"));
            entries.push((root_path.clone(), rel, size));
            total_size = total_size.saturating_add(size);
        } else if root_path.is_dir() {
            // Preserve the root folder itself by prefixing its name to relative entries
            let root_name = root_path
                .file_name()
                .map(|n| PathBuf::from(n))
                .unwrap_or_else(|| PathBuf::from("unknown"));

            let walker = WalkDir::new(root_path)
                .max_depth(std::usize::MAX)
                .follow_links(false);
            for entry in walker.into_iter().filter_map(|e| e.ok()) {
                if state.cancelled.load(Ordering::Relaxed)
                    || state.current_id.load(Ordering::Relaxed) != request_id
                {
                    let _ = handle.emit(
                        "clipboard-paste-cancelled",
                        serde_json::json!({ "request_id": request_id }),
                    );
                    return Ok(());
                }

                let path = entry.path();

                // skip the directory root itself (we create it implicitly)
                if &path == root_path {
                    continue;
                }

                if let Ok(md) = entry.metadata() {
                    if md.is_file() {
                        let size = md.len();

                        // relative path inside the root folder
                        let inner_rel = match path.strip_prefix(root_path) {
                            Ok(r) => r.to_path_buf(),
                            Err(_) => path
                                .file_name()
                                .map(|n| PathBuf::from(n))
                                .unwrap_or_default(),
                        };

                        // prepend the folder name so "src/main.rs" stays inside "src/"
                        let rel = root_name.join(inner_rel);

                        entries.push((path.to_path_buf(), rel, size));
                        total_size = total_size.saturating_add(size);
                    }
                }
            }
        } else {
            // If path doesn't exist, skip but log
            println!(
                "clipboard path not found or unsupported: {}",
                root_path.display()
            );
        }
    }

    // Emit scan result to frontend
    let _ = handle.emit(
        "clipboard-paste-scan",
        serde_json::json!({
            "request_id": request_id,
            "total_size": total_size,
            "file_count": entries.len(),
        }),
    );

    // Phase 2: perform copying
    // The frontend will sum sizes from file events to produce a progress bar
    for (src, rel, _size) in entries.iter() {
        // cancellation check
        if state.cancelled.load(Ordering::Relaxed)
            || state.current_id.load(Ordering::Relaxed) != request_id
        {
            let _ = handle.emit(
                "clipboard-paste-cancelled",
                serde_json::json!({ "request_id": request_id }),
            );
            return Ok(());
        }

        // compute destination path
        let dest_path = {
            let mut d = dest_root.join(&rel);
            // ensure parent dir exists
            if let Some(parent) = d.parent() {
                if let Err(err) = fs::create_dir_all(parent) {
                    eprintln!("Failed to create dirs {}: {}", parent.display(), err);
                    // attempt to continue
                }
            }

            // If file exists, choose a unique name (append " (copy)", " (copy 2)", etc.)
            if d.exists() {
                println!("Conflict encountered in stream!");
                // emit conflict to frontend so it can show modal
                let _ = handle.emit(
                    "clipboard-paste-conflict",
                    serde_json::json!({
                        "request_id": request_id,
                        "src": src.display().to_string(),
                        "dest": d.display().to_string(),
                        "name": d.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string(),
                    }),
                );

                // build a ConflictRequest
                let conflict_req = ConflictRequest {
                    request_id,
                    src: src.display().to_string(),
                    dest: d.display().to_string(),
                    name: d
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_string(),
                };

                // ask the state to block until user responds (or optionally timeout)
                match state.request_conflict_decision(conflict_req) {
                    Ok(response) => {
                        match response.strategy {
                            DuplicateStrategy::Ignore => {
                                // skip this file; continue outer loop
                                continue;
                            }
                            DuplicateStrategy::Replace => {
                                // remove existing file and proceed to copy
                                let _ = std::fs::remove_file(&d);
                                // fallthrough and copy src -> d
                            }
                            DuplicateStrategy::Index => {
                                // compute unique index name as you did before
                                let base = d
                                    .file_stem()
                                    .and_then(|s| s.to_str())
                                    .unwrap_or("file")
                                    .to_string();
                                let ext = d
                                    .extension()
                                    .and_then(|s| s.to_str())
                                    .map(|s| format!(".{}", s))
                                    .unwrap_or_default();
                                let mut idx = 1;
                                loop {
                                    let name = if idx == 1 {
                                        format!("{} (copy){}", base, ext)
                                    } else {
                                        format!("{} (copy {}){}", base, idx, ext)
                                    };
                                    let try_path = d.with_file_name(name);
                                    if !try_path.exists() {
                                        d = try_path;
                                        break;
                                    }
                                    idx += 1;
                                }
                            }
                        }

                        // If response.repeat_for_all is true, you may want to set an ephemeral variable so subsequent conflicts
                        // skip prompting and use the same strategy; implement as you see fit (store in local variable).
                    }
                    Err(e) => {
                        // state was cancelled or mismatched; handle gracefully
                        eprintln!("conflict decision failed: {}", e);
                        // you might choose to skip or abort whole operation
                        continue;
                    }
                }
            }

            d
        };

        // Do the copy. For simplicity, we do a single fs::copy call (not chunked).
        // For very large files you could copy in chunks and emit incremental bytes; implement later if needed.
        match fs::copy(&src, &dest_path) {
            Ok(bytes_copied) => {
                // emit event for copied file (src, dest, size)
                let _ = handle.emit(
                    "clipboard-paste-file",
                    serde_json::json!({
                        "request_id": request_id,
                        "src": src.display().to_string(),
                        "dest": dest_path.display().to_string(),
                        "size": bytes_copied,
                    }),
                );
            }
            Err(err) => {
                // emit a failure event but continue with other files
                let _ = handle.emit(
                    "clipboard-paste-file-error",
                    serde_json::json!({
                        "request_id": request_id,
                        "src": src.display().to_string(),
                        "dest": dest_path.display().to_string(),
                        "error": err.to_string(),
                    }),
                );
            }
        }
    }

    println!("Copy stream completed!");

    // Phase 3: done
    let _ = handle.emit(
        "clipboard-paste-complete",
        serde_json::json!({
            "request_id": request_id,
            "total_size": total_size,
            "files_copied": entries.len(),
        }),
    );

    Ok(())
}