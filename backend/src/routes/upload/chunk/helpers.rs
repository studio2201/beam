//! Helpers for the chunk-upload route, factored out of `mod.rs`.
//!
//! Each function corresponds to one phase of the upload pipeline:
//!
//! - [`resolve_metadata`] — read the in-progress upload metadata from
//!   the cache or from disk.
//! - [`already_complete`] — early-return response when the upload has
//!   already hit its target file size.
//! - [`truncate_chunk_if_oversized`] — clamp the incoming chunk to
//!   the remaining byte budget.
//! - [`open_or_get_file_handle`] — lazily open the partial file and
//!   cache the handle.
//! - [`finalize_if_complete`] — atomic rename + state cleanup when
//!   the last chunk just landed.

use std::path::Path as StdPath;
use std::sync::Arc;

use serde_json::json;
use tokio::io::AsyncWriteExt;

use crate::config::AppConfig;
use crate::routes::upload::UploadState;
use crate::routes::upload::metadata::{delete_upload_metadata, read_upload_metadata};

/// Resolve upload metadata from the in-memory cache or from disk.
///
/// Returns `Some(metadata)` on a hit, or `None` if no session exists for
/// the given `upload_id`. Caches the disk-loaded value into the active
/// map before returning.
pub async fn resolve_metadata(
    state: &UploadState,
    config: &AppConfig,
    upload_id: &str,
) -> Option<crate::routes::upload::metadata::UploadMetadata> {
    {
        let active = state.active_uploads.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(m) = active.get(upload_id).cloned() {
            return Some(m);
        }
    }
    let on_disk = read_upload_metadata(&config.upload_dir, upload_id).await?;
    let mut active = state.active_uploads.lock().unwrap_or_else(|e| e.into_inner());
    active.insert(upload_id.to_string(), on_disk.clone());
    Some(on_disk)
}

/// If the upload has already received its full file, return the response
/// the handler should emit. Otherwise return `None`.
pub fn already_complete(
    metadata: &crate::routes::upload::metadata::UploadMetadata,
    state: &UploadState,
    config: &AppConfig,
    upload_id: &str,
) -> Option<serde_json::Value> {
    if metadata.bytes_received < metadata.file_size {
        return None;
    }
    let partial = StdPath::new(&metadata.partial_file_path);
    let final_path = StdPath::new(&metadata.file_path);
    if !final_path.exists() && partial.exists() {
        let _ = std::fs::rename(partial, final_path);
    }
    let _ = delete_upload_metadata(&config.upload_dir, upload_id);
    state
        .active_uploads
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .remove(upload_id);
    state
        .file_handles
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .remove(upload_id);
    Some(json!({ "bytesReceived": metadata.file_size, "progress": 100 }))
}

/// Clamp an incoming chunk to the remaining byte budget. Returns the
/// truncated body plus the (possibly updated) `bytes_received` value.
pub fn truncate_chunk_if_oversized(
    chunk: axum::body::Bytes,
    bytes_received: u64,
    file_size: u64,
) -> (axum::body::Bytes, u64, u64) {
    let chunk_size = chunk.len() as u64;
    if bytes_received + chunk_size <= file_size {
        return (chunk, chunk_size, bytes_received + chunk_size);
    }
    let write_size = file_size.saturating_sub(bytes_received);
    let truncated = if write_size > 0 {
        chunk.slice(0..(write_size as usize))
    } else {
        chunk.slice(0..0)
    };
    let new_received = if write_size == 0 {
        file_size
    } else {
        bytes_received + write_size
    };
    (truncated, write_size, new_received)
}

/// Return the cached file handle for `upload_id` if present, otherwise
/// open a new one and cache it. Returns an error string on I/O failure.
pub async fn open_or_get_file_handle(
    state: &UploadState,
    upload_id: &str,
    partial_path: &StdPath,
) -> Result<Arc<tokio::sync::Mutex<tokio::fs::File>>, String> {
    {
        let handles = state.file_handles.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(h) = handles.get(upload_id).cloned() {
            return Ok(h);
        }
    }
    let file = match tokio::fs::OpenOptions::new()
        .write(true)
        .append(true)
        .create(true)
        .open(partial_path)
        .await
    {
        Ok(f) => f,
        Err(e) => {
            tracing::error!("Failed to open partial file {:?}: {}", partial_path, e);
            return Err(format!("Failed to open partial file: {e}"));
        }
    };
    let arc = Arc::new(tokio::sync::Mutex::new(file));
    let mut handles = state.file_handles.lock().unwrap_or_else(|e| e.into_inner());
    Ok(handles
        .entry(upload_id.to_string())
        .or_insert(arc)
        .clone())
}

/// Write `bytes` to the file handle, flushing after.
pub async fn write_chunk(
    file: Arc<tokio::sync::Mutex<tokio::fs::File>>,
    bytes: &axum::body::Bytes,
) -> Result<(), String> {
    let mut guard = file.lock().await;
    if let Err(e) = guard.write_all(bytes).await {
        return Err(format!("Failed to write chunk: {e}"));
    }
    let _ = guard.flush().await;
    Ok(())
}

/// If `bytes_received` has reached `file_size`, atomically rename the
/// partial file to its final name and clean up state. Returns `true` if
/// finalization ran.
pub async fn finalize_if_complete(
    state: &UploadState,
    config: &AppConfig,
    upload_id: &str,
    metadata: &crate::routes::upload::metadata::UploadMetadata,
) -> bool {
    if metadata.bytes_received < metadata.file_size {
        return false;
    }
    {
        let mut handles = state.file_handles.lock().unwrap_or_else(|e| e.into_inner());
        handles.remove(upload_id);
    }
    let partial = StdPath::new(&metadata.partial_file_path);
    let final_path = StdPath::new(&metadata.file_path);
    match tokio::fs::rename(partial, final_path).await {
        Ok(_) => {
            tracing::info!(
                "Upload completed and finalized: {} as {:?}",
                metadata.original_filename,
                final_path
            );
            delete_upload_metadata(&config.upload_dir, upload_id).await;
            state
                .active_uploads
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .remove(upload_id);
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            tracing::warn!(
                "Partial file {:?} missing during finalization, assuming completed elsewhere.",
                partial
            );
            delete_upload_metadata(&config.upload_dir, upload_id).await;
            state
                .active_uploads
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .remove(upload_id);
        }
        Err(e) => {
            tracing::error!(
                "CRITICAL: Failed to rename partial file {:?} to {:?}: {}",
                partial,
                final_path,
                e
            );
        }
    }
    true
}

/// Compute the progress percentage (0-100) from received vs total bytes.
#[must_use]
pub fn progress_percent(bytes_received: u64, file_size: u64) -> u64 {
    if file_size == 0 {
        return 100;
    }
    let pct = (bytes_received as f64 / file_size as f64 * 100.0).round() as u64;
    pct.min(100)
}
