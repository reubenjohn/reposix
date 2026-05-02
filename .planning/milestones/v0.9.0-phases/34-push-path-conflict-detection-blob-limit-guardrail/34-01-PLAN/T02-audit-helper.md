← [back to index](./index.md)

# Task 01-T02 — Add `log_blob_limit_exceeded` audit helper + Cache method

<read_first>
- `crates/reposix-cache/src/audit.rs` (entire file — pattern off `log_helper_fetch_error`)
- `crates/reposix-cache/src/cache.rs:115-163` (existing wrapper methods)
</read_first>

<action>
Edit `crates/reposix-cache/src/audit.rs`. After `log_helper_fetch_error` (around line 153), add:

```rust
/// Insert `op='blob_limit_exceeded'` row — one per `command=fetch` request
/// that would have wanted more blobs than `REPOSIX_BLOB_LIMIT` allows.
/// `bytes` records the would-be want count; `reason` records `limit=<M>`.
/// Best-effort: SQL errors WARN-log (the helper has already written the
/// agent-facing stderr message and is about to exit non-zero).
pub fn log_blob_limit_exceeded(
    conn: &Connection,
    backend: &str,
    project: &str,
    want_count: u32,
    limit: u32,
) {
    let reason = format!("limit={limit}");
    let res = conn.execute(
        "INSERT INTO audit_events_cache (ts, op, backend, project, bytes, reason) \
         VALUES (?1, 'blob_limit_exceeded', ?2, ?3, ?4, ?5)",
        params![
            Utc::now().to_rfc3339(),
            backend,
            project,
            i64::from(want_count),
            reason,
        ],
    );
    if let Err(e) = res {
        warn!(target: "reposix_cache::audit_failure",
              backend, project, want_count, limit,
              "log_blob_limit_exceeded failed: {e}");
    }
}
```

Add a unit test in the same file's `mod tests` block:

```rust
#[test]
fn log_blob_limit_exceeded_inserts_row() {
    let tmp = tempdir().unwrap();
    let conn = open_cache_db(tmp.path()).unwrap();
    log_blob_limit_exceeded(&conn, "sim", "demo", 250, 200);
    let (op, bytes, reason): (String, i64, String) = conn
        .query_row(
            "SELECT op, bytes, reason FROM audit_events_cache WHERE op = 'blob_limit_exceeded'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .unwrap();
    assert_eq!(op, "blob_limit_exceeded");
    assert_eq!(bytes, 250);
    assert_eq!(reason, "limit=200");
}
```

Edit `crates/reposix-cache/src/cache.rs`. After `log_helper_fetch_error` (around line 163), add a wrapper method:

```rust
/// Write an `op='blob_limit_exceeded'` audit row. Best-effort.
///
/// # Panics
/// Panics if the internal `cache.db` mutex is poisoned.
pub fn log_blob_limit_exceeded(&self, want_count: u32, limit: u32) {
    let db = self.db.lock().expect("cache.db mutex poisoned");
    crate::audit::log_blob_limit_exceeded(
        &db,
        &self.backend_name,
        &self.project,
        want_count,
        limit,
    );
}
```
</action>

<acceptance_criteria>
- `grep -n "fn log_blob_limit_exceeded" crates/reposix-cache/src/audit.rs` finds the new function.
- `grep -n "fn log_blob_limit_exceeded" crates/reposix-cache/src/cache.rs` finds the new wrapper.
- `cargo test -p reposix-cache log_blob_limit_exceeded_inserts_row` exits 0.
- `cargo build -p reposix-cache` exits 0.
</acceptance_criteria>

<threat_model>
Audit insert uses bound `params![]` (no SQL injection). The `want_count` is u32 from `RpcStats` (already bounded by `saturating_add` in `proxy_one_rpc`). No new exfil surface.
</threat_model>
