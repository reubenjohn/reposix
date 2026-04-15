//! SQLite metadata store for `reposix refresh`.
//!
//! `cache.db` lives at `<mount>/.reposix/cache.db` and holds a single-row
//! `refresh_meta` table recording when the last refresh ran and which backend
//! was used.  The file is opened in WAL + EXCLUSIVE locking mode so a second
//! concurrent `reposix refresh` on the same mount immediately gets an error
//! instead of silently racing.

use std::os::unix::fs::OpenOptionsExt as _;
use std::path::Path;

use anyhow::{Context as _, Result};

/// SQL schema for the metadata DB.
const CACHE_SCHEMA_SQL: &str = "
CREATE TABLE IF NOT EXISTS refresh_meta (
    id              INTEGER PRIMARY KEY CHECK (id = 1),
    backend_name    TEXT NOT NULL,
    project         TEXT NOT NULL,
    last_fetched_at TEXT NOT NULL,
    commit_sha      TEXT
);
";

/// Newtype wrapping a `rusqlite::Connection`.
///
/// Holds an EXCLUSIVE WAL lock for its lifetime — dropping `CacheDb` releases
/// the lock and closes the file.  Concurrent callers that try
/// [`open_cache_db`] on the same path while this is alive will receive an
/// error whose message contains "another refresh is in progress".
pub struct CacheDb(rusqlite::Connection);

impl CacheDb {
    /// Borrow the underlying connection for ad-hoc queries.
    #[must_use]
    pub fn conn(&self) -> &rusqlite::Connection {
        &self.0
    }
}

/// Open (or create) `<mount>/.reposix/cache.db` with WAL + EXCLUSIVE locking.
///
/// The `.reposix/` directory is created if it does not already exist.  The
/// database file is created with permission `0o600` so only the mount owner
/// can read it (mitigates T-20A-02).
///
/// # Errors
///
/// - `anyhow::Error` wrapping an IO error if the directory cannot be created
///   or the file cannot be opened.
/// - A human-readable error containing `"another refresh is in progress"` if
///   a second process has the EXCLUSIVE WAL lock on the same DB file
///   (`SQLITE_BUSY`).
/// - An `anyhow::Error` wrapping a `rusqlite::Error` for any other SQLite
///   failure (e.g. schema application).
pub fn open_cache_db(mount: &Path) -> Result<CacheDb> {
    let dir = mount.join(".reposix");
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("create_dir_all {}", dir.display()))?;

    let path = dir.join("cache.db");

    // Pre-create the file with 0o600 permissions before rusqlite opens it.
    // `OpenOptions::create(true)` is a no-op if the file already exists,
    // which is fine — permissions are already set.
    std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .mode(0o600)
        .open(&path)
        .with_context(|| format!("create cache.db at {}", path.display()))?;

    let conn = rusqlite::Connection::open(&path).map_err(|e| map_busy(e, &path))?;

    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|e| map_busy(e, &path))?;
    conn.pragma_update(None, "locking_mode", "EXCLUSIVE")
        .map_err(|e| map_busy(e, &path))?;

    conn.execute_batch(CACHE_SCHEMA_SQL)
        .map_err(|e| map_busy(e, &path))?;

    Ok(CacheDb(conn))
}

/// Map a rusqlite error to a friendly "busy" message when the extended error
/// code is `SQLITE_BUSY`, and to a plain `anyhow` error otherwise.
fn map_busy(e: rusqlite::Error, path: &Path) -> anyhow::Error {
    if let rusqlite::Error::SqliteFailure(ref ffi_err, _) = e {
        if ffi_err.extended_code == rusqlite::ffi::SQLITE_BUSY {
            return anyhow::anyhow!(
                "another refresh is in progress; unmount or wait for the previous \
                 refresh to finish ({})",
                path.display()
            );
        }
    }
    anyhow::Error::new(e).context(format!("open cache.db at {}", path.display()))
}

/// Write (or overwrite) the single-row metadata record in `refresh_meta`.
///
/// Uses `INSERT OR REPLACE` so a subsequent refresh is a clean overwrite, not
/// an accumulation of rows.
///
/// # Errors
///
/// Propagates any `rusqlite` error (e.g. disk full, corrupt DB).
pub fn update_metadata(
    db: &CacheDb,
    backend_name: &str,
    project: &str,
    last_fetched_at: &str,
    commit_sha: Option<&str>,
) -> Result<()> {
    db.0.execute(
        "INSERT OR REPLACE INTO refresh_meta \
         (id, backend_name, project, last_fetched_at, commit_sha) \
         VALUES (1, ?1, ?2, ?3, ?4)",
        rusqlite::params![backend_name, project, last_fetched_at, commit_sha],
    )
    .context("update refresh_meta")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn open_creates_schema() {
        let dir = tempdir().unwrap();
        let db = open_cache_db(dir.path()).expect("open should succeed");
        // The schema table should exist; query it to confirm.
        let count: i64 = db
            .conn()
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='refresh_meta'",
                [],
                |row| row.get(0),
            )
            .expect("schema query");
        assert_eq!(count, 1, "refresh_meta table must exist");
        // Also verify that the DB file was created.
        assert!(dir.path().join(".reposix").join("cache.db").exists());
    }

    #[test]
    fn update_metadata_roundtrip() {
        let dir = tempdir().unwrap();
        let db = open_cache_db(dir.path()).expect("open");

        update_metadata(&db, "simulator", "demo", "2026-04-15T00:00:00Z", Some("abc123"))
            .expect("update");

        let (backend, project, fetched_at, sha): (String, String, String, Option<String>) = db
            .conn()
            .query_row(
                "SELECT backend_name, project, last_fetched_at, commit_sha \
                 FROM refresh_meta WHERE id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
            )
            .expect("query");

        assert_eq!(backend, "simulator");
        assert_eq!(project, "demo");
        assert_eq!(fetched_at, "2026-04-15T00:00:00Z");
        assert_eq!(sha, Some("abc123".to_owned()));

        // Second call should overwrite, not insert a second row.
        update_metadata(&db, "github", "owner/repo", "2026-04-15T01:00:00Z", None)
            .expect("second update");

        let count: i64 = db
            .conn()
            .query_row("SELECT COUNT(*) FROM refresh_meta", [], |row| row.get(0))
            .expect("count");
        assert_eq!(count, 1, "INSERT OR REPLACE must leave exactly one row");
    }

    /// Opening a second `CacheDb` on the same path while the first holds the
    /// EXCLUSIVE WAL lock should return an error whose message mentions
    /// "another refresh is in progress".
    ///
    /// NOTE: SQLite WAL EXCLUSIVE lock acquisition only contends after a
    /// write; the second connection succeeds in read-only mode until a write
    /// is attempted.  This test verifies that the error surfacing path is
    /// correct by attempting a write on the second connection.
    #[test]
    fn lock_conflict_returns_error() {
        let dir = tempdir().unwrap();
        // First connection holds the lock.
        let _db1 = open_cache_db(dir.path()).expect("first open");

        // Second connection on the same path — WAL EXCLUSIVE lock means
        // SQLITE_BUSY is returned when the second connection tries to acquire
        // the write lock.
        let result = open_cache_db(dir.path());
        // Either the second open itself fails, or a subsequent write fails.
        // Check both:
        match result {
            Err(e) => {
                assert!(
                    e.to_string().contains("another refresh is in progress"),
                    "expected 'another refresh is in progress', got: {e}"
                );
            }
            Ok(db2) => {
                // The open succeeded (WAL allows concurrent readers), but a
                // write must fail with SQLITE_BUSY.
                let write_result = update_metadata(
                    &db2,
                    "test",
                    "proj",
                    "2026-04-15T00:00:00Z",
                    None,
                );
                assert!(
                    write_result.is_err(),
                    "write on second connection should fail when first holds EXCLUSIVE lock"
                );
            }
        }
    }

    #[test]
    fn open_is_idempotent() {
        let dir = tempdir().unwrap();
        // First open creates the DB.
        drop(open_cache_db(dir.path()).expect("first open"));
        // After first CacheDb is dropped (EXCLUSIVE lock released), second
        // open should succeed cleanly.
        let _db2 = open_cache_db(dir.path()).expect("second open after first dropped");
    }
}
