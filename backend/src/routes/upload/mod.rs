mod metadata;
mod init;
mod chunk;

use axum::{
    routing::post,
    Router,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use self::init::init_upload;
use self::chunk::{upload_chunk, cancel_upload};

pub fn router() -> Router<crate::AppState> {
    Router::new()
        .route("/init", post(init_upload))
        .route("/chunk/:uploadId", post(upload_chunk))
        .route("/cancel/:uploadId", post(cancel_upload))
}

pub struct UploadState {
    pub folder_mappings: Mutex<HashMap<String, String>>,
    pub batch_activity: Mutex<HashMap<String, std::time::Instant>>,
}

impl UploadState {
    pub fn new() -> Self {
        Self {
            folder_mappings: Mutex::new(HashMap::new()),
            batch_activity: Mutex::new(HashMap::new()),
        }
    }
}

pub fn start_batch_cleanup(state: Arc<UploadState>) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
        loop {
            interval.tick().await;
            let now = std::time::Instant::now();
            let timeout = std::time::Duration::from_secs(30 * 60);
            
            let mut expired_batches = Vec::new();
            {
                let activity = state.batch_activity.lock().unwrap();
                for (batch_id, last_activity) in activity.iter() {
                    if now.duration_since(*last_activity) >= timeout {
                        expired_batches.push(batch_id.clone());
                    }
                }
            }
            
            if !expired_batches.is_empty() {
                tracing::info!("Cleaning up {} inactive batch sessions", expired_batches.len());
                let mut activity = state.batch_activity.lock().unwrap();
                let mut mappings = state.folder_mappings.lock().unwrap();
                
                for batch_id in expired_batches {
                    activity.remove(&batch_id);
                    mappings.retain(|key, _| !key.ends_with(&format!("-{}", batch_id)));
                }
            }
        }
    });
}
