← [back to index](./index.md)

# Task 02-T02 — `audit::log_delta_sync` helper + unit test

<read_first>
- `crates/reposix-cache/src/audit.rs` (entire file)
</read_first>

<action>
Edit `crates/reposix-cache/src/audit.rs` — append:

```rust
/// Insert `op='delta_sync'` row. Written from inside the SQLite
/// transaction owned by [`crate::Cache::sync`] — the caller MUST pass
/// the `Transaction`'s connection reference (not the outer
/// `Connection`) so this row commits atomically with the `meta` and
/// `oid_map` writes.
///
/// `since_iso` is the RFC3339 string of the `last_fetched_at` that
/// was passed to the backend; `items_returned` is the count of IDs
/// the backend declared changed.
pub fn log_delta_sync_tx(
    tx: &rusqlite::Transaction<'_>,
    backend: &str,
    project: &str,
    since_iso: Option<&str>,
    items_returned: usize,
) -> rusqlite::Result<()> {
    let reason = match since_iso {
        Some(s) => format!("since={s}"),
        None => "since=NULL".to_owned(),
    };
    tx.execute(
        "INSERT INTO audit_events_cache (ts, op, backend, project, bytes, reason) \
         VALUES (?1, 'delta_sync', ?2, ?3, ?4, ?5)",
        params![
            Utc::now().to_rfc3339(),
            backend,
            project,
            i64::try_from(items_returned).unwrap_or(i64::MAX),
            reason,
        ],
    )?;
    Ok(())
}
```

Add a unit test in the same file's `#[cfg(test)] mod tests` (create the module if it doesn't exist yet — pattern after `crates/reposix-cache/src/db.rs::tests`):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_cache_db;
    use tempfile::tempdir;

    #[test]
    fn log_delta_sync_tx_inserts_row() {
        let tmp = tempdir().unwrap();
        let mut conn = open_cache_db(tmp.path()).unwrap();
        let tx = conn.transaction().unwrap();
        log_delta_sync_tx(&tx, "sim", "demo", Some("2026-04-24T00:00:00Z"), 3).unwrap();
        tx.commit().unwrap();
        let n: i64 = conn.query_row(
            "SELECT COUNT(*) FROM audit_events_cache WHERE op = 'delta_sync'",
            [], |r| r.get(0),
        ).unwrap();
        assert_eq!(n, 1);
        let (bytes, reason): (i64, String) = conn.query_row(
            "SELECT bytes, reason FROM audit_events_cache WHERE op = 'delta_sync'",
            [], |r| Ok((r.get(0)?, r.get(1)?)),
        ).unwrap();
        assert_eq!(bytes, 3);
        assert_eq!(reason, "since=2026-04-24T00:00:00Z");
    }

    #[test]
    fn log_delta_sync_tx_roll_back_does_not_leak_row() {
        let tmp = tempdir().unwrap();
        let mut conn = open_cache_db(tmp.path()).unwrap();
        let tx = conn.transaction().unwrap();
        log_delta_sync_tx(&tx, "sim", "demo", Some("2026-04-24T00:00:00Z"), 1).unwrap();
        // Intentionally drop without commit.
        drop(tx);
        let n: i64 = conn.query_row(
            "SELECT COUNT(*) FROM audit_events_cache WHERE op = 'delta_sync'",
            [], |r| r.get(0),
        ).unwrap();
        assert_eq!(n, 0, "rollback must not leave the delta_sync row");
    }
}
```
</action>

<acceptance_criteria>
- `cargo test -p reposix-cache log_delta_sync_tx_inserts_row` exits 0.
- `cargo test -p reposix-cache log_delta_sync_tx_roll_back_does_not_leak_row` exits 0 (atomicity proof at the unit level).
- `grep -n 'log_delta_sync_tx' crates/reposix-cache/src/audit.rs` finds both the def and the module-level re-export path.
</acceptance_criteria>

<threat_model>
The transaction-scoped helper is the mechanical guarantee that audit rows and meta updates are atomic. The rollback test proves that a partial sync (e.g., backend fetch failure after audit insert but before commit) does NOT leak a misleading "we synced at T" row.
</threat_model>
