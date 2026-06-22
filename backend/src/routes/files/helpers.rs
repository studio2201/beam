use chrono::{DateTime, Utc};
use serde::Serialize;
use std::fs;
use std::path::Path as StdPath;

#[derive(Serialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum FileItem {
    File {
        name: String,
        path: String,
        size: u64,
        #[serde(rename = "formattedSize")]
        formatted_size: String,
        #[serde(rename = "uploadDate")]
        upload_date: DateTime<Utc>,
        extension: String,
    },
    Directory {
        name: String,
        path: String,
        size: u64,
        #[serde(rename = "formattedSize")]
        formatted_size: String,
        #[serde(rename = "uploadDate")]
        upload_date: DateTime<Utc>,
        children: Vec<FileItem>,
    },
}

pub fn get_directory_contents(dir_path: &StdPath, relative_path: &str) -> std::io::Result<Vec<FileItem>> {
    let mut items = Vec::new();
    
    if !dir_path.exists() {
        return Ok(items);
    }
    
    let entries = fs::read_dir(dir_path)?;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        
        if name == ".metadata" || name.starts_with('.') {
            continue;
        }
        
        let full_path = entry.path();
        let item_relative_path = if relative_path.is_empty() {
            name.clone()
        } else {
            format!("{}/{}", relative_path, name)
        };
        
        let metadata = entry.metadata()?;
        let upload_date: DateTime<Utc> = metadata.modified()?.into();
        
        if metadata.is_dir() {
            let children = get_directory_contents(&full_path, &item_relative_path)?;
            let size = calculate_total_size(&children);
            items.push(FileItem::Directory {
                name,
                path: item_relative_path,
                size,
                formatted_size: crate::utils::format_file_size(size, None),
                upload_date,
                children,
            });
        } else {
            let size = metadata.len();
            let extension = StdPath::new(&name)
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| format!(".{}", e.to_lowercase()))
                .unwrap_or_default();
                
            items.push(FileItem::File {
                name,
                path: item_relative_path,
                size,
                formatted_size: crate::utils::format_file_size(size, None),
                upload_date,
                extension,
            });
        }
    }
    
    items.sort_by(|a, b| {
        let a_type = match a {
            FileItem::Directory { .. } => 0,
            FileItem::File { .. } => 1,
        };
        let b_type = match b {
            FileItem::Directory { .. } => 0,
            FileItem::File { .. } => 1,
        };
        
        if a_type != b_type {
            a_type.cmp(&b_type)
        } else {
            let a_name = match a {
                FileItem::Directory { name, .. } | FileItem::File { name, .. } => name,
            };
            let b_name = match b {
                FileItem::Directory { name, .. } | FileItem::File { name, .. } => name,
            };
            a_name.cmp(b_name)
        }
    });
    
    Ok(items)
}

pub fn calculate_total_size(items: &[FileItem]) -> u64 {
    items.iter().map(|item| match item {
        FileItem::File { size, .. } => *size,
        FileItem::Directory { size, .. } => *size,
    }).sum()
}

pub fn count_files(items: &[FileItem]) -> u64 {
    items.iter().map(|item| match item {
        FileItem::File { .. } => 1,
        FileItem::Directory { children, .. } => count_files(children),
    }).sum()
}

pub fn create_safe_content_disposition(filename: &str) -> String {
    let basename = StdPath::new(filename)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(filename);

    let sanitized: String = basename.chars()
        .map(|c| if c.is_ascii_control() || c == '"' || c == '\\' { '_' } else { c })
        .collect();

    let is_ascii_printable = sanitized.chars().all(|c| c >= ' ' && c <= '~');

    if is_ascii_printable {
        let escaped = sanitized.replace('\\', "\\\\").replace('"', "\\\"");
        format!("attachment; filename=\"{}\"", escaped)
    } else {
        let encoded = percent_encoding::utf8_percent_encode(&sanitized, percent_encoding::NON_ALPHANUMERIC).to_string();
        let ascii_safe: String = sanitized.chars()
            .map(|c| if c >= ' ' && c <= '~' { c } else { '_' })
            .collect();
        format!("attachment; filename=\"{}\"; filename*=UTF-8''{}", ascii_safe, encoded)
    }
}
