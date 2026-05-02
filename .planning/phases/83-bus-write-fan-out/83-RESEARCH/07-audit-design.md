← [back to index](./index.md) · phase 83 research

## Mirror-Lag Audit Row Shape

**Recommendation: NEW op `helper_push_partial_fail_mirror_lag`.** Add to `cache_schema.sql:28-48` CHECK list (sibling of existing `helper_push_accepted`).

**Schema row:**
```
op:          'helper_push_partial_fail_mirror_lag'
backend:     <backend_name>           e.g. 'sim' / 'confluence'
project:     <project>                e.g. 'demo' / 'TokenWorld'
issue_id:    NULL                     (this is a helper-RPC turn, not per-record)
oid:         <NEW_SHA hex>            the SoT SHA that head moved to
bytes:       NULL                     (no natural byte payload)
reason:      "exit=<N>;tail=<stderr_tail>"
ts:          <RFC3339>
```

**Why a new op vs. `mirror_sync_written` with status:**
- The existing `mirror_sync_written` row is written on the success path AND on the SoT-succeed-but-SHA-derivation-failed path; it conflates "ref writes attempted" semantics. Reusing it for partial-fail would muddy the success-vs-fail distinction.
- The CHECK constraint enumerates the op set; querying *"all partial-fails in last 24h"* is one `WHERE op = 'helper_push_partial_fail_mirror_lag'` clause. A status field would require `WHERE reason LIKE '%fail%'` — fragile string-matching.
- Consistent with the existing `helper_push_accepted` vs `helper_push_rejected_conflict` distinction: each push end-state has its own op.

**Helper signature** (mints in `crates/reposix-cache/src/audit.rs`, sibling of `log_helper_push_accepted` at line 230):

```rust
pub fn log_helper_push_partial_fail_mirror_lag(
    conn: &Connection,
    backend: &str,
    project: &str,
    sot_sha_hex: &str,
    exit_code: i32,
    stderr_tail: &str,
) {
    let reason = format!("exit={exit_code};tail={stderr_tail}");
    let res = conn.execute(
        "INSERT INTO audit_events_cache (ts, op, backend, project, oid, reason) \
         VALUES (?1, 'helper_push_partial_fail_mirror_lag', ?2, ?3, ?4, ?5)",
        params![Utc::now().to_rfc3339(), backend, project, sot_sha_hex, reason],
    );
    if let Err(e) = res {
        warn!(target: "reposix_cache::audit_failure",
              backend, project, exit_code,
              "log_helper_push_partial_fail_mirror_lag failed: {e}");
    }
}
```

**Wrapped accessor on Cache** (sibling of `log_mirror_sync_written` at `mirror_refs.rs:274`):

```rust
impl Cache {
    pub fn log_helper_push_partial_fail_mirror_lag(
        &self, sot_sha_hex: &str, exit_code: i32, stderr_tail: &str,
    ) {
        let conn = self.db.lock().expect("cache.db mutex poisoned");
        audit::log_helper_push_partial_fail_mirror_lag(
            &conn, &self.backend_name, &self.project,
            sot_sha_hex, exit_code, stderr_tail,
        );
    }
}
```

**Schema delta:** add `'helper_push_partial_fail_mirror_lag'` to the CHECK list in `crates/reposix-cache/fixtures/cache_schema.sql:28-48`. Update the comment on line 26-27 to cite "P83 sibling events extend this further" → "P83 ships `helper_push_partial_fail_mirror_lag`".


## Audit Completeness Contract (per OP-3)

For each end-state, both tables must have the expected rows. **This is the load-bearing contract** — incomplete audit = incomplete feature.

| End-state | `audit_events_cache` rows (cache) | `audit_events` rows (backend) |
|---|---|---|
| **Bus push, SoT ok, mirror ok** (DVCS-BUS-WRITE-01..03) | `helper_backend_instantiated` + `helper_push_started` + `helper_push_accepted` + `mirror_sync_written` + (per-record `helper_push_sanitized_field` for any Update with sanitize) | One row per executed `create_record` / `update_record` / `delete_or_close` |
| **Bus push, SoT ok, mirror fail** (DVCS-BUS-WRITE-02 partial path) | `helper_backend_instantiated` + `helper_push_started` + `helper_push_accepted` + `helper_push_partial_fail_mirror_lag` (NEW; **NO `mirror_sync_written` row**) + sanitize rows | One row per executed REST mutation (same as success — SoT writes already landed) |
| **Bus push, SoT precheck conflict** (DVCS-BUS-WRITE-01 bail path) | `helper_backend_instantiated` + `helper_push_started` + `helper_push_rejected_conflict` (existing op from P81 path) | None (no REST mutations attempted) |
| **Bus push, SoT 409 post-precheck** (fault inj c) | `helper_backend_instantiated` + `helper_push_started` (no accepted/conflict op — see § Pitfall 3 + Open Question 1) | One row for any record whose PATCH succeeded BEFORE the 409; none for the 409'd record or subsequent records |
| **Bus push, mirror remote not configured** (DVCS-BUS-WRITE-05) | `helper_backend_instantiated` only (P82 bails before stdin read) | None |

**Verification approach for the audit-completeness catalog row:**
- `bus_write_audit_completeness.rs` integration test runs each end-state once.
- After each run, the test opens the cache.db (via `rusqlite::Connection::open(<cache_path>/cache.db)`) and queries both tables for the expected op set + count.
- Asserts row counts match the table above.

