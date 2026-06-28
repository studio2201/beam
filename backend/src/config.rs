//! Beam-specific configuration layered on top of shared [`ServerConfig`].
//!
//! Beam-specific fields (upload dir, file size limits, retention) live here.
//! Common fields (port, pin, enable_*, etc.) come from [`ServerConfig`].

use std::env;
use std::path::PathBuf;

use shared_backend::server::ServerConfig;

/// Beam application configuration. Wraps [`ServerConfig`] with upload and
/// retention settings that are specific to the file-sharing use case.
#[derive(Clone, Debug)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub upload_dir: PathBuf,
    pub max_file_size: u64,
    pub auto_upload: bool,
    pub show_file_list: bool,
    pub trust_proxy: bool,
    pub trusted_proxy_ips: Option<Vec<String>>,
    pub allowed_extensions: Option<Vec<String>>,
    pub client_max_retries: u32,
    pub max_storage_limit: Option<u64>,
    pub retention_period_days: Option<u64>,
    pub node_env: String,
}

impl AppConfig {
    /// Build a config by combining shared [`ServerConfig::from_env`] with
    /// beam-specific env parsing.
    pub fn load() -> Self {
        let server = ServerConfig::from_env("BEAM");

        let upload_dir = if let Ok(dir) = env::var("UPLOAD_DIR")
            && !dir.is_empty()
        {
            PathBuf::from(dir)
        } else {
            env::var("LOCAL_UPLOAD_DIR")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("./local_uploads"))
        };

        let max_file_size = parse_or("MAX_FILE_SIZE", 1024u64) * 1024 * 1024;

        Self {
            server,
            upload_dir,
            max_file_size,
            auto_upload: parse_bool("AUTO_UPLOAD"),
            show_file_list: parse_bool("SHOW_FILE_LIST"),
            trust_proxy: parse_bool("TRUST_PROXY"),
            trusted_proxy_ips: env::var("TRUSTED_PROXY_IPS").ok().map(|ips| {
                ips.split(',')
                    .map(|ip| ip.trim().to_string())
                    .filter(|ip| !ip.is_empty())
                    .collect()
            }),
            allowed_extensions: env::var("ALLOWED_EXTENSIONS")
                .ok()
                .filter(|s| !s.is_empty())
                .map(|exts| {
                    exts.split(',')
                        .map(|ext| {
                            let mut trimmed = ext.trim().to_lowercase();
                            if !trimmed.starts_with('.') {
                                trimmed.insert(0, '.');
                            }
                            trimmed
                        })
                        .collect()
                }),
            client_max_retries: parse_or("CLIENT_MAX_RETRIES", 5u32),
            max_storage_limit: env::var("MAX_STORAGE_LIMIT_GB")
                .ok()
                .and_then(|s| s.parse::<u64>().ok())
                .map(|gb| gb * 1024 * 1024 * 1024),
            retention_period_days: env::var("RETENTION_PERIOD_DAYS")
                .ok()
                .and_then(|s| s.parse().ok()),
            node_env: env::var("NODE_ENV").unwrap_or_else(|_| "production".to_string()),
        }
    }
}

fn parse_bool(name: &str) -> bool {
    env::var(name).map(|v| v == "true").unwrap_or(false)
}

fn parse_or<T>(name: &str, default: T) -> T
where
    T: std::str::FromStr,
{
    env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}
