← [back to index](./index.md) · phase 83 research

## Fault-Injection Test Infrastructure

### Donor patterns

- **wiremock SoT** — `tests/push_conflict.rs::stale_base_push_emits_fetch_first_and_writes_no_rest` is the canonical donor for SoT-side faults. Uses `Mock::given(method("PATCH")).respond_with(ResponseTemplate::new(409))` to inject error status.
- **file:// mirror with passing push** — `tests/bus_precheck_b.rs::make_synced_mirror_fixture` is the canonical donor for a bare mirror that accepts pushes. **Reuse verbatim** for fault scenarios where mirror succeeds.
- **file:// mirror with failing push** — NEW. Pattern: `git init --bare <dir>` + write `<dir>/hooks/update` containing `#!/bin/sh\necho "update hook intentionally failed for fault test" >&2\nexit 1\n` + `chmod +x`. The update hook fires on every ref update and exits non-zero, causing `git push` to report `! [remote rejected] main -> main (hook declined)`.

### Test (a) — Mirror push fails between confluence-write and ack

**Setup:** wiremock SoT (full happy path: list_records returns prior, list_changed_since returns empty for PRECHECK B since cursor is fresh, PATCH for the changed record returns 200) + file:// mirror with FAILING update hook.

**Test name:** `bus_write_mirror_fail_returns_ok_with_lag_audit_row` (in `tests/bus_write_mirror_fail.rs`).

**Assertions:**
1. Helper exits zero (Q3.6 — SoT contract satisfied → ok).
2. Helper stdout contains `ok refs/heads/main`.
3. Helper stderr contains `warning: SoT push succeeded; mirror push failed`.
4. `refs/mirrors/<sot>-head` resolves to a NEW SHA (head moved).
5. `refs/mirrors/<sot>-synced-at` either absent (first push) or unchanged from baseline (frozen).
6. `audit_events_cache` count where op = `helper_push_partial_fail_mirror_lag`: 1.
7. `audit_events_cache` count where op = `mirror_sync_written`: 0 (the success-only row).
8. `audit_events_cache` count where op = `helper_push_accepted`: 1 (SoT side did succeed).
9. wiremock saw exactly 1 PATCH (assert via `Mock::expect(1)`).

### Test (b) — Confluence write fails mid-stream (5xx on second PATCH)

**Setup:** wiremock SoT with TWO records to update; first PATCH (id=1) returns 200, second PATCH (id=2) returns 500. file:// mirror with PASSING hook.

**Test name:** `bus_write_sot_mid_stream_fail_no_mirror_push_no_lag_audit` (in `tests/bus_write_sot_fail.rs`).

**Assertions:**
1. Helper exits non-zero.
2. Helper stdout contains `error refs/heads/main some-actions-failed` (existing handle_export-shape protocol error).
3. wiremock saw 2 PATCH requests (id=1 + id=2; the loop bailed at id=2's 500).
4. `audit_events_cache` count where op = `helper_push_accepted`: 0 (didn't reach the success branch).
5. `audit_events_cache` count where op = `helper_push_partial_fail_mirror_lag`: 0 (mirror never attempted).
6. `audit_events_cache` count where op = `helper_push_started`: 1 (always written).
7. **Mirror baseline ref unchanged** — file:// bare repo's main still points at the seed SHA (assert via `git -C <mirror_dir> rev-parse main`).
8. `refs/mirrors/<sot>-head` and `synced-at` UNCHANGED from baseline.
9. NOTE: per § Pitfall 3, the SoT *partially* committed — id=1 is updated server-side; id=2 is not. The test asserts this exact state by querying wiremock's request log for which PATCHes returned 200.

### Test (c) — Confluence 409 after PRECHECK B passed

**Setup:** wiremock SoT — PRECHECK B's `?since=` route returns `[]` (Stable); list_records returns prior; PATCH for id=1 returns 409 with version-mismatch body. file:// mirror with PASSING hook.

**Test name:** `bus_write_post_precheck_conflict_409_no_mirror_push` (in `tests/bus_write_post_precheck_409.rs`).

**Assertions:**
1. Helper exits non-zero.
2. Helper stdout contains `error refs/heads/main some-actions-failed` (the existing fail-on-execute path).
3. Helper stderr names the failing record id.
4. wiremock saw exactly 1 PATCH (the one that 409'd) AND exactly 1 list_changed_since (PRECHECK B Stable).
5. **Mirror NOT pushed** — `git -C <mirror_dir> rev-parse main` returns the seed SHA.
6. `audit_events_cache` count where op = `helper_push_accepted`: 0.
7. `audit_events_cache` count where op = `helper_push_partial_fail_mirror_lag`: 0.
8. `refs/mirrors/<sot>-head` and `synced-at` UNCHANGED.
9. **OPEN: do we audit the failed REST attempt?** See Open Question 1.

### Test (d) — Audit completeness happy-path

**Setup:** Standard happy-path (wiremock SoT + file:// mirror with passing hook + 2 records to update + 1 to create).

**Test name:** `bus_write_audit_completeness_happy_path_writes_both_tables` (in `tests/bus_write_audit_completeness.rs`).

**Assertions:**
1. Helper exits zero, stdout `ok refs/heads/main`.
2. Open both `audit_events_cache` (in cache.db) and `audit_events` (in sim's DB if exposed; OR via dedicated test sim that exposes its audit table).
3. Cache audit has rows for: `helper_backend_instantiated`, `helper_push_started`, `helper_push_accepted`, `mirror_sync_written`. Optionally: `helper_push_sanitized_field` × 2 (one per Update).
4. Backend audit has 3 rows: 2× `update_record` + 1× `create_record`.
5. Both tables have row counts matching the table in § "Audit Completeness Contract".

### `tests/common.rs` extension

Append two helpers:

```rust
/// Build a file:// bare mirror whose `update` hook always fails.
/// Used by mirror-fail fault tests.
pub fn make_failing_mirror_fixture() -> (tempfile::TempDir, String) {
    let mirror = tempfile::tempdir().expect("mirror tempdir");
    run_git_in(mirror.path(), &["init", "--bare", "."]);
    let hook = mirror.path().join("hooks").join("update");
    std::fs::write(&hook,
        "#!/bin/sh\necho \"intentional fail for fault test\" >&2\nexit 1\n"
    ).expect("write update hook");
    let mut perms = std::fs::metadata(&hook).unwrap().permissions();
    use std::os::unix::fs::PermissionsExt;
    perms.set_mode(0o755);
    std::fs::set_permissions(&hook, perms).expect("chmod update hook");
    let url = format!("file://{}", mirror.path().display());
    (mirror, url)
}

/// Open the cache.db at the deterministic path for (backend, project)
/// and count rows matching `op`. Used by audit-completeness assertions.
pub fn count_audit_cache_rows(cache_dir: &std::path::Path,
                              backend: &str, project: &str, op: &str) -> i64 {
    let db_path = cache_dir.join("reposix")
        .join(format!("{backend}-{project}.git"))
        .join("cache.db");
    let conn = rusqlite::Connection::open(&db_path).expect("open cache.db");
    conn.query_row(
        "SELECT COUNT(*) FROM audit_events_cache WHERE op = ?1",
        rusqlite::params![op],
        |r| r.get(0),
    ).expect("count audit rows")
}
```


## Catalog Row Design (per QG-06)

Land in `quality/catalogs/agent-ux.json` (per P82 D-04: agent-ux is the home, NOT a new bus-remote.json). All hand-edited per the existing `_provenance_note` pattern (Principle A binding for non-docs-alignment dimensions defers to GOOD-TO-HAVES-01).

**P83a rows (4 rows + 4 verifiers):**

1. `agent-ux/bus-write-sot-first-success` — happy path; SoT writes + mirror writes + both refs updated; ok returned. Verifier: `quality/gates/agent-ux/bus-write-sot-first-success.sh` runs `cargo test -p reposix-remote --test bus_write_happy -- happy_path_writes_both_refs_and_acks_ok`.
2. `agent-ux/bus-write-mirror-fail-returns-ok` — SoT succeeds + mirror fails → `ok` returned + lag audit row + warn. Verifier: runs `bus_write_mirror_fail_returns_ok_with_lag_audit_row` test.
3. `agent-ux/bus-write-no-helper-retry` — single mirror push only (no retry). Verifier: greps `crates/reposix-remote/src/bus_handler.rs` for absence of retry constructs (`for _ in 0..` adjacent to push_mirror calls); EXIT 0 if no retry present, EXIT 1 if any retry shape detected.
4. `agent-ux/bus-write-no-mirror-remote-still-fails` — regression for SC4 / Q3.5. Verifier: runs `bus_write_no_mirror_remote_emits_q35_hint` test (P82 hint preserved end-to-end after P83 lands).

**P83b rows (4 rows + 4 verifiers):**

5. `agent-ux/bus-write-fault-injection-mirror-fail` — fault (a). Verifier: runs `bus_write_mirror_fail_*` tests asserting (lag audit row, head ref moved, synced-at frozen, ok returned).
6. `agent-ux/bus-write-fault-injection-sot-mid-stream` — fault (b). Verifier: runs `bus_write_sot_mid_stream_fail_*` tests asserting (no mirror push, no lag audit, mirror baseline preserved).
7. `agent-ux/bus-write-fault-injection-post-precheck-409` — fault (c). Verifier: runs `bus_write_post_precheck_conflict_409_*` tests asserting (no mirror push, version-mismatch error cites record id).
8. `agent-ux/bus-write-audit-completeness` — both tables have expected row sets on every end-state. Verifier: runs `bus_write_audit_completeness_happy_path_writes_both_tables` test which queries both audit tables.

**Total: 8 rows across P83a + P83b.** Catalog-first invariant: each phase's first commit mints its rows status:FAIL with TINY shell verifier shells BEFORE any Rust changes.

