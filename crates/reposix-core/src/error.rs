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

    /// Issue body / file format violation.
    #[error("invalid issue file: {0}")]
    InvalidIssue(String),

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
}
