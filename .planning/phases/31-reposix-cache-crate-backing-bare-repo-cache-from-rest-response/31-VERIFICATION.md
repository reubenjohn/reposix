---
phase: 31
status: passed
score: 7/7
verified_at: 2026-04-24
verifier: phase-runner
---

# Phase 31 — Verification

Goal-backward verification against the six success criteria in
`.planning/ROADMAP.md` §Phase 31 plus the ARCH-02 append-only
trigger assertion.

## Summary

**PASSED — 7 / 7 criteria covered.**

`cargo build -p reposix-cache` and `cargo clippy -p reposix-cache
--all-targets -- -D warnings` are both clean. All 11 reposix-cache
tests green (9 runtime + 2 trybuild fixtures). Workspace regression
suite: 452 tests passing, 0 failures.

## Per-criterion verification

### 1. `cargo build -p reposix-cache` + clippy clean

**COVERED.**

```
$ cargo build -p reposix-cache
    Finished `dev` profile [...] in 22.15s

$ cargo clippy -p reposix-cache --all-targets -- -D warnings
    Finished `dev` profile [...] in 0.31s
```

Both exit 0. No warnings, no errors.

### 2. `Cache::build_from(backend)` produces valid bare repo with tree of N issue paths; no blobs written

**COVERED** by two integration tests in
`crates/reposix-cache/tests/`:

- `tree_contains_all_issues.rs::tree_contains_all_seeded_issues`
  (N=10) — walks `refs/heads/main`'s commit tree, asserts
  `issues/` subtree contains exactly 10 `<id>.md` entries with a
  matching `issues/1.md`.
- `tree_contains_all_issues.rs::tree_contains_single_issue` (N=1).
- `blobs_are_lazy.rs::no_blob_objects_after_build_from` — walks
  `.git/objects/`, asserts `blob_count == 0`, `commit_count == 1`,
  `tree_count >= 1`.

Implementation bypasses `Repository::edit_tree` (which validates
referenced objects exist) and builds `gix::objs::Tree` directly via
`Repository::write_object` — noted inline in `builder.rs` as the
deliberate lazy-invariant bypass.

### 3. One `op='materialize'` audit row per blob materialization

**COVERED** by
`materialize_one.rs::read_blob_materializes_exactly_one_and_audits`:

1. Seeds 5 issues → `build_from`.
2. Calls `read_blob(oid_of_issue_1)` → asserts 1 materialize row.
3. Calls `read_blob(oid_of_issue_1)` again → asserts 2 materialize
   rows (proves each call fires a row; proves idempotent content
   address leaves blob count at 1).

ARCH-02 mechanical invariant: `count(*) where op='materialize' ==
number_of_read_blob_calls`.

### 4. `Cache::read_blob` returns `Tainted<Vec<u8>>`; compile-fail asserts

**COVERED** by:

- **Type signature** — `pub async fn read_blob(&self, oid:
  gix::ObjectId) -> Result<Tainted<Vec<u8>>>` in
  `crates/reposix-cache/src/builder.rs:172`.
- **Compile-fail fixture** —
  `tests/compile-fail/tainted_blob_into_egress.rs` tries to pass
  `Tainted<Vec<u8>>` into a privileged sink expecting
  `Untainted<Vec<u8>>`. Compiler rejects with `E0308: mismatched
  types`. Diagnostic captured in sibling `.stderr`; trybuild
  verifies both the failure AND the diagnostic shape match.

### 5. Non-allowlisted origin returns error AND writes `op='egress_denied'`

**COVERED** by
`egress_denied_logs.rs::egress_denied_writes_audit_row_and_returns_egress_error`:

1. Stub `EgressRejectingBackend` whose `get_issue` always returns
   `reposix_core::Error::InvalidOrigin`.
2. `read_blob` classifies via `classify_backend_error` (typed match
   on `InvalidOrigin` + substring fallback), fires
   `log_egress_denied` BEFORE returning, returns `Error::Egress(_)`.
3. Test asserts `matches!(err, Error::Egress(_))` AND
   `SELECT COUNT(*) FROM audit_events_cache WHERE op='egress_denied' == 1`
   AND zero `materialize` rows.

### 6. SQLite audit table append-only — trigger asserted

**COVERED** by
`audit_is_append_only.rs::update_and_delete_on_audit_table_both_fail`:

1. `open_cache_db` + seed one row via `log_tree_sync`.
2. `UPDATE audit_events_cache SET ts='tampered' WHERE id=1` →
   `SQLITE_CONSTRAINT` with message containing `"append-only"`.
3. `DELETE FROM audit_events_cache WHERE id=1` → same.
4. Row count unchanged after both failures.

Triggers defined in
`crates/reposix-cache/fixtures/cache_schema.sql`:

```sql
CREATE TRIGGER audit_cache_no_update BEFORE UPDATE ON audit_events_cache
    BEGIN SELECT RAISE(ABORT, 'audit_events_cache is append-only'); END;
CREATE TRIGGER audit_cache_no_delete BEFORE DELETE ON audit_events_cache
    BEGIN SELECT RAISE(ABORT, 'audit_events_cache is append-only'); END;
```

Paired with `SQLITE_DBCONFIG_DEFENSIVE` flag set in
`db.rs::open_cache_db` to block `writable_schema` bypass (H-02
hardening).

### 7. `Untainted::new` `pub(crate)` discipline locked

**COVERED** (bonus, above ROADMAP's 6) by
`tests/compile-fail/untainted_new_is_pub_crate.rs`: calling
`reposix_core::Untainted::new` from `reposix-cache` fails with
`E0624: associated function 'new' is private`. Diagnostic captured
in sibling `.stderr`; trybuild verifies.

## Zero-regression check

```
$ cargo test --workspace 2>&1 | grep -c "test result: ok"
30
$ cargo test --workspace 2>&1 | grep FAILED | wc -l
0
```

452 workspace tests passing, 0 failures. No other crate's tests
changed behavior after Phase 31's refactor (`cache_db.rs` lift).

## Operating-principle hooks (from project CLAUDE.md)

| OP | Requirement | Status |
| --- | --- | --- |
| OP-1 Simulator-first | All tests use `wiremock` + `SimBackend` (HTTP client for the sim). No real-backend tests. | OK |
| OP-2 Tainted-by-default | `Cache::read_blob` returns `Tainted<Vec<u8>>`; trybuild compile-fail enforces. | OK |
| OP-3 Audit log non-optional | `audit_events_cache` rows written on every `build_from` (`tree_sync`) + every `read_blob` (`materialize`) + every egress denial (`egress_denied`). Append-only triggers enforce immutability. | OK |
| OP-4 No hidden state | Cache path deterministic via `resolve_cache_path(backend, project)`; overridable by `REPOSIX_CACHE_DIR`. No `/tmp` fallbacks. | OK |
| Egress allowlist | Zero `reqwest::Client` constructors in `crates/reposix-cache/src/`; all HTTP routes through `BackendConnector` → `reposix_core::http::client()`. Verified via grep. | OK |

## Artefacts

- Code: `crates/reposix-cache/{src,tests,fixtures}/`
- Plan summaries: `31-{01,02,03}-SUMMARY.md`
- Commits: 14 atomic commits on main, `ee48a46..0fa960c`.

## Phase 31 verdict

**PASSED.** Ready for Phase 32 (`stateless-connect` in
`git-remote-reposix`) to consume the `Cache::build_from` +
`Cache::read_blob` API as the tunnel target for protocol-v2 traffic.
