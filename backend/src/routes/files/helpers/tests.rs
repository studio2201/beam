use super::*;

#[test]
fn dangerous_extensions_get_octet_stream() {
    for ext in [
        "html", "htm", "svg", "xhtml", "xht", "mhtml", "js", "mjs", "xml", "xsl", "pdf",
    ] {
        let filename = format!("test.{ext}");
        assert_eq!(
            get_content_type(&filename),
            "application/octet-stream",
            "extension {ext} should map to octet-stream"
        );
    }
}

#[test]
fn dangerous_extensions_case_insensitive() {
    for ext in ["HTML", "Svg", "XHTML", "JS"] {
        let filename = format!("test.{ext}");
        assert_eq!(get_content_type(&filename), "application/octet-stream");
    }
}

#[test]
fn safe_image_extensions_keep_their_types() {
    assert_eq!(get_content_type("photo.jpg"), "image/jpeg");
    assert_eq!(get_content_type("photo.png"), "image/png");
    assert_eq!(get_content_type("photo.gif"), "image/gif");
}

#[test]
fn unknown_extensions_default_to_octet_stream() {
    assert_eq!(get_content_type("file.qwxyz"), "application/octet-stream");
}

#[test]
fn content_disposition_is_always_attachment() {
    let d = create_safe_content_disposition("photo.png");
    assert!(d.starts_with("attachment;"), "got: {d}");
    assert!(d.contains("photo.png"), "got: {d}");
}

#[test]
fn content_disposition_handles_unicode_safely() {
    let d = create_safe_content_disposition("файл.txt");
    assert!(d.starts_with("attachment;"));
    assert!(d.contains("filename*=UTF-8''"), "got: {d}");
}

#[test]
fn content_disposition_sanitizes_quotes_and_backslashes() {
    let d = create_safe_content_disposition("evil\"name\\.txt");
    assert!(d.starts_with("attachment;"));
    // Quotes should be escaped or replaced
    assert!(!d.contains("evil\"name"), "raw quote in disposition: {d}");
}
