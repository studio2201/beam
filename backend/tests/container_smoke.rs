//! Container smoke tests for `beam`.
//!
//! Run against a live container started with `BEAM_PIN` set:
//!   docker run -e BEAM_PIN=<pin> -p <port>:4401 ...
//!   SMOKE_PORT=<port> SMOKE_PIN=<pin> \
//!     cargo test --test container_smoke -- --ignored --nocapture

use reqwest::Client;
use serde_json::Value;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const APP_NAME: &str = "beam";
const DEFAULT_PORT: u16 = 4401;

// Beam uses custom static-file routes (no /assets/ prefix).
const FAVICON_CANDIDATES: &[&str] = &["/favicon.png", "/favicon.svg", "/assets/favicon.png"];
const MANIFEST_CANDIDATES: &[&str] = &["/manifest.json", "/assets/manifest.json"];
const CONFIG_CANDIDATES: &[&str] = &["/api/auth/config", "/api/config"];
const SERVICE_WORKER_CANDIDATES: &[&str] = &[
    "/service-worker.js",
    "/api/service-worker.js",
    "/assets/service-worker.js",
];

fn port() -> u16 {
    std::env::var("SMOKE_PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(DEFAULT_PORT)
}

fn pin() -> String {
    std::env::var("SMOKE_PIN").unwrap_or_else(|_| "1234".to_string())
}

fn base_url() -> String {
    format!("http://127.0.0.1:{}", port())
}

fn client() -> Client {
    Client::builder()
        .cookie_store(true)
        .timeout(Duration::from_secs(10))
        .build()
        .expect("reqwest client")
}

fn unique_id() -> String {
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("smoke-{}-{}", std::process::id(), ms)
}

async fn wait_for_health() {
    let c = client();
    for _ in 0..30 {
        if let Ok(r) = c.get(format!("{}/health", base_url())).send().await {
            if r.status().is_success() {
                return;
            }
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
    }
    panic!("container at {} never became healthy", base_url());
}

async fn try_paths(c: &Client, paths: &[&str]) -> Option<reqwest::Response> {
    for p in paths {
        if let Ok(r) = c.get(format!("{}{}", base_url(), p)).send().await {
            if r.status().is_success() {
                return Some(r);
            }
        }
    }
    None
}

async fn login(c: &Client) {
    let r = c
        .post(format!("{}/api/auth/verify-pin", base_url()))
        .header("Origin", base_url())
        .header("Referer", format!("{}/", base_url()))
        .json(&serde_json::json!({ "pin": pin() }))
        .send()
        .await
        .unwrap();
    assert!(r.status().is_success(), "auth login failed: {}", r.status());
}

// ---------- common tests ----------

#[tokio::test]
#[ignore]
async fn health_returns_200() {
    let c = client();
    let r = c.get(format!("{}/health", base_url())).send().await.unwrap();
    assert_eq!(r.status(), 200, "expected 200 from /health");
}

#[tokio::test]
#[ignore]
async fn root_serves_html() {
    let c = client();
    let r = c.get(&base_url()).send().await.unwrap();
    assert_eq!(r.status(), 200, "expected 200 from /");
    let ct = r
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(ct.starts_with("text/html"), "expected text/html, got {ct:?}");
}

#[tokio::test]
#[ignore]
async fn favicon_resolves() {
    let c = client();
    let r = try_paths(&c, FAVICON_CANDIDATES)
        .await
        .unwrap_or_else(|| panic!("no favicon path returned 2xx: {FAVICON_CANDIDATES:?}"));
    let ct = r
        .headers()
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    assert!(
        ct.starts_with("image/") || ct.starts_with("application/octet-stream"),
        "expected image/* (or octet-stream), got {ct:?}"
    );
}

#[tokio::test]
#[ignore]
async fn manifest_parses_as_pwa() {
    let c = client();
    let r = try_paths(&c, MANIFEST_CANDIDATES)
        .await
        .unwrap_or_else(|| panic!("no manifest path returned 2xx: {MANIFEST_CANDIDATES:?}"));
    let v: Value = r.json().await.unwrap();
    assert!(v["name"].is_string(), "manifest.name must be a string, got {v:?}");
    assert!(v["icons"].is_array(), "manifest.icons must be an array");
}

#[tokio::test]
#[ignore]
async fn config_endpoint_has_site_title() {
    let c = client();
    let r = try_paths(&c, CONFIG_CANDIDATES)
        .await
        .unwrap_or_else(|| panic!("no config path returned 2xx: {CONFIG_CANDIDATES:?}"));
    let v: Value = r.json().await.unwrap();
    let title = v["siteTitle"]
        .as_str()
        .or_else(|| v["site_title"].as_str())
        .unwrap_or("");
    assert!(
        title.eq_ignore_ascii_case(APP_NAME),
        "expected siteTitle == {APP_NAME:?}, got {title:?}"
    );
}

#[tokio::test]
#[ignore]
async fn service_worker_or_frontend_serves() {
    let c = client();
    let r = try_paths(&c, SERVICE_WORKER_CANDIDATES).await;
    assert!(
        r.is_some(),
        "no service-worker path returned 2xx: {SERVICE_WORKER_CANDIDATES:?}"
    );
}

// ---------- per-app tests: beam (upload flow) ----------

#[tokio::test]
#[ignore]
async fn upload_init_returns_upload_id() {
    wait_for_health().await;
    let c = client();
    login(&c).await;

    let filename = format!("{}.txt", unique_id());
    // Beam's `InitUploadPayload` expects `fileSize` (camelCase), not `size`.
    let r = c
        .post(format!("{}/api/upload/init", base_url()))
        .header("Origin", base_url())
        .header("Referer", format!("{}/", base_url()))
        .json(&serde_json::json!({
            "filename": filename,
            "fileSize": 11u64,
            "contentType": "text/plain",
        }))
        .send()
        .await
        .unwrap();
    assert!(
        r.status().is_success(),
        "upload init must succeed (2xx), got {}",
        r.status()
    );
    let v: Value = r.json().await.unwrap();
    assert!(
        v["uploadId"].is_string() || v["id"].is_string(),
        "upload init must return an uploadId or id field, got {v:?}"
    );
}

#[tokio::test]
#[ignore]
async fn files_list_returns_array() {
    wait_for_health().await;
    let c = client();
    login(&c).await;
    let r = c
        .get(format!("{}/api/files", base_url()))
        .send()
        .await
        .unwrap();
    assert_eq!(r.status(), 200, "expected 200 from /api/files");
    let v: Value = r.json().await.unwrap();
    assert!(
        v.is_array()
            || v["files"].is_array()
            || v["items"].is_array(),
        "files list must be array or {{files: []}} or {{items: []}}, got {v:?}"
    );
}
