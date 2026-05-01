//! Reconciliation walk — match working-tree records to backend records
//! by `id` in frontmatter (DVCS-ATTACH-02, v0.13.0).
//!
//! Five cases per architecture-sketch.md § "Reconciliation cases":
//!
//! 1. Match: local record's id matches a backend record → row written
//!    to `cache_reconciliation`.
//! 2. Backend-deleted: local record's id has no matching backend record
//!    → warn + skip (subject to [`OrphanPolicy`]).
//! 3. No-id: local file lacks `id` frontmatter (or fails to parse) →
//!    warn + skip.
//! 4. Duplicate-id: two local files claim the same id → hard error;
//!    reconciliation aborts BEFORE any rows are written (atomicity).
//! 5. Mirror-lag: backend has id; no local file → cache notes for next
//!    fetch.
//!
//! The walker is `pub fn walk_and_reconcile(...)`; the typed result is
//! [`ReconciliationReport`]. Orphan handling is selected via
//! [`OrphanPolicy`]. POC-FINDINGS F01 — the walker accepts an
//! `ignore` slice (default `[".git", ".github"]`) so vendored docs
//! don't pollute the table on real checkouts.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use reposix_core::{frontmatter, RecordId};

use crate::cache::Cache;
use crate::error::{Error, Result};

/// Outcome of [`walk_and_reconcile`]. The CLI's stderr summary + the
/// `cache_reconciliation` table are derived from this.
#[derive(Debug, Default)]
pub struct ReconciliationReport {
    /// Count of local records matched to a backend record (case 1).
    pub matched_count: usize,
    /// Count of local files lacking parseable `id` frontmatter (case 3).
    pub no_id_count: usize,
    /// Count of local records whose `id` has no matching backend
    /// record (case 2).
    pub backend_deleted_count: usize,
    /// Count of backend records with no matching local file (case 5).
    pub mirror_lag_count: usize,
    /// Pairs of `(id, [paths])` where multiple local files claim the
    /// same id (case 4). Non-empty → reconciliation aborted; no rows
    /// written.
    pub duplicate_id_files: Vec<(RecordId, Vec<PathBuf>)>,
}

/// Policy controlling case-2 (backend-deleted) handling.
#[derive(Debug, Clone, Copy)]
pub enum OrphanPolicy {
    /// Abort attach (default).
    Abort,
    /// Delete the local file (destructive).
    DeleteLocal,
    /// Treat the local file as a new record to be created on next push.
    /// `walk_and_reconcile` only logs the intent in this scaffold; the
    /// "create on push" follow-through arrives with the bus-remote
    /// machinery in P82+.
    ForkAsNew,
}

/// Walk the working tree, parse frontmatter, write `cache_reconciliation`
/// rows for matched records.
///
/// Hard-errors on duplicate-id (case 4) — the report's
/// `duplicate_id_files` is populated and NO rows are written
/// (transaction rollback by drop). Skips no-id (case 3) and
/// backend-deleted (case 2; subject to `orphan_policy`).
///
/// `ignore` is a slice of directory names (NOT globs) to prune from
/// the walk — `.git` and `.github` are the canonical defaults
/// (POC-FINDINGS F01). Names are compared component-wise against the
/// path; a name matching ANY component prunes the entry.
///
/// # Errors
/// - Returns [`Error::Sqlite`] if the `SQLite` write fails.
/// - Returns [`Error::Backend`] if `cache.list_record_ids` errors.
/// - Returns [`Error::Git`] if `cache.find_oid_for_record` errors.
/// - Returns [`Error::Io`] on I/O failure walking the tree.
///
/// # Panics
/// Panics if the internal `cache.db` mutex is poisoned (consistent
/// with the rest of the cache crate's panic discipline — a poisoned
/// mutex means a writer panicked and process state is corrupt).
//
// The walk fans out to (a) frontmatter-classify, (b) duplicate-id
// detect, (c) backend cross-reference, (d) orphan-policy apply, (e)
// transactional INSERT. Each stage is short and the audit story is
// linear; splitting across helper fns would obscure the case ordering
// the architecture-sketch table fixes verbatim.
#[allow(clippy::too_many_lines)]
pub fn walk_and_reconcile(
    work: &Path,
    cache: &mut Cache,
    orphan_policy: OrphanPolicy,
    ignore: &[String],
) -> Result<ReconciliationReport> {
    let mut local_ids: HashMap<RecordId, Vec<PathBuf>> = HashMap::new();
    let mut no_id_files: Vec<PathBuf> = Vec::new();

    for entry in walkdir::WalkDir::new(work)
        .into_iter()
        .filter_entry(|e| !is_ignored(e.path(), ignore))
        .filter_map(std::result::Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().is_none_or(|e| e != "md") {
            continue;
        }
        let Ok(bytes) = std::fs::read_to_string(entry.path()) else {
            continue;
        };
        match frontmatter::parse(&bytes) {
            Ok(rec) => {
                local_ids
                    .entry(rec.id)
                    .or_default()
                    .push(entry.path().to_path_buf());
            }
            Err(_) => {
                no_id_files.push(entry.path().to_path_buf());
            }
        }
    }

    let mut report = ReconciliationReport {
        no_id_count: no_id_files.len(),
        ..Default::default()
    };

    // Detect duplicates BEFORE any INSERT (atomicity case 4).
    for (id, paths) in &local_ids {
        if paths.len() > 1 {
            report.duplicate_id_files.push((*id, paths.clone()));
        }
    }
    if !report.duplicate_id_files.is_empty() {
        // Caller surfaces the bail; we return early without writing.
        return Ok(report);
    }

    // Cross-reference against cache's view of backend records.
    let backend_records = cache.list_record_ids()?;
    let backend_set: HashSet<RecordId> = backend_records.iter().copied().collect();

    // Pre-compute (id, oid) pairs for matched records BEFORE opening the
    // SQLite transaction — `find_oid_for_record` itself takes the cache
    // mutex (see Cache impl), and we cannot hold the transaction's lock
    // and a borrow_mut concurrently.
    let mut match_rows: Vec<(RecordId, Option<gix::ObjectId>, PathBuf)> = Vec::new();
    let mut deleted_rows: Vec<(RecordId, PathBuf)> = Vec::new();
    for (id, paths) in &local_ids {
        let path = paths[0].clone();
        if backend_set.contains(id) {
            let oid = cache.find_oid_for_record(*id)?;
            match_rows.push((*id, oid, path));
        } else {
            deleted_rows.push((*id, path));
        }
    }

    // Apply orphan policy for case 2 (backend-deleted).
    for (id, path) in &deleted_rows {
        match orphan_policy {
            OrphanPolicy::Abort => {
                eprintln!("BACKEND_DELETED id={} local_file={}", id.0, path.display());
            }
            OrphanPolicy::DeleteLocal => {
                let _ = std::fs::remove_file(path);
                eprintln!(
                    "BACKEND_DELETED id={} local_file={} action=DELETED",
                    id.0,
                    path.display(),
                );
            }
            OrphanPolicy::ForkAsNew => {
                eprintln!(
                    "BACKEND_DELETED id={} local_file={} action=FORK_AS_NEW (TODO P82+)",
                    id.0,
                    path.display(),
                );
            }
        }
        report.backend_deleted_count += 1;
    }

    // Case 5: backend has, local lacks → mirror-lag.
    for backend_id in &backend_set {
        if !local_ids.contains_key(backend_id) {
            report.mirror_lag_count += 1;
        }
    }

    // Case 3: emit warnings for no-id files.
    for f in &no_id_files {
        eprintln!("NO_ID local_file={}", f.display());
    }

    // Write case-1 rows in a single transaction.
    let now = chrono::Utc::now().to_rfc3339();
    {
        let mut conn = cache.connection_mut()?;
        let tx = conn.transaction()?;
        for (id, oid, path) in &match_rows {
            let oid_hex = oid.map(|o| o.to_hex().to_string()).unwrap_or_default();
            tx.execute(
                "INSERT OR REPLACE INTO cache_reconciliation \
                 (record_id, oid, local_path, attached_at) \
                 VALUES (?1, ?2, ?3, ?4)",
                rusqlite::params![
                    i64::try_from(id.0).map_err(|_| Error::Backend(format!(
                        "RecordId {} too large for SQLite INTEGER",
                        id.0
                    )))?,
                    oid_hex,
                    path.to_string_lossy(),
                    &now,
                ],
            )?;
            report.matched_count += 1;
        }
        tx.commit()?;
    }

    Ok(report)
}

/// Return `true` iff any component of `path` matches any name in
/// `ignore`. Used as the prune predicate for `WalkDir::filter_entry`.
fn is_ignored(path: &Path, ignore: &[String]) -> bool {
    if ignore.is_empty() {
        return false;
    }
    path.components().any(|c| {
        let name = c.as_os_str();
        ignore.iter().any(|n| std::ffi::OsStr::new(n) == name)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// DVCS-ATTACH-04 reframed (part 1 — type-system assertion).
    ///
    /// This test compiles iff `Cache::read_blob` returns
    /// `Tainted<Vec<u8>>`. If a future refactor changes the return
    /// type, this test fails to compile and grades the OP-2 invariant
    /// RED at the type-system level.
    ///
    /// Part 2 (the integration test that forces one runtime
    /// materialization after attach) lands in 79-03 T02.
    //
    // The `_*` underscore prefixes are intentional — they signal the
    // type-only sink + dead-branch fixture pattern. Clippy's
    // `used_underscore_*` lints would normally flag using such names,
    // but the underscore IS the convention here (the names are
    // documentation, not "unused but wired").
    #[allow(clippy::used_underscore_binding, clippy::used_underscore_items)]
    #[tokio::test]
    async fn cache_read_blob_returns_tainted_type() {
        // Compile-time guarantee: declare a function that ONLY accepts
        // `Tainted<Vec<u8>>`. Feeding the result of `read_blob` into it
        // pins the return type via type-inference.
        fn _is_tainted(_: reposix_core::Tainted<Vec<u8>>) {}

        // The body is gated by `if false` so the call never executes
        // at runtime — we don't need a populated cache, only a
        // syntactically-valid call site to lock the type-check.
        if false {
            // Intentionally unreachable: feeds the read_blob future
            // into the type-only sink.
            #[allow(
                clippy::diverging_sub_expression,
                clippy::unreachable,
                unreachable_code
            )]
            {
                let _cache: Cache = unreachable!("compile-time only");
                let oid = gix::ObjectId::null(gix::hash::Kind::Sha1);
                let bytes: reposix_core::Tainted<Vec<u8>> = _cache.read_blob(oid).await.unwrap();
                _is_tainted(bytes);
            }
        }
        // Test passes if it compiles.
    }
}
