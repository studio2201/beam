//! `POST /api/upload/:upload_id/chunk` — receive one chunk of an
//! in-progress upload and append it to the partial file.
//!
//! The handler is intentionally a thin orchestrator over the helpers in
//! [`super::helpers`]; each phase (resolve metadata, finalize if
//! already complete, truncate oversize chunks, open the file handle,
//! write, finalize on completion) is a single function call.

pub mod helpers;

use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde_json::json;
use std::path::Path as StdPath;
use std::sync::Arc;

use crate::config::AppConfig;
use crate::routes::auth::RequirePin;
use crate::routes::upload::UploadState;

#[tracing::instrument(
    skip(config, state, _auth, headers, body),
    fields(upload_id = %upload_id, chunk_size = body.len())
)]
pub async fn upload_chunk(
    State(config): State<Arc<AppConfig>>,
    State(state): State<Arc<UploadState>>,
    _auth: RequirePin,
    Path(upload_id): Path<String>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Response {
    let chunk_size = body.len() as u64;
    if chunk_size == 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Empty chunk received" })),
        )
            .into_response();
    }

    // 1. Resolve metadata from cache or disk.
    let Some(mut metadata) =
        helpers::resolve_metadata(&state, &config, &upload_id).await
    else {
        let client_batch_id = headers
            .get("x-batch-id")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("none");
        tracing::warn!(
            "Upload metadata not found for chunk request: {}. Client Batch ID: {}.",
            upload_id,
            client_batch_id
        );
        return (
            StatusCode::NOT_FOUND,
            Json(json!({ "error": "Upload session not found or already completed" })),
        )
            .into_response();
    };

    // 2. Bump batch activity.
    if !metadata.batch_id.is_empty() && crate::utils::is_valid_batch_id(&metadata.batch_id) {
        state
            .batch_activity
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(metadata.batch_id.clone(), std::time::Instant::now());
    }

    // 3. Already-complete early return.
    if let Some(resp) = helpers::already_complete(&metadata, &state, &config, &upload_id) {
        return Json(resp).into_response();
    }

    // 4. Truncate the chunk if it overshoots the file size.
    let (chunk_bytes, write_size, new_received) = helpers::truncate_chunk_if_oversized(
        body,
        metadata.bytes_received,
        metadata.file_size,
    );
    if write_size > 0 {
        metadata.bytes_received = new_received;
    } else {
        // No bytes to write (we're already at file_size); finalize and exit.
        let _ = helpers::finalize_if_complete(&state, &config, &upload_id, &metadata).await;
        return Json(json!({
            "bytesReceived": metadata.file_size,
            "progress": 100
        }))
        .into_response();
    }

    // 5. Open (or reuse) the file handle, write the chunk.
    let partial_path = StdPath::new(&metadata.partial_file_path);
    let file = match helpers::open_or_get_file_handle(&state, &upload_id, partial_path).await {
        Ok(f) => f,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e })),
            )
                .into_response();
        }
    };
    if let Err(e) = helpers::write_chunk(file, &chunk_bytes).await {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": e })),
        )
            .into_response();
    }

    // 6. Update progress, persist metadata, and finalize if this was
    //    the last chunk.
    let progress = helpers::progress_percent(metadata.bytes_received, metadata.file_size);
    metadata.last_activity = chrono::Utc::now().timestamp_millis() as u64;
    {
        let mut active = state
            .active_uploads
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        active.insert(upload_id.clone(), metadata.clone());
    }
    tracing::debug!(
        "Chunk written for {}: {}/{} ({}%)",
        upload_id,
        metadata.bytes_received,
        metadata.file_size,
        progress
    );

    if metadata.bytes_received >= metadata.file_size {
        tracing::info!(
            "Upload {} ({}) completed {} bytes.",
            upload_id,
            metadata.original_filename,
            metadata.bytes_received
        );
        let _ = helpers::finalize_if_complete(&state, &config, &upload_id, &metadata).await;
    }

    Json(json!({
        "bytesReceived": metadata.bytes_received,
        "progress": progress
    }))
    .into_response()
}
