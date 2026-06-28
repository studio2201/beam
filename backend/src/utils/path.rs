use std::fs;
use std::path::{Component, Path, PathBuf};

pub fn normalize_path(path: &Path) -> PathBuf {
    let mut components = path.components().peekable();
    let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek() {
        let buf = PathBuf::from(c.as_os_str());
        components.next();
        buf
    } else {
        PathBuf::new()
    };

    let mut normalized = Vec::new();
    for component in components {
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                ret.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(c) => {
                normalized.push(c);
            }
        }
    }
    for component in normalized {
        ret.push(component);
    }
    ret
}

#[must_use]
pub fn is_path_within_upload_dir(
    file_path: &Path,
    upload_dir: &Path,
    require_exists: bool,
) -> bool {
    let real_upload_dir = match fs::canonicalize(upload_dir) {
        Ok(p) => p,
        Err(_) => return false,
    };

    if require_exists {
        if !file_path.exists() {
            return false;
        }
        return match fs::canonicalize(file_path) {
            Ok(p) => p.starts_with(&real_upload_dir),
            Err(_) => false,
        };
    }

    if file_path
        .components()
        .any(|c| matches!(c, Component::ParentDir))
    {
        return false;
    }

    let absolute_path = if file_path.is_absolute() {
        file_path.to_path_buf()
    } else {
        std::env::current_dir().unwrap_or_default().join(file_path)
    };

    let mut existing = PathBuf::new();
    let mut suffix = PathBuf::new();
    for component in absolute_path.components() {
        match component {
            Component::Prefix(p) => {
                existing = PathBuf::from(p.as_os_str());
            }
            Component::RootDir => {
                existing = PathBuf::from(std::path::MAIN_SEPARATOR.to_string());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                unreachable!("filtered above");
            }
            Component::Normal(c) => {
                let candidate = existing.join(c);
                if candidate.exists() {
                    existing = candidate;
                } else {
                    suffix.push(c);
                }
            }
        }
    }

    if suffix == existing || existing.as_os_str().is_empty() {
        return normalize_path(&absolute_path).starts_with(&real_upload_dir);
    }

    let canonical_existing = match fs::canonicalize(&existing) {
        Ok(p) => p,
        Err(_) => return false,
    };

    if !canonical_existing.starts_with(&real_upload_dir) {
        return false;
    }

    let candidate = canonical_existing.join(&suffix);
    candidate.starts_with(&real_upload_dir)
}

pub fn sanitize_filename_safe(filename: &str) -> String {
    if filename.is_empty() {
        return "unnamed_file.txt".to_string();
    }

    let path = Path::new(filename);
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_default();

    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unnamed_file");

    let mut base_name = stem.replace(|c: char| c.is_whitespace() || c == '+', "_");

    base_name = base_name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == '.')
        .collect();

    while base_name.contains("__") {
        base_name = base_name.replace("__", "_");
    }

    let trimmed = base_name
        .trim_matches(|c| c == '.' || c == '_' || c == '-')
        .to_string();
    let mut final_base = if trimmed.is_empty() {
        "file".to_string()
    } else {
        trimmed
    };

    let reserved_names = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    if reserved_names.contains(&final_base.to_uppercase().as_str()) {
        final_base.push_str("_file");
    }

    if final_base.len() > 200 {
        final_base.truncate(200);
    }

    let clean_ext: String = ext
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '.')
        .collect();

    if clean_ext.is_empty() {
        final_base
    } else if clean_ext.starts_with('.') {
        format!("{}{}", final_base, clean_ext)
    } else {
        format!("{}.{}", final_base, clean_ext)
    }
}

pub fn sanitize_path_preserve_dirs_safe(file_path: &str) -> String {
    if file_path.is_empty() {
        return "unnamed_file.txt".to_string();
    }

    if file_path.split(['/', '\\']).any(|p| p == "..") {
        tracing::warn!(
            "sanitize_path_preserve_dirs_safe: rejected path containing '..': {file_path}"
        );
        return "unnamed_file.txt".to_string();
    }

    let parts: Vec<String> = file_path
        .split('/')
        .map(|part| part.replace('\\', "/"))
        .flat_map(|part| {
            part.split('/')
                .map(|p| p.to_string())
                .collect::<Vec<String>>()
        })
        .filter(|part| !part.is_empty() && part != "." && part != "..")
        .map(|part| sanitize_filename_safe(&part))
        .collect();

    if parts.is_empty() {
        "unnamed_file.txt".to_string()
    } else {
        parts.join("/")
    }
}
