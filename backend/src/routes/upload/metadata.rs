use serde::{Deserialize, Serialize};
use std::path::{Path as StdPath, PathBuf};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UploadMetadata {
    #[serde(rename = "uploadId")]
    pub upload_id: String,
    #[serde(rename = "originalFilename")]
    pub original_filename: String,
    #[serde(rename = "filePath")]
    pub file_path: String,
    #[serde(rename = "partialFilePath")]
    pub partial_file_path: String,
    #[serde(rename = "fileSize")]
    pub file_size: u64,
    #[serde(rename = "bytesReceived")]
    pub bytes_received: u64,
    #[serde(rename = "batchId")]
    pub batch_id: String,
    #[serde(rename = "createdAt")]
    pub created_at: u64,
    #[serde(rename = "lastActivity")]
    pub last_activity: u64,
}

pub fn get_metadata_path(upload_dir: &StdPath, upload_id: &str) -> PathBuf {
    upload_dir.join(".metadata").join(format!("{}.meta", upload_id))
}

pub async fn read_upload_metadata(upload_dir: &StdPath, upload_id: &str) -> Option<UploadMetadata> {
    if upload_id.contains("..") {
        return None;
    }
    let path = get_metadata_path(upload_dir, upload_id);
    let content = tokio::fs::read_to_string(&path).await.ok()?;
    serde_json::from_str(&content).ok()
}

pub async fn write_upload_metadata(upload_dir: &StdPath, upload_id: &str, mut metadata: UploadMetadata) -> std::io::Result<()> {
    if upload_id.contains("..") {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid upload ID"));
    }
    let metadata_dir = upload_dir.join(".metadata");
    tokio::fs::create_dir_all(&metadata_dir).await?;
    
    let path = get_metadata_path(upload_dir, upload_id);
    metadata.last_activity = chrono::Utc::now().timestamp_millis() as u64;
    
    let content = serde_json::to_string_pretty(&metadata)?;
    
    let temp_name = format!("{}.{}.tmp", upload_id, rand::random::<u32>());
    let temp_path = metadata_dir.join(&temp_name);
    
    tokio::fs::write(&temp_path, content).await?;
    if let Err(e) = tokio::fs::rename(&temp_path, &path).await {
        let _ = tokio::fs::remove_file(&temp_path).await;
        return Err(e);
    }
    Ok(())
}

pub async fn delete_upload_metadata(upload_dir: &StdPath, upload_id: &str) {
    if upload_id.contains("..") {
        return;
    }
    let path = get_metadata_path(upload_dir, upload_id);
    let _ = tokio::fs::remove_file(path).await;
}

pub fn get_unique_folder_path(folder_path: &StdPath) -> PathBuf {
    let mut counter = 1;
    let mut final_path = folder_path.to_path_buf();
    
    while final_path.exists() {
        let parent = folder_path.parent().unwrap_or_else(|| StdPath::new(""));
        let folder_name = folder_path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        let new_name = format!("{} ({})", folder_name, counter);
        final_path = parent.join(new_name);
        counter += 1;
    }
    
    final_path
}
