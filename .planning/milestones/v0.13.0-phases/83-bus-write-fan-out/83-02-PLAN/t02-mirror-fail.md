← [back to index](./index.md)

# Task 83-02-T02 — Mirror-fail integration test (`bus_write_mirror_fail.rs`)

<read_first>
- `crates/reposix-remote/tests/common.rs` (post-P83-01 T05) —
  confirm `make_failing_mirror_fixture` exists, gated `#[cfg(unix)]`,
  with the public signature `pub fn make_failing_mirror_fixture()
  -> (tempfile::TempDir, String)`.
  **M3 cross-plan rename pin:** the function name
  `make_failing_mirror_fixture` MUST match the producer in 83-01 T05
  verbatim. If 83-01 used a different name (rename during execution),
  the executor MUST use the same name here (NO rename) so the
  cross-plan contract holds. The name is also baked into 83-01
  T01's catalog row sources + 83-01 T05's commit message —
  renaming would invalidate both.
- `crates/reposix-remote/tests/common.rs` — confirm
  `count_audit_cache_rows(cache_db_path: &Path, op: &str) -> i64`
  exists.
- `crates/reposix-remote/tests/bus_write_happy.rs` (post-P83-01
  T05) — donor pattern for the helper-driver scaffolding (working
  tree + remote setup + helper invocation).
- `crates/reposix-remote/tests/perf_l1.rs` — wiremock SoT pattern
  (`Mock::given(method("GET")).and(path_regex(...))...`).
- `crates/reposix-cache/src/cache.rs` — locating the cache.db
  path for the deterministic `count_audit_cache_rows` query.
</read_first>

<action>
Create `crates/reposix-remote/tests/bus_write_mirror_fail.rs`. The
test reuses the helper-driver scaffolding from
`bus_write_happy.rs` but swaps the passing mirror fixture for
`make_failing_mirror_fixture`.

### 2a. New file — `crates/reposix-remote/tests/bus_write_mirror_fail.rs`

```rust
// crates/reposix-remote/tests/bus_write_mirror_fail.rs
//
// Fault-injection (a) — RESEARCH.md § "Test (a)": mirror push fails
// between confluence-write and ack. Helper must:
//   - exit zero with `ok refs/heads/main` (Q3.6 SoT contract).
//   - write helper_push_partial_fail_mirror_lag audit row.
//   - advance refs/mirrors/<sot>-head (head moved).
//   - LEAVE refs/mirrors/<sot>-synced-at frozen at last sync (or
//     absent on first push).
//   - leave mirror's main ref UNCHANGED (failing hook rejects).

#[cfg(unix)]
mod common;

#[cfg(unix)]
#[test]
fn bus_write_mirror_fail_returns_ok_with_lag_audit_row() {
    // 1. wiremock SoT setup: list_records returns prior; list_changed_since
    //    returns empty (PRECHECK B Stable, first push); PATCH for the
    //    changed record returns 200 (SoT-success).
    // 2. Build FAILING file:// bare mirror via
    //    common::make_failing_mirror_fixture() — `update` hook exits 1.
    // 3. Init working tree with `git init` + add the bare mirror as
    //    a local remote.
    // 4. Drive helper via assert_cmd::Command::cargo_bin("git-remote-reposix")
    //    with bus URL `reposix::sim::demo?mirror=file://<bare_dir>`;
    //    pipe expected protocol turn (capabilities/list/export with
    //    a small fast-import payload that updates id=1).
    //
    // 5. Assertions:
    //    - exit code zero.
    //    - stdout contains "ok refs/heads/main".
    //    - stderr contains "warning: SoT push succeeded; mirror push failed".
    //    - stderr contains "Reason: exit=" (the audit row's reason format).
    //    - common::count_audit_cache_rows(<cache_db>, "helper_push_partial_fail_mirror_lag") == 1.
    //    - common::count_audit_cache_rows(<cache_db>, "mirror_sync_written") == 0.
    //    - common::count_audit_cache_rows(<cache_db>, "helper_push_accepted") == 1.
    //    - `git -C <wtree> for-each-ref refs/mirrors/sim-head` returns the new SoT SHA.
    //    - `git -C <wtree> for-each-ref refs/mirrors/sim-synced-at` is either
    //      absent (first push) OR unchanged from baseline.
    //    - `git -C <bare_mirror_dir> rev-parse main` is the seed SHA
    //      (failing-update-hook rejected the push).
    //    - wiremock saw at least 1 PATCH call (`Mock::expect(1)` — SoT side
    //      DID succeed before the mirror push).
    todo!("executor implements per the assertion list above")
}
```

The test target is `bus_write_mirror_fail` (matches the catalog
row's `cargo test --test bus_write_mirror_fail` shape from T01).

### 2b. Cargo check + run new test

```bash
cargo check -p reposix-remote --tests 2>&1 | tail -10
cargo nextest run -p reposix-remote --test bus_write_mirror_fail 2>&1 | tail -20
# expect: bus_write_mirror_fail_returns_ok_with_lag_audit_row passes
```

If the test fails for reasons other than the bus_handler logic,
investigate the fixture: confirm `make_failing_mirror_fixture`
actually returns a mirror that rejects pushes (the `update` hook
must be executable AND exit non-zero). RESEARCH.md Assumption A2.

### 2c. Atomic commit

```bash
git add crates/reposix-remote/tests/bus_write_mirror_fail.rs
git commit -m "test(reposix-remote): bus_write_mirror_fail.rs fault-injection (a) (DVCS-BUS-WRITE-06 mirror-fail scenario)

- crates/reposix-remote/tests/bus_write_mirror_fail.rs — bus_write_mirror_fail_returns_ok_with_lag_audit_row asserts:
  - helper exits zero with `ok refs/heads/main` (Q3.6 SoT contract)
  - audit_events_cache has helper_push_partial_fail_mirror_lag (1) + helper_push_accepted (1) + NO mirror_sync_written (0)
  - refs/mirrors/<sot>-head advanced; synced-at FROZEN
  - mirror's main ref UNCHANGED (failing-update-hook rejected the push)
  - stderr WARN names SoT-success-mirror-fail
- gated #[cfg(unix)] per D-04 RATIFIED — failing-update-hook + chmod 0o755 is POSIX-specific

Phase 83 / Plan 02 / Task 02 / DVCS-BUS-WRITE-06 fault scenario (a)."
```

NO push — T04 is terminal.
</action>

<verify>
  <automated>cargo nextest run -p reposix-remote --test bus_write_mirror_fail 2>&1 | tail -10 && grep -q 'bus_write_mirror_fail_returns_ok_with_lag_audit_row' crates/reposix-remote/tests/bus_write_mirror_fail.rs && grep -q 'make_failing_mirror_fixture' crates/reposix-remote/tests/bus_write_mirror_fail.rs && grep -q 'helper_push_partial_fail_mirror_lag' crates/reposix-remote/tests/bus_write_mirror_fail.rs</automated>
</verify>

<done>
- `crates/reposix-remote/tests/bus_write_mirror_fail.rs` exists,
  gated `#[cfg(unix)]`, with `bus_write_mirror_fail_returns_ok_with_lag_audit_row`
  asserting all 9 invariants from § 2a.
- Test invokes `common::make_failing_mirror_fixture()` (P83-01 T05
  helper) for the failing mirror.
- Test invokes `common::count_audit_cache_rows()` for the audit
  row count assertions.
- `cargo nextest run -p reposix-remote --test bus_write_mirror_fail`
  passes (assuming Linux CI; cfg(unix) gate makes Windows skip
  cleanly).
- Single atomic commit; commit message names DVCS-BUS-WRITE-06
  scenario (a) + Q3.6 + D-04.
- NO push — T04 is terminal.
</done>
