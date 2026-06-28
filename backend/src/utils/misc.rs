pub fn format_file_size(bytes: u64, unit: Option<&str>) -> String {
    let units = ["B", "KB", "MB", "GB", "TB"];

    if let Some(u) = unit {
        let requested = u.to_uppercase();
        if let Some(idx) = units.iter().position(|&x| x == requested) {
            let size = bytes as f64 / 1024_f64.powi(idx as i32);
            return format!("{:.2}{}", size, requested);
        }
    }

    let mut size = bytes as f64;
    let mut unit_idx = 0;
    while size >= 1024.0 && unit_idx < units.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }
    format!("{:.2}{}", size, units[unit_idx])
}

pub fn is_valid_batch_id(batch_id: &str) -> bool {
    let parts: Vec<&str> = batch_id.split('-').collect();
    if parts.len() != 2 {
        return false;
    }
    if !parts[0].chars().all(|c| c.is_ascii_digit()) {
        return false;
    }
    let second = parts[1];
    if second.len() < 8 || second.len() > 9 {
        return false;
    }
    second
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
}
