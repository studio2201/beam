use crate::utils;
use std::path::{Path, PathBuf};

#[test]
fn test_normalize_path() {
    let path = Path::new("foo/bar/../baz");
    let norm = utils::normalize_path(path);
    assert_eq!(norm, PathBuf::from("foo/baz"));
}

#[test]
fn test_sanitize_filename_safe() {
    let clean = utils::sanitize_filename_safe("my file+name.txt");
    assert_eq!(clean, "my_file_name.txt");

    let clean_reserved = utils::sanitize_filename_safe("CON.txt");
    assert_eq!(clean_reserved, "CON_file.txt");

    let clean_empty = utils::sanitize_filename_safe("");
    assert_eq!(clean_empty, "unnamed_file.txt");
}

#[test]
fn test_sanitize_path_preserve_dirs_safe() {
    let clean = utils::sanitize_path_preserve_dirs_safe("folder/sub folder/my file.txt");
    assert_eq!(clean, "folder/sub_folder/my_file.txt");
}

#[test]
fn test_sanitize_path_preserve_dirs_safe_rejects_parent_traversal() {
    let clean = utils::sanitize_path_preserve_dirs_safe("foo/../etc/passwd");
    assert_eq!(
        clean, "unnamed_file.txt",
        "parent-traversal components must be rejected"
    );
    let clean2 = utils::sanitize_path_preserve_dirs_safe("../etc/passwd");
    assert_eq!(clean2, "unnamed_file.txt");
    let clean3 = utils::sanitize_path_preserve_dirs_safe("foo/..bar/baz");
    assert_eq!(
        clean3, "foo/file.bar/baz",
        "leading dots are stripped from each part"
    );
}

#[test]
fn test_format_file_size() {
    assert_eq!(utils::format_file_size(500, None), "500.00B");
    assert_eq!(utils::format_file_size(2048, None), "2.00KB");
    assert_eq!(utils::format_file_size(2097152, Some("MB")), "2.00MB");
}

#[test]
fn test_is_valid_batch_id() {
    assert!(utils::is_valid_batch_id("1719000000-abcdef12"));
    assert!(!utils::is_valid_batch_id("1719000000-abc"));
    assert!(!utils::is_valid_batch_id("abc-abcdef12"));
}

fn check_under_upload(file_path: &Path, upload_dir: &Path) -> bool {
    utils::is_path_within_upload_dir(file_path, upload_dir, false)
}

#[test]
fn test_is_path_within_upload_dir_accepts_normal_paths() {
    let tmp = std::env::temp_dir().join(format!(
        "beam_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&tmp).unwrap();

    let existing_file = tmp.join("real_file.txt");
    std::fs::write(&existing_file, "ok").unwrap();

    assert!(check_under_upload(&existing_file, &tmp));
    assert!(check_under_upload(
        &tmp.join("subdir/never_created.txt"),
        &tmp
    ));

    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn test_is_path_within_upload_dir_rejects_absolute_paths() {
    let tmp = std::env::temp_dir().join("beam_test_abs");
    std::fs::create_dir_all(&tmp).unwrap();
    assert!(!check_under_upload(Path::new("/etc/passwd"), &tmp));
    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn test_is_path_within_upload_dir_rejects_parent_traversal_in_nonexistent_suffix() {
    let tmp = std::env::temp_dir().join(format!(
        "beam_test_traversal_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&tmp).unwrap();

    let evil = tmp.join("legit_dir/../escape.txt");
    assert!(!check_under_upload(&evil, &tmp));

    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn test_is_path_within_upload_dir_rejects_when_parent_is_symlink_outside() {
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let outer = std::env::temp_dir().join(format!(
            "beam_outer_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let upload = outer.join("uploads");
        std::fs::create_dir_all(&upload).unwrap();
        let outside = outer.join("outside");
        std::fs::create_dir_all(&outside).unwrap();
        let link = upload.join("escape");
        symlink(&outside, &link).unwrap();

        let evil_target = link.join("passwd");
        assert!(
            !check_under_upload(&evil_target, &upload),
            "symlinked subdir must be detected as outside the upload dir"
        );

        std::fs::remove_dir_all(&outer).ok();
    }
}

#[test]
fn test_is_path_within_upload_dir_require_exists_canonicalizes() {
    let tmp = std::env::temp_dir().join(format!(
        "beam_test_exists_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&tmp).unwrap();
    let real = tmp.join("ok.txt");
    std::fs::write(&real, "data").unwrap();
    assert!(utils::is_path_within_upload_dir(&real, &tmp, true));
    assert!(!utils::is_path_within_upload_dir(
        Path::new("/this/does/not/exist"),
        &tmp,
        true
    ));
    std::fs::remove_dir_all(&tmp).ok();
}

#[test]
fn test_property_sanitize_path_never_panics() {
    let long_string = "a/".repeat(500);
    let inputs = vec![
        "",
        "   ",
        "\0",
        "\x00\x01\x02",
        "../../../../etc/passwd",
        "C:\\Windows\\System32\\cmd.exe",
        "CON.txt",
        "AUX",
        "   ..   ",
        "🔥🦀🎉",
        long_string.as_str(),
        "../../..//..//",
        "hello\0world",
    ];

    for input in inputs {
        let clean = utils::sanitize_filename_safe(input);
        assert!(!clean.contains('\0'));
        let path_clean = utils::sanitize_path_preserve_dirs_safe(input);
        assert!(!path_clean.contains('\0'));
    }
}

#[test]
fn test_property_is_path_within_upload_dir_never_panics() {
    let tmp = std::env::temp_dir();
    let malformed_paths = vec![
        "",
        "/",
        "relative/path",
        "../../outside",
        "/etc/passwd",
        "\0",
        "C:\\Windows",
    ];

    for path_str in malformed_paths {
        let p = Path::new(path_str);
        let _ = utils::is_path_within_upload_dir(p, &tmp, false);
        let _ = utils::is_path_within_upload_dir(p, &tmp, true);
    }
}

#[test]
fn test_property_format_file_size_never_panics() {
    let sizes = vec![0, 1, 1024, 1048576, u64::MAX];
    let units = vec![
        None,
        Some("B"),
        Some("KB"),
        Some("MB"),
        Some("GB"),
        Some("TB"),
        Some("INVALID"),
    ];

    for size in sizes {
        for unit in &units {
            let res = utils::format_file_size(size, *unit);
            assert!(!res.is_empty());
        }
    }
}
