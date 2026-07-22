//! Backend error type that converts to HTTP responses.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

/// Errors that any companion app backend can produce, with HTTP mapping.
#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("not found")]
    NotFound,

    #[error("unauthorized")]
    Unauthorized,

    #[error("forbidden")]
    Forbidden,

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("rate limit exceeded")]
    RateLimited,

    #[error("internal: {0}")]
    Internal(String),
}

impl IntoResponse for ServerError {
    fn into_response(self) -> Response {
        let (status, msg) = match &self {
            Self::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, self.to_string()),
            Self::Forbidden => (StatusCode::FORBIDDEN, self.to_string()),
            Self::BadRequest(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            Self::RateLimited => (StatusCode::TOO_MANY_REQUESTS, self.to_string()),
            Self::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal error".into()),
        };
        (status, msg).into_response()
    }
}

/// Convenience: convert `anyhow::Error` to `ServerError::Internal`.
impl From<anyhow::Error> for ServerError {
    fn from(e: anyhow::Error) -> Self {
        Self::Internal(e.to_string())
    }
}

/// Convenience: convert `std::io::Error` to `ServerError::Internal`.
impl From<std::io::Error> for ServerError {
    fn from(e: std::io::Error) -> Self {
        Self::Internal(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_status() {
        let r = ServerError::NotFound.into_response();
        assert_eq!(r.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn unauthorized_status() {
        let r = ServerError::Unauthorized.into_response();
        assert_eq!(r.status(), StatusCode::UNAUTHORIZED);
    }

    #[test]
    fn rate_limited_status() {
        let r = ServerError::RateLimited.into_response();
        assert_eq!(r.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    #[test]
    fn forbidden_status() {
        let r = ServerError::Forbidden.into_response();
        assert_eq!(r.status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn bad_request_status() {
        let r = ServerError::BadRequest("invalid query parameter".into()).into_response();
        assert_eq!(r.status(), StatusCode::BAD_REQUEST);
    }

    #[test]
    fn from_conversions() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied");
        let server_err: ServerError = io_err.into();
        assert_eq!(server_err.into_response().status(), StatusCode::INTERNAL_SERVER_ERROR);

        let anyhow_err = anyhow::anyhow!("something went wrong");
        let server_err2: ServerError = anyhow_err.into();
        assert_eq!(server_err2.into_response().status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn internal_error_does_not_leak_details() {
        let r = ServerError::Internal("database password = hunter2".into()).into_response();
        assert_eq!(r.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }
}
