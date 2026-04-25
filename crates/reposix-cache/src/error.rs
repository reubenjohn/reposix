//! Typed error for the reposix-cache crate.

use thiserror::Error;

/// Errors produced by the reposix-cache crate.
#[derive(Debug, Error)]
pub enum Error {
    /// I/O failure (directory creation, file open).
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    /// Backend (`BackendConnector`) returned an error. In Plan 02 the
    /// egress-allowlist sub-case is split out as [`Error::Egress`] so the
    /// audit layer can distinguish egress denial from other failures.
    #[error("backend: {0}")]
    Backend(String),

    /// gix operation failed (init, `write_object`, `edit_tree`, commit,
    /// `edit_reference`). Stringified to keep the variant small — the
    /// underlying gix error types form a large, version-unstable taxonomy.
    #[error("git: {0}")]
    Git(String),

    /// Rendering issue to canonical on-disk bytes failed.
    #[error("render: {0}")]
    Render(#[from] reposix_core::Error),

    /// `SQLite` failure. Plan 01 does not yet open a DB; Plan 02 wires
    /// this variant to the actual audit + meta store.
    #[error("sqlite: {0}")]
    Sqlite(String),

    /// Cache path already belongs to a different `(backend, project)`
    /// than the one passed to [`crate::Cache::open`]. Plan 02 scaffolds
    /// the identity check in [`crate::Cache::open`]; Phase 33 tightens.
    #[error("cache collision: expected {expected}, found {found}")]
    CacheCollision {
        /// The `(backend, project)` the caller asked for.
        expected: String,
        /// The `(backend, project)` currently recorded in `meta`.
        found: String,
    },

    /// Outbound HTTP origin not in `REPOSIX_ALLOWED_ORIGINS`. Distinct
    /// from [`Error::Backend`] so callers (and the audit layer) can
    /// branch on denial vs generic backend failure.
    #[error("egress denied: {0}")]
    Egress(String),

    /// Blob OID requested by a consumer (e.g. Phase 32's helper) has
    /// no entry in `oid_map` — the cache has never tracked this blob.
    #[error("unknown blob oid: {0}")]
    UnknownOid(String),

    /// Backend returned bytes whose computed blob OID differs from the
    /// OID recorded in `oid_map`. Indicates an eventual-consistency
    /// race on the backend side (same issue id, different content
    /// between `list_records` and `get_record`).
    #[error("oid drift: requested {requested}, backend returned {actual} for issue {issue_id}")]
    OidDrift {
        /// OID recorded in `oid_map` at `build_from` time.
        requested: String,
        /// OID produced by hashing the bytes the backend just returned.
        actual: String,
        /// Issue id whose bytes drifted.
        issue_id: String,
    },
}

/// Alias for this crate's `Result`.
pub type Result<T> = std::result::Result<T, Error>;

impl From<gix::init::Error> for Error {
    fn from(e: gix::init::Error) -> Self {
        Self::Git(e.to_string())
    }
}

impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        Self::Sqlite(e.to_string())
    }
}
