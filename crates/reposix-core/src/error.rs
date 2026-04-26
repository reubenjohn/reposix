//! Error types shared by every crate.

use thiserror::Error;

/// Convenience alias for results in the reposix workspace.
pub type Result<T> = std::result::Result<T, Error>;

/// All errors that can flow across crate boundaries.
#[derive(Debug, Error)]
pub enum Error {
    /// Frontmatter parsing or serialization failure.
    #[error("frontmatter: {0}")]
    Frontmatter(String),

    /// Record body / file format violation.
    #[error("invalid record file: {0}")]
    InvalidRecord(String),

    /// Remote URL could not be parsed into a [`RemoteSpec`](crate::RemoteSpec).
    #[error("invalid remote url: {0}")]
    InvalidRemote(String),

    /// Underlying I/O error.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// JSON serialization error.
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    /// YAML serialization error.
    #[error(transparent)]
    Yaml(#[from] serde_yaml::Error),

    /// Untyped error escape hatch — only for cases where typing the error adds no value.
    #[error("{0}")]
    Other(String),

    /// URL rejected by the egress allowlist (SG-01).
    #[error("blocked origin: {0}")]
    InvalidOrigin(String),

    /// Path/filename rejected by the path validator (SG-04).
    #[error("invalid path: {0}")]
    InvalidPath(String),

    /// Underlying HTTP/transport error from reqwest.
    #[error(transparent)]
    Http(#[from] reqwest::Error),

    /// The requested record does not exist on the backend.
    ///
    /// Migrated from `Error::Other(format!("not found: {context}"))` —
    /// see code-quality audit P1-1 (POLISH2-09 partial). Display string
    /// retains the `"not found:"` prefix so existing substring-matching
    /// callers (e.g. `reposix-swarm`'s `ErrorKind::classify`) continue to
    /// classify correctly during the migration.
    #[error("not found: {project}/{id}")]
    NotFound {
        /// Project / space / repo slug the lookup targeted.
        project: String,
        /// Record id (issue id, page id, etc.) that was not found.
        id: String,
    },

    /// The backend received the request but does not support the operation
    /// (e.g., a read-only connector being asked to update). Reserved for
    /// future migration of the read-only-backend disambiguator currently
    /// emitted as `Error::Other("not supported: ...")` by adapters such as
    /// `reposix-jira` (full migration scheduled for v0.12.0).
    #[error("not supported: {operation}")]
    NotSupported {
        /// Human-readable name of the unsupported operation.
        operation: String,
    },

    /// Optimistic concurrency check failed: the caller's `version` does
    /// not match the backend's current version.
    ///
    /// Migrated from `Error::Other(format!("version mismatch: {body}"))` —
    /// see code-quality audit P1-1 + P1-5 (POLISH2-09 partial). Closes the
    /// stringly-typed protocol where downstream callers used to recover the
    /// rejection body via `msg.strip_prefix("version mismatch: ")` plus
    /// `serde_json::from_str` — pattern-match on `body` instead.
    #[error("version mismatch: current={current} requested={requested}")]
    VersionMismatch {
        /// Backend's current version (as reported in the rejection body).
        current: String,
        /// Caller-supplied `If-Match` / `expected_version` value.
        requested: String,
        /// Full backend response body for callers that want to inspect
        /// the rejection (e.g., the simulator's HTTP 409 JSON).
        body: String,
    },
}
