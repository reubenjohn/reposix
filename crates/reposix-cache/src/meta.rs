//! Non-append-only tables: `meta` (key/value) and `oid_map`
//! (content-addressable lookup).

use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension as _};

use crate::error::{Error, Result};

/// Upsert a single key/value pair into `meta`. `updated_at` is set to now.
///
/// # Errors
/// Returns [`Error::Sqlite`] for any rusqlite failure.
pub fn set_meta(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO meta (key, value, updated_at) VALUES (?1, ?2, ?3) \
         ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        params![key, value, Utc::now().to_rfc3339()],
    )?;
    Ok(())
}

/// Look up a `meta` value. Returns `None` if the key is not present.
///
/// # Errors
/// Returns [`Error::Sqlite`] for any rusqlite failure other than no-rows.
pub fn get_meta(conn: &Connection, key: &str) -> Result<Option<String>> {
    conn.query_row(
        "SELECT value FROM meta WHERE key = ?1",
        params![key],
        |row| row.get::<_, String>(0),
    )
    .optional()
    .map_err(Error::from)
}

/// Insert `(oid, issue_id)` into `oid_map`. `INSERT OR REPLACE` so
/// re-syncs are clean overwrites.
///
/// # Errors
/// Returns [`Error::Sqlite`] for any rusqlite failure.
pub fn put_oid_mapping(
    conn: &Connection,
    backend: &str,
    project: &str,
    oid_hex: &str,
    issue_id: &str,
) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO oid_map (oid, issue_id, backend, project) VALUES (?1, ?2, ?3, ?4)",
        params![oid_hex, issue_id, backend, project],
    )?;
    Ok(())
}

/// Delete `oid_map` rows for `(backend, project)` whose `issue_id` is NOT in
/// `keep_ids` — the DELETION-direction half of the tree↔`oid_map` coherence
/// invariant (ADR-010 / RBF-LR-02). `keep_ids` is the current `list_records`
/// id set the caller just rebuilt the HEAD tree from; any `oid_map` row for an
/// id absent from it belongs to an upstream-deleted record. Left in place,
/// that ghost row is resurrected by `Cache::list_record_ids` (an unfiltered
/// `SELECT DISTINCT issue_id`), fed to the export planner as `prior`, and
/// turned into a phantom `PlannedAction::Delete`; the backend 404s the
/// already-gone id and the helper forces a FALSE `SotPartialFail` plus a false
/// `helper_push_partial_fail_sot` audit row on EVERY push
/// (`.planning/CONSULT-DECISIONS.md` § D-P93-01/02). // banned-words: ok
///
/// Deletes ALL rows for a vanished id — including historical oids
/// `put_oid_mapping` retains for reverse lookup (`get_issue_for_oid`). That is
/// correct: a record the `SoT` removed can never be legitimately re-fetched, so
/// the HEAD tree no longer references any of its blobs and no lazy fetch will
/// ask for them. Rows for a STILL-present id are never touched (its id is in
/// `keep_ids`, so `NOT IN` excludes every one of its rows), preserving the
/// historical-version reverse-lookup guarantee for live records.
///
/// Callers MUST invoke this inside the same transaction as the sibling
/// `put_oid_mapping` upserts so a crash cannot half-apply the coherence
/// restore. Returns the number of ghost rows pruned.
///
/// # Errors
/// Returns [`Error::Sqlite`] for any rusqlite failure.
pub fn prune_oid_map(
    conn: &Connection,
    backend: &str,
    project: &str,
    keep_ids: &[&str],
) -> Result<usize> {
    if keep_ids.is_empty() {
        // Every record was deleted upstream — no id survives, so every
        // oid_map row for this (backend, project) is a ghost.
        let n = conn.execute(
            "DELETE FROM oid_map WHERE backend = ?1 AND project = ?2",
            params![backend, project],
        )?;
        return Ok(n);
    }
    // Placeholders ?3.. for the keep set (?1/?2 are backend/project). NOT IN
    // (<keep set>) targets exactly the ids that vanished from list_records.
    let placeholders = (0..keep_ids.len())
        .map(|i| format!("?{}", i + 3))
        .collect::<Vec<_>>()
        .join(", ");
    let sql = format!(
        "DELETE FROM oid_map WHERE backend = ?1 AND project = ?2 \
         AND issue_id NOT IN ({placeholders})"
    );
    let mut binds: Vec<&dyn rusqlite::ToSql> = Vec::with_capacity(keep_ids.len() + 2);
    binds.push(&backend);
    binds.push(&project);
    for id in keep_ids {
        binds.push(id);
    }
    let n = conn.execute(&sql, binds.as_slice())?;
    Ok(n)
}

/// Look up the `issue_id` for a blob OID. Returns `None` if unknown.
///
/// # Errors
/// Returns [`Error::Sqlite`] for any rusqlite failure.
pub fn get_issue_for_oid(conn: &Connection, oid_hex: &str) -> Result<Option<String>> {
    conn.query_row(
        "SELECT issue_id FROM oid_map WHERE oid = ?1",
        params![oid_hex],
        |row| row.get::<_, String>(0),
    )
    .optional()
    .map_err(Error::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_cache_db;
    use tempfile::tempdir;

    /// `(oid, issue_id)` pairs currently in `oid_map` for `(sim, demo)`,
    /// sorted for stable assertions.
    fn rows(conn: &Connection) -> Vec<(String, String)> {
        let mut stmt = conn
            .prepare(
                "SELECT oid, issue_id FROM oid_map \
                 WHERE backend = 'sim' AND project = 'demo' ORDER BY oid",
            )
            .unwrap();
        let mut out: Vec<(String, String)> = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .map(std::result::Result::unwrap)
            .collect();
        out.sort();
        out
    }

    #[test]
    fn prune_drops_absent_ids_but_retains_every_row_of_present_ids() {
        let tmp = tempdir().unwrap();
        let conn = open_cache_db(tmp.path()).unwrap();
        // Issue 1 has TWO historical oids (a prior version + current) — the
        // reverse-lookup rows `find_oid_for_record` keeps for live records.
        put_oid_mapping(&conn, "sim", "demo", "aa", "1").unwrap();
        put_oid_mapping(&conn, "sim", "demo", "bb", "1").unwrap();
        // Issue 2 is the one about to be deleted upstream.
        put_oid_mapping(&conn, "sim", "demo", "cc", "2").unwrap();
        // A different (backend, project) that must be untouched by scoping.
        put_oid_mapping(&conn, "sim", "other", "dd", "2").unwrap();

        // list_records now returns only issue 1 (issue 2 deleted upstream).
        let pruned = prune_oid_map(&conn, "sim", "demo", &["1"]).unwrap();
        assert_eq!(pruned, 1, "exactly issue 2's single row should be pruned");

        // Both of issue 1's historical rows survive; issue 2's is gone.
        assert_eq!(
            rows(&conn),
            vec![
                ("aa".to_string(), "1".to_string()),
                ("bb".to_string(), "1".to_string())
            ],
        );
        // Scoping: the (sim, other) row is untouched.
        let other: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM oid_map WHERE backend = 'sim' AND project = 'other'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            other, 1,
            "prune must not cross the (backend, project) scope"
        );
    }

    #[test]
    fn prune_with_empty_keep_set_clears_the_whole_scope() {
        // The all-records-deleted-upstream case: keep set empty → every
        // oid_map row for (sim, demo) is a ghost and must go, while a row in
        // a sibling scope survives.
        let tmp = tempdir().unwrap();
        let conn = open_cache_db(tmp.path()).unwrap();
        put_oid_mapping(&conn, "sim", "demo", "aa", "1").unwrap();
        put_oid_mapping(&conn, "sim", "demo", "bb", "2").unwrap();
        put_oid_mapping(&conn, "sim", "other", "cc", "1").unwrap();

        let pruned = prune_oid_map(&conn, "sim", "demo", &[]).unwrap();
        assert_eq!(
            pruned, 2,
            "both (sim, demo) rows are ghosts with no survivors"
        );
        assert!(rows(&conn).is_empty(), "(sim, demo) scope fully cleared");
        let other: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM oid_map WHERE backend = 'sim' AND project = 'other'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(other, 1, "empty-keep prune must stay within its scope");
    }
}
