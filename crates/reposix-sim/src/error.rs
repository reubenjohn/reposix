//! `ApiError` — the uniform error type for every axum handler in the sim.
//!
//! Each variant carries the minimum information the caller needs; the full
//! error chain is logged via `tracing::error!` and does NOT leak into the
//! response body (T-02-04: no rusqlite internals to clients).

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};
use thiserror::Error;

/// Every error the sim's HTTP handlers can raise.
#[derive(Debug, Error)]
pub enum ApiError {
    /// Resource absent. Produces 404.
    #[error("not found")]
    NotFound,

    /// Client-supplied input failed validation. Produces 400.
    #[error("bad request: {0}")]
    BadRequest(String),

    /// `If-Match` version did not match the current row's version. Produces 409.
    #[error("version mismatch: current={current} sent={sent:?}")]
    VersionMismatch {
        /// Server-side current version (what the client should have sent).
        current: u64,
        /// Raw If-Match value as received (without RFC-7232 quotes).
        sent: String,
    },

    /// Underlying `SQLite` error. Produces 500 (opaque body). The detailed
    /// error is logged via `tracing::error!` server-side.
    #[error("db error: {0}")]
    Db(#[from] rusqlite::Error),

    /// Underlying JSON error. Produces 400 (request-side) or 500
    /// (response-side). Handler code decides via `ApiError::BadRequest` which
    /// side of the boundary the error came from; this variant is the escape
    /// hatch for library-level Serde failures.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// Internal invariant violation (e.g. schema load returned Err, or a
    /// unicode assumption about label JSON failed). Produces 500 with an
    /// opaque body.
    #[error("internal error: {0}")]
    Internal(String),
}

impl ApiError {
    /// HTTP status for this error.
    #[must_use]
    pub fn status(&self) -> StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::VersionMismatch { .. } => StatusCode::CONFLICT,
            Self::Db(_) | Self::Json(_) | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// Stable error-kind string for the JSON body.
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::NotFound => "not_found",
            Self::BadRequest(_) => "bad_request",
            Self::VersionMismatch { .. } => "version_mismatch",
            Self::Db(_) | Self::Json(_) | Self::Internal(_) => "internal",
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status();
        let kind = self.kind();
        let body: Value = match &self {
            Self::NotFound => json!({"error": kind, "message": "not found"}),
            Self::BadRequest(msg) => json!({"error": kind, "message": msg}),
            Self::VersionMismatch { current, sent } => {
                json!({
                    "error": kind,
                    "current": current,
                    "sent": sent,
                })
            }
            // Do not leak internal details — log, then return opaque body.
            Self::Db(e) => {
                tracing::error!(error = %e, "db error");
                json!({"error": kind, "message": "internal error"})
            }
            Self::Json(e) => {
                tracing::error!(error = %e, "json error");
                json!({"error": kind, "message": "internal error"})
            }
            Self::Internal(msg) => {
                tracing::error!(error = %msg, "internal error");
                json!({"error": kind, "message": "internal error"})
            }
        };
        (status, Json(body)).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::ApiError;
    use axum::response::IntoResponse;

    #[test]
    fn version_mismatch_is_409() {
        let resp = ApiError::VersionMismatch {
            current: 5,
            sent: "bogus".into(),
        }
        .into_response();
        assert_eq!(resp.status().as_u16(), 409);
    }

    #[test]
    fn not_found_is_404() {
        let resp = ApiError::NotFound.into_response();
        assert_eq!(resp.status().as_u16(), 404);
    }

    #[test]
    fn bad_request_is_400() {
        let resp = ApiError::BadRequest("nope".into()).into_response();
        assert_eq!(resp.status().as_u16(), 400);
    }

    #[test]
    fn db_error_is_500() {
        // Connection::open on a bogus path yields an rusqlite::Error.
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        let err = conn.prepare("SELECT * FROM does_not_exist").unwrap_err();
        let resp = ApiError::Db(err).into_response();
        assert_eq!(resp.status().as_u16(), 500);
    }
}
