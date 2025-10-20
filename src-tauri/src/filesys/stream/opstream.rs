use std::{
    fs,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Condvar, Mutex,
    },
    thread,
    time::Duration,
};

use jwalk::WalkDir;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

use crate::filesys::os::windows::{get_system_clipboard, set_system_clipboard, ClipboardOp};

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
    set_system_clipboard(paths, ClipboardOp::Copy)
}

#[tauri::command]
pub fn cut_items_to_clipboard(paths: Vec<String>) -> Result<(), String> {
    set_system_clipboard(paths, ClipboardOp::Move)
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

    // 1) Get clipboard paths and operation
    let (clipboard_paths, clipboard_op) = match get_system_clipboard() {
        Ok(v) => v,
        Err(e) => return Err(format!("Failed to read clipboard: {}", e)),
    };

    if clipboard_paths.is_empty() {
        return Err("Clipboard does not contain file paths".into());
    }

    // Normalize working dir
    let dest_root = PathBuf::from(&working_dir);
    if !dest_root.is_dir() {
        return Err(format!(
            "Working dir is not a directory: {}",
            dest_root.display()
        ));
    }

    // Phase 1: scan -> build list of files to copy/move
    let mut entries: Vec<(PathBuf, PathBuf, u64)> = Vec::new(); // (src, rel, size)
    let mut total_size: u64 = 0;

    for root_path in &clipboard_paths {
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

        if root_path.is_file() {
            let size = fs::metadata(root_path).map(|m| m.len()).unwrap_or(0);
            let rel = root_path
                .file_name()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("unknown"));
            entries.push((root_path.clone(), rel, size));
            total_size = total_size.saturating_add(size);
        } else if root_path.is_dir() {
            // include root folder name
            let root_name = root_path
                .file_name()
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("unknown"));

            let walker = WalkDir::new(root_path).follow_links(false);
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
                if &path == root_path {
                    continue;
                }

                if let Ok(md) = entry.metadata() {
                    if md.is_file() {
                        let size = md.len();
                        let inner_rel = path
                            .strip_prefix(root_path)
                            .map(|r| r.to_path_buf())
                            .unwrap_or_else(|_| PathBuf::from("unknown"));
                        let rel = root_name.join(inner_rel);

                        entries.push((path.to_path_buf(), rel, size));
                        total_size = total_size.saturating_add(size);
                    }
                }
            }
        }
    }

    // Emit scan result
    let _ = handle.emit(
        "clipboard-paste-scan",
        serde_json::json!({
            "request_id": request_id,
            "total_size": total_size,
            "file_count": entries.len(),
            "operation": format!("{:?}", clipboard_op),
        }),
    );

    // Phase 2: perform copying or moving
    let mut repeat_strategy: Option<DuplicateStrategy> = None;
    let mut repeat_for_all = false;

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

        let mut dest_path = dest_root.join(&rel);
        if let Some(parent) = dest_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        // conflict handling
        if dest_path.exists() {
            let chosen_strategy = if repeat_for_all {
                repeat_strategy.unwrap_or(DuplicateStrategy::Index)
            } else {
                thread::sleep(Duration::from_millis(50));
                let _ = handle.emit(
                    "clipboard-paste-conflict",
                    serde_json::json!({
                        "request_id": request_id,
                        "src": src.display().to_string(),
                        "dest": dest_path.display().to_string(),
                        "name": dest_path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string(),
                    }),
                );

                let conflict_req = ConflictRequest {
                    request_id,
                    src: src.display().to_string(),
                    dest: dest_path.display().to_string(),
                    name: dest_path
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_string(),
                };

                match state.request_conflict_decision(conflict_req) {
                    Ok(resp) => {
                        if resp.repeat_for_all {
                            repeat_for_all = true;
                            repeat_strategy = Some(resp.strategy);
                        }
                        resp.strategy
                    }
                    Err(_) => continue,
                }
            };

            match chosen_strategy {
                DuplicateStrategy::Ignore => continue,
                DuplicateStrategy::Replace => {
                    let _ = fs::remove_file(&dest_path);
                }
                DuplicateStrategy::Index => {
                    let file_name = dest_path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("file");
                    let ext = dest_path
                        .extension()
                        .and_then(|s| s.to_str())
                        .map(|s| format!(".{}", s))
                        .unwrap_or_default();

                    let mut i = 1;
                    loop {
                        let candidate = format!("{} ({}){}", file_name, i, ext);
                        let try_path = dest_path.with_file_name(candidate);
                        if !try_path.exists() {
                            dest_path = try_path;
                            break;
                        }
                        i += 1;
                    }
                }
            }
        }

        // perform file operation (copy or move)
        let result = match clipboard_op {
            ClipboardOp::Copy | ClipboardOp::Link => fs::copy(src, &dest_path)
                .map(|bytes| (bytes, false)), // false = not removed
            ClipboardOp::Move => {
                // try rename first (fast path)
                match fs::rename(src, &dest_path) {
                    Ok(_) => Ok((0, true)), // true = source removed
                    Err(_) => {
                        // fallback: cross-device move (copy + remove)
                        let copy_result = fs::copy(src, &dest_path);
                        if copy_result.is_ok() {
                            let _ = fs::remove_file(src);
                        }
                        copy_result.map(|bytes| (bytes, true))
                    }
                }
            },
            // handle any future/unexpected variants gracefully
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Unsupported clipboard operation: {:?}", clipboard_op),
            )),
        };

        match result {
            Ok((bytes, _removed)) => {
                let _ = handle.emit(
                    "clipboard-paste-file",
                    serde_json::json!({
                        "request_id": request_id,
                        "src": src.display().to_string(),
                        "dest": dest_path.display().to_string(),
                        "size": bytes,
                        "operation": format!("{:?}", clipboard_op),
                    }),
                );
            }
            Err(err) => {
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

    // Done
    let _ = handle.emit(
        "clipboard-paste-complete",
        serde_json::json!({
            "request_id": request_id,
            "total_size": total_size,
            "files_processed": entries.len(),
            "operation": format!("{:?}", clipboard_op),
        }),
    );

    Ok(())
}
