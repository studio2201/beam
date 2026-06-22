use crate::api::{cancel_upload, init_upload, upload_chunk};
use crate::types::Msg;
use crate::utils::get_file_path;

pub async fn perform_file_upload(
    file: web_sys::File,
    batch_id: String,
    max_retries: usize,
    link: yew::html::Scope<crate::app::App>,
) {
    let path = get_file_path(&file);
    let size = file.size() as u64;

    // 1. Initialize file upload
    link.send_message(Msg::UploadProgressUpdate(
        path.clone(),
        0,
        0.0,
        "initializing...".to_string(),
        None,
    ));

    let upload_id = match init_upload(&path, size, &batch_id).await {
        Ok(uid) => uid,
        Err(e) => {
            link.send_message(Msg::UploadFailed(path.clone(), e));
            return;
        }
    };

    link.send_message(Msg::UploadInit(path.clone(), upload_id.clone()));

    // 2. Perform chunked uploads
    let chunk_size = 1024 * 1024; // 1MB chunks
    let mut position = 0u64;
    let mut failed = false;

    let mut last_uploaded_bytes = 0u64;
    let window_obj = web_sys::window().unwrap();
    let perf = window_obj.performance().unwrap();
    let mut last_upload_time = perf.now();

    if size == 0 {
        link.send_message(Msg::UploadProgressUpdate(
            path.clone(),
            0,
            0.0,
            "complete".to_string(),
            None,
        ));
        link.send_message(Msg::UploadCompleted(path.clone()));
        return;
    }

    while position < size {
        let start = position;
        let end = std::cmp::min(position + chunk_size, size);

        // Slice chunk
        let blob = match file.slice_with_f64_and_f64(start as f64, end as f64) {
            Ok(b) => b,
            Err(e) => {
                link.send_message(Msg::UploadFailed(
                    path.clone(),
                    format!("Slice failed: {:?}", e),
                ));
                failed = true;
                break;
            }
        };

        // Read chunk to Vec<u8>
        let array_buffer_val = match wasm_bindgen_futures::JsFuture::from(blob.array_buffer()).await
        {
            Ok(ab) => ab,
            Err(e) => {
                link.send_message(Msg::UploadFailed(
                    path.clone(),
                    format!("Read buffer failed: {:?}", e),
                ));
                failed = true;
                break;
            }
        };

        let array_buffer = js_sys::ArrayBuffer::from(array_buffer_val);
        let uint8_array = js_sys::Uint8Array::new(&array_buffer);
        let mut chunk_data = vec![0u8; uint8_array.length() as usize];
        uint8_array.copy_to(&mut chunk_data);

        // Upload chunk with retry logic
        let mut chunk_success = false;
        let mut chunk_error_msg = String::new();

        for attempt in 0..=max_retries {
            if attempt > 0 {
                link.send_message(Msg::UploadProgressUpdate(
                    path.clone(),
                    position,
                    0.0,
                    format!("Retrying attempt {}/{}...", attempt, max_retries),
                    Some("var(--warning-color)".to_string()),
                ));

                // Exponential backoff delay
                let delay = std::cmp::min(1000 * 2_u64.pow(attempt as u32 - 1), 30000);
                gloo_timers::future::sleep(std::time::Duration::from_millis(delay)).await;
            }

            match upload_chunk(&upload_id, &batch_id, chunk_data.clone()).await {
                Ok(_progress) => {
                    chunk_success = true;

                    // Calculate rates
                    let current_time = perf.now();
                    let time_diff = (current_time - last_upload_time) / 1000.0; // convert to secs
                    let bytes_diff = end - last_uploaded_bytes;

                    let rate = if time_diff > 0.0 {
                        bytes_diff as f64 / time_diff
                    } else {
                        0.0
                    };

                    position = end;
                    last_uploaded_bytes = position;
                    last_upload_time = current_time;

                    link.send_message(Msg::UploadProgressUpdate(
                        path.clone(),
                        position,
                        rate,
                        "uploading...".to_string(),
                        None,
                    ));
                    break;
                }
                Err(err) => {
                    // Special 404 handler on retry: assume completed
                    if attempt > 0 && err.contains("404") {
                        chunk_success = true;
                        position = size;
                        link.send_message(Msg::UploadProgressUpdate(
                            path.clone(),
                            size,
                            0.0,
                            "complete".to_string(),
                            None,
                        ));
                        break;
                    }
                    chunk_error_msg = err;
                }
            }
        }

        if !chunk_success {
            failed = true;
            let _ = cancel_upload(&upload_id).await;
            link.send_message(Msg::UploadFailed(
                path.clone(),
                format!("Chunk upload failed: {}", chunk_error_msg),
            ));
            break;
        }
    }

    if !failed {
        link.send_message(Msg::UploadCompleted(path.clone()));
    }
}
