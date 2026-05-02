← [back to index](./index.md) · phase 83 plan 01

## Task 83-01-T05 — 2 integration tests + `tests/common.rs` helpers

<read_first>
- `crates/reposix-remote/tests/common.rs` (existing — `sample_issues`,
  `sim_backend`; the new helpers append).
- `crates/reposix-remote/tests/bus_precheck_b.rs` lines 60-100
  (`make_synced_mirror_fixture` — donor pattern for the file://
  bare mirror with passing update hook).
- `crates/reposix-remote/tests/bus_precheck_a.rs`
  (`bus_no_remote_configured_emits_q35_hint` — donor pattern for
  `bus_write_no_mirror_remote.rs`).
- `crates/reposix-remote/tests/mirror_refs.rs` (helper-driver
  donor — `drive_helper_export`, `render_with_overrides`).
- `crates/reposix-remote/tests/perf_l1.rs` (wiremock fixture donor
  — list_records / list_changed_since / PATCH mocks).
- `crates/reposix-cache/src/cache.rs::Cache::cache_dir` (or
  equivalent — locating cache.db path for `count_audit_cache_rows`).
</read_first>

<action>
This is the cargo-heaviest task in P83-01. Per CLAUDE.md "Build
memory budget" hold the cargo lock for `reposix-remote` integration
tests. NO parallel cargo invocations. NEVER `cargo --workspace`.

### 5a. Append helpers to `crates/reposix-remote/tests/common.rs`

```rust
// Appended to existing file. Imports (add to the existing top-of-file
// `use ...` block as needed):
//   use std::path::Path;
//   use std::process::Command;

/// Build a `file://` bare mirror whose `update` hook always fails.
/// Used by mirror-fail fault tests (P83-02 T02). Returns the
/// tempdir handle (KEEP IN SCOPE for the test's lifetime — drop
/// removes the dir) and the `file://` URL.
///
/// Gated `#[cfg(unix)]` per D-04 — the `update`-hook + `chmod 0o755`
/// pattern is POSIX-specific. Reposix CI is Linux-only at this
/// phase; macOS dev workflow honors the same hook semantics.
#[cfg(unix)]
pub fn make_failing_mirror_fixture() -> (tempfile::TempDir, String) {
    use std::os::unix::fs::PermissionsExt;
    let mirror = tempfile::tempdir().expect("mirror tempdir");
    let status = Command::new("git")
        .args(["init", "--bare", "."])
        .current_dir(mirror.path())
        .status()
        .expect("spawn git init");
    assert!(status.success(), "git init --bare failed");
    let hook = mirror.path().join("hooks").join("update");
    std::fs::write(
        &hook,
        "#!/bin/sh\necho \"intentional fail for fault test\" >&2\nexit 1\n",
    ).expect("write update hook");
    let mut perms = std::fs::metadata(&hook)
        .expect("stat hook").permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&hook, perms).expect("chmod update hook");
    let url = format!("file://{}", mirror.path().display());
    (mirror, url)
}

/// Open the cache.db at the deterministic path for (backend, project)
/// and count rows matching `op`. Used by audit-completeness assertions
/// in P83-01 and P83-02 tests.
pub fn count_audit_cache_rows(cache_db_path: &Path, op: &str) -> i64 {
    let conn = rusqlite::Connection::open(cache_db_path)
        .expect("open cache.db");
    conn.query_row(
        "SELECT COUNT(*) FROM audit_events_cache WHERE op = ?1",
        rusqlite::params![op],
        |r| r.get::<_, i64>(0),
    )
    .expect("count audit rows")
}
```

### 5b. New file — `crates/reposix-remote/tests/bus_write_happy.rs`

Setup: wiremock SoT (full happy path: `list_records` returns prior;
`list_changed_since` returns empty for PRECHECK B since cursor is
fresh; PATCH for the changed record returns 200) + file:// mirror
with PASSING update hook (default — `git init --bare` ships no
hook).

Test name: `happy_path_writes_both_refs_and_acks_ok`.

Assertions:

1. Helper exits zero.
2. Helper stdout contains `ok refs/heads/main`.
3. `git -C <wtree> for-each-ref refs/mirrors/<sot>-head` returns
   the new SoT SHA (head moved).
4. `git -C <wtree> for-each-ref refs/mirrors/<sot>-synced-at`
   returns a recent timestamp.
5. `audit_events_cache` count where op = `helper_push_started`: 1.
6. `audit_events_cache` count where op = `helper_push_accepted`: 1.
7. `audit_events_cache` count where op = `mirror_sync_written`: 1.
8. `audit_events_cache` count where op = `helper_push_partial_fail_mirror_lag`: 0.
9. wiremock saw at least 1 PATCH call.
10. `git -C <mirror> rev-parse main` returns the new SoT SHA
    (mirror push landed).

The test reuses `make_synced_mirror_fixture` shape from
`bus_precheck_b.rs` (file:// bare mirror with default — passing —
hook). The helper-driver pattern from `mirror_refs.rs` is the
scaffolding donor.

Sketch (executor fills in fixture details):

```rust
// crates/reposix-remote/tests/bus_write_happy.rs
mod common;

#[test]
fn happy_path_writes_both_refs_and_acks_ok() {
    // 1. wiremock SoT setup: mock list_records (prior), mock
    //    list_changed_since (empty — first push), mock PATCH (200).
    // 2. Build passing file:// bare mirror via tempfile + git init --bare.
    // 3. Init working tree with `git init` + add the bare mirror as
    //    a local remote.
    // 4. Drive helper via assert_cmd::Command::cargo_bin("git-remote-reposix")
    //    with bus URL `reposix::sim::demo?mirror=file://<bare_dir>`;
    //    pipe expected protocol turn (capabilities/list/export with
    //    a small fast-import payload that updates id=1).
    // 5. Assert exit zero; stdout contains "ok refs/heads/main".
    // 6. Assert refs/mirrors/<sot>-head and -synced-at populated
    //    via `git for-each-ref` against the wtree.
    // 7. Assert audit_events_cache row counts via count_audit_cache_rows.
    // 8. Assert wiremock saw >= 1 PATCH.
    // 9. Assert mirror's main ref points at new SoT SHA via
    //    `git -C <bare_mirror> rev-parse main`.
    todo!("executor implements per the assertion list above")
}
```

The executor implements the body. The test target is `bus_write_happy`
(name matches the catalog row's `cargo test --test bus_write_happy`
shape).

### 5c. New file — `crates/reposix-remote/tests/bus_write_no_mirror_remote.rs`

Setup: bus URL with a `mirror_url` that's NOT configured as a local
`git remote`. Helper short-circuits at P82's STEP 0 (no matching
remote) and emits the verbatim Q3.5 hint — even after P83's write
fan-out lands.

Test name: `bus_write_no_mirror_remote_emits_q35_hint`.

Assertions (donor `bus_precheck_a.rs::bus_no_remote_configured_emits_q35_hint`):

1. Helper exits non-zero.
2. Helper stderr contains the verbatim hint:
   *"configure the mirror remote first: `git remote add <name>
   <mirror-url>`"*.
3. NO auto-mutation of `.git/config` — assert via reading
   `<wtree>/.git/config` BEFORE and AFTER the helper invocation;
   bytes are identical.
4. NO PATCH calls hit wiremock SoT (`Mock::expect(0)` on the PATCH
   route).
5. `audit_events_cache` count where op = `helper_push_started`: 0.
   Note: count == 0 here because the no-remote-configured precheck
   (P82's STEP 0) fires BEFORE `bus_handler` reaches its own
   `helper_push_started` write. Contrast with 83-02 T03 §3a where
   the test fails MID-WRITE after the started row landed → count == 1.
6. `audit_events_cache` count where op = `helper_backend_instantiated`: 1.
   This row is written by `ensure_cache` / backend instantiation
   path EARLIER than STEP 0's no-remote check, so it lands even on
   the bail-out path. Asserting count == 1 here pins the row's
   provenance to the bus_handler entry, matching the contract row
   at PLAN-OVERVIEW.md line 647 (audit-completeness table).

Sketch:

```rust
// crates/reposix-remote/tests/bus_write_no_mirror_remote.rs
mod common;

#[test]
fn bus_write_no_mirror_remote_emits_q35_hint() {
    // 1. Init a working tree with NO remote configured.
    // 2. Seed wiremock SoT with standard list_records mocks +
    //    Mock::expect(0) on PATCH.
    // 3. Capture .git/config bytes before invocation.
    // 4. Drive helper with bus URL pointing at a mirror_url not
    //    in any local remote.<name>.url.
    // 5. Assert exit non-zero, stderr contains Q3.5 hint, .git/config
    //    bytes unchanged, wiremock saw 0 PATCH calls,
    //    helper_push_started count is 0,
    //    helper_backend_instantiated count is 1 (lands earlier than STEP 0).
    todo!("executor implements per the assertion list above")
}
```

### 5d. Cargo check + run new tests

```bash
cargo check -p reposix-remote --tests 2>&1 | tail -10
cargo nextest run -p reposix-remote --test bus_write_happy 2>&1 | tail -20
cargo nextest run -p reposix-remote --test bus_write_no_mirror_remote 2>&1 | tail -20
cargo nextest run -p reposix-remote 2>&1 | tail -20
```

### 5e. Atomic tests-and-helpers commit

```bash
git add crates/reposix-remote/tests/common.rs \
        crates/reposix-remote/tests/bus_write_happy.rs \
        crates/reposix-remote/tests/bus_write_no_mirror_remote.rs
git commit -m "test(reposix-remote): bus write fan-out happy-path + no-mirror-remote regression integration tests (DVCS-BUS-WRITE-01..05)

- crates/reposix-remote/tests/common.rs — append make_failing_mirror_fixture (cfg(unix); D-04 RATIFIED) + count_audit_cache_rows helpers; both consumed by P83-02
- crates/reposix-remote/tests/bus_write_happy.rs — happy_path_writes_both_refs_and_acks_ok asserts both refs advance on SoT-success+mirror-success; audit_events_cache has helper_push_started + helper_push_accepted + mirror_sync_written rows; helper exits zero with ok refs/heads/main; mirror's main ref points at new SoT SHA
- crates/reposix-remote/tests/bus_write_no_mirror_remote.rs — bus_write_no_mirror_remote_emits_q35_hint regression asserts P82's no-mirror-remote short-circuit holds end-to-end after P83 lands; .git/config unchanged; ZERO PATCH calls hit wiremock; ZERO helper_push_started audit rows

Existing single-backend integration tests (mirror_refs, push_conflict, bulk_delete_cap, perf_l1, stateless_connect) ALL still GREEN per D-05.

Phase 83 / Plan 01 / Task 05 / DVCS-BUS-WRITE-01 + 03 + 05."
```

NO push — T06 is terminal.
</action>

<verify>
  <automated>cargo nextest run -p reposix-remote --test bus_write_happy --test bus_write_no_mirror_remote 2>&1 | tail -10 && cargo nextest run -p reposix-remote 2>&1 | tail -5 && grep -q 'pub fn make_failing_mirror_fixture' crates/reposix-remote/tests/common.rs && grep -q 'pub fn count_audit_cache_rows' crates/reposix-remote/tests/common.rs && grep -q 'happy_path_writes_both_refs_and_acks_ok' crates/reposix-remote/tests/bus_write_happy.rs && grep -q 'bus_write_no_mirror_remote_emits_q35_hint' crates/reposix-remote/tests/bus_write_no_mirror_remote.rs</automated>
</verify>

<done>
- `crates/reposix-remote/tests/common.rs` exports
  `make_failing_mirror_fixture` (gated `#[cfg(unix)]`) +
  `count_audit_cache_rows`. Both pub fns; P83-02 consumes.
- `crates/reposix-remote/tests/bus_write_happy.rs` exists with
  `happy_path_writes_both_refs_and_acks_ok` asserting all 10
  invariants from § 5b.
- `crates/reposix-remote/tests/bus_write_no_mirror_remote.rs`
  exists with `bus_write_no_mirror_remote_emits_q35_hint`
  asserting all 5 invariants from § 5c.
- `cargo nextest run -p reposix-remote --test bus_write_happy
  --test bus_write_no_mirror_remote` passes.
- `cargo nextest run -p reposix-remote` passes ALL tests
  (regression invariant per D-05 confirmed).
- Single atomic commit; commit message names DVCS-BUS-WRITE-01..05
  + D-04 + D-05.
- NO push — T06 is terminal.
</done>

---

## Task 83-01-T06 — Catalog flip + CLAUDE.md update + per-phase push
