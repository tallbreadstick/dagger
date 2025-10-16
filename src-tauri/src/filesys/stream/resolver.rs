use tauri::State;
use std::sync::Arc;
use serde::Deserialize;

use crate::filesys::stream::opstream::{ConflictResponse, CopyStreamState, DuplicateStrategy};

#[derive(Deserialize)]
pub struct ResolveCopyPayload {
    pub request_id: u64,
    pub strategy: String, // "Ignore" | "Replace" | "Index"
    pub repeat_for_all: bool,
}

#[tauri::command]
pub async fn resolve_copy_conflict(
    payload: ResolveCopyPayload,
    state: State<'_, Arc<CopyStreamState>>
) -> Result<(), String> {
    // parse strategy
    let strat = match payload.strategy.as_str() {
        "Ignore" => DuplicateStrategy::Ignore,
        "Replace" => DuplicateStrategy::Replace,
        "Index" => DuplicateStrategy::Index,
        other => return Err(format!("unknown strategy: {}", other)),
    };

    let resp = ConflictResponse {
        strategy: strat,
        repeat_for_all: payload.repeat_for_all,
    };

    // submit response (this will notify the blocked copy thread)
    state.submit_conflict_response(payload.request_id, resp)
        .map_err(|e| format!("failed to submit response: {}", e))
}
