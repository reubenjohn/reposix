//! Typed error for the reposix-cache crate.

use thiserror::Error;

/// Errors produced by the reposix-cache crate.
#[derive(Debug, Error)]
pub enum Error {
    /// I/O failure (directory creation, file open).
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    /// Backend (`BackendConnector`) returned an error. The
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

    /// `SQLite` failure from the audit + meta store.
    #[error("sqlite: {0}")]
    Sqlite(String),

    /// Cache path already belongs to a different `(backend, project)`
    /// than the one passed to [`crate::Cache::open`]. The identity check
    /// fires in [`crate::Cache::open`].
    #[error(
        "cache collision: this cache dir belongs to {found} but was opened for {expected}. \
         Most often this is the GitHub owner/repo identity migration (S-260707-gh404): a \
         pre-fix reposix recorded the sanitized slug (e.g. github:owner-repo) as the cache \
         identity, while the fixed binary uses the raw slug (github:owner/repo). Delete the \
         stale cache dir and re-run `reposix init`/`reposix attach` to rebuild it from the \
         backend."
    )]
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

    /// Blob OID requested by a consumer (e.g. the `git-remote-reposix`
    /// stateless-connect handler) has no entry in `oid_map` — the cache
    /// has never tracked this blob.
    #[error("unknown blob oid: {0}")]
    UnknownOid(String),

    /// Backend returned bytes whose computed blob OID differs from the
    /// OID recorded in `oid_map` at `build_from` time. Two distinct
    /// causes produce this, and they have OPPOSITE recovery answers:
    ///
    /// 1. A genuine **eventual-consistency race** — the backend's content
    ///    for this id actually changed between the `list_records` walk
    ///    (which seeded the `oid_map` oid) and the later `get_record`.
    ///    `reposix sync --reconcile` CAN heal this: a fresh `list_records`
    ///    walk re-hashes the now-current content and the oid realigns.
    ///
    /// 2. A **systematic backend rendering-representation mismatch** — the
    ///    same id renders to DIFFERENT bytes via `list_records` vs
    ///    `get_record` regardless of timing (e.g. the pre-fix Confluence
    ///    LIST path requested no `body-format`, so listed pages carried an
    ///    empty body while `get_record` returned the real ADF body). This
    ///    class is closed for ADF-native pages by the adapter render-parity
    ///    fix; where it is NOT closed, `--reconcile` CANNOT heal it — a
    ///    re-list reproduces the SAME mismatched oid deterministically, so
    ///    only the adapter/backend fix removes the drift (reproduction:
    ///    `crates/reposix-cache/tests/oid_drift_reconcile.rs`).
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
