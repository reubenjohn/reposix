← [back to index](./index.md)

# Task 83-02-T03 — SoT-fail tests (`bus_write_sot_fail.rs` + `bus_write_post_precheck_409.rs`)

<read_first>
- `crates/reposix-remote/tests/push_conflict.rs::stale_base_push_emits_fetch_first_and_writes_no_rest`
  — donor pattern for wiremock 409 injection on PATCH (`Mock::given(method("PATCH"))
  .respond_with(ResponseTemplate::new(409))`).
- `crates/reposix-remote/tests/perf_l1.rs` — donor pattern for
  multi-record wiremock setup + the per-route ordering needed for
  the mid-stream test (PATCH id=1 returns 200, PATCH id=2 returns
  500 — wiremock's `up_to_n_times(1)` chained with a fallback
  route, OR two separate routes keyed by request body).
- `crates/reposix-remote/tests/bus_write_mirror_fail.rs` (post-T02)
  — sibling test pattern for the working-tree + bare-mirror
  scaffolding.
- `crates/reposix-remote/src/main.rs::handle_export` lines 502-512
  (the `execute_action` loop's bail-on-first-error semantic — D-09
  / Pitfall 3 contract being asserted).
- `crates/reposix-remote/src/write_loop.rs::apply_writes` (post-P83-01
  T02) — the SotPartialFail outcome path.
</read_first>

<action>
Two test files, both running in this single task. Per CLAUDE.md
"Build memory budget" the executor holds the cargo lock for
`reposix-remote` across both `cargo nextest run` invocations
(sequentially, not in parallel).

### 3a. New file — `crates/reposix-remote/tests/bus_write_sot_fail.rs`

Setup: wiremock SoT with TWO records to update — first PATCH
(id=1) returns 200, second PATCH (id=2) returns 500. file://
mirror with PASSING update hook (default — `git init --bare`).

Test name: `bus_write_sot_mid_stream_fail_no_mirror_push_no_lag_audit`.

Assertions (RESEARCH.md § "Test (b)"):

1. Helper exits non-zero.
2. Helper stdout contains `error refs/heads/main some-actions-failed`.
3. wiremock saw exactly 2 PATCH requests (id=1 + id=2; the loop
   bailed at id=2's 500).
4. `audit_events_cache` count where op = `helper_push_accepted`: 0
   (didn't reach the success branch).
5. `audit_events_cache` count where op = `helper_push_partial_fail_mirror_lag`: 0
   (mirror never attempted).
6. `audit_events_cache` count where op = `helper_push_started`: 1.
   Note: count == 1 here because the test fails MID-WRITE after
   `bus_handler` writes its `helper_push_started` row but before
   `apply_writes` reaches the success branch. Contrast with 83-01
   T05 §5c (no-mirror-remote test) where the no-remote precheck
   fires BEFORE the started row → count == 0.
7. **Mirror baseline ref unchanged** — file:// bare repo's `main`
   still points at the seed SHA (`git -C <mirror_dir> rev-parse main`).
8. `refs/mirrors/<sot>-head` and `refs/mirrors/<sot>-synced-at`
   UNCHANGED from baseline.
9. **D-09 / Pitfall 3 partial state assertion:** wiremock's
   request log shows PATCH id=1 returned 200 (id=1 IS updated
   server-side); PATCH id=2 returned 500 (id=2 unchanged); no
   subsequent PATCHes attempted (loop bailed). The test does NOT
   assert "all-or-nothing" — that's not what the helper does.

```rust
// crates/reposix-remote/tests/bus_write_sot_fail.rs
mod common;

#[test]
fn bus_write_sot_mid_stream_fail_no_mirror_push_no_lag_audit() {
    // 1. wiremock SoT: list_records returns prior with id=1 and id=2;
    //    list_changed_since returns empty (PRECHECK B Stable);
    //    PATCH id=1 returns 200; PATCH id=2 returns 500.
    //    Implementation note: route the two PATCHes by request body
    //    (the issue id appears in the URL path or body); use
    //    Mock::given(method("PATCH")).and(path_regex(".*/issues/1"))
    //    .respond_with(ResponseTemplate::new(200)) and a sibling
    //    route for /issues/2 returning 500.
    // 2. Build PASSING file:// bare mirror via tempfile + git init --bare.
    //    Capture the mirror's main ref BEFORE the test for the
    //    "baseline preserved" assertion (note: empty mirrors have
    //    no main; ref creation is the baseline).
    // 3. Init working tree + add bare mirror as local remote.
    // 4. Drive helper with bus URL + fast-import payload that
    //    updates id=1 AND id=2 (two records → two PATCHes).
    // 5. Assert exit non-zero; stdout contains
    //    "error refs/heads/main some-actions-failed".
    // 6. Assert wiremock request log: exactly 2 PATCHes (id=1 + id=2);
    //    id=1's response was 200; id=2's was 500.
    // 7. Assert audit_events_cache row counts via count_audit_cache_rows.
    // 8. Assert `git -C <mirror_dir> rev-parse main` matches baseline
    //    (mirror unchanged).
    // 9. Assert refs/mirrors/<sot>-head and -synced-at unchanged.
    todo!("executor implements per the assertion list above")
}
```

### 3b. New file — `crates/reposix-remote/tests/bus_write_post_precheck_409.rs`

Setup: wiremock SoT — `list_changed_since` returns `[]` (PRECHECK B
Stable); `list_records` returns prior; PATCH for id=1 returns 409
with version-mismatch body. file:// mirror with PASSING hook.

Test name: `bus_write_post_precheck_conflict_409_no_mirror_push`.

Assertions (RESEARCH.md § "Test (c)"):

1. Helper exits non-zero.
2. Helper stdout contains `error refs/heads/main some-actions-failed`
   (the existing `apply_writes` SotPartialFail path — `execute_action`
   propagates the 409 as an Err which the loop counts as failure).
3. Helper stderr names the failing record id (the existing
   per-action `diag(error: <e>)` line in `apply_writes` includes
   the underlying `BackendConnector` error message which carries
   the record id — confirm during T03 read_first that this is the
   case).
4. wiremock saw exactly 1 PATCH (the one that 409'd) AND exactly 1
   `list_changed_since` (PRECHECK B Stable).
5. **Mirror NOT pushed** — `git -C <mirror_dir> rev-parse main`
   returns the seed SHA OR no-such-ref.
6. `audit_events_cache` count where op = `helper_push_accepted`: 0.
7. `audit_events_cache` count where op = `helper_push_partial_fail_mirror_lag`: 0.
8. `refs/mirrors/<sot>-head` and `synced-at` UNCHANGED.
9. **D-02 RATIFIED:** test does NOT assert
   `audit_events_cache::helper_push_rest_failure` (or any
   per-failure op) — that's a v0.14.0 GOOD-TO-HAVE candidate.

```rust
// crates/reposix-remote/tests/bus_write_post_precheck_409.rs
mod common;

#[test]
fn bus_write_post_precheck_conflict_409_no_mirror_push() {
    // 1. wiremock SoT: list_changed_since returns [] (Stable);
    //    list_records returns prior with id=1; PATCH id=1 returns
    //    409 with version-mismatch JSON body (use the donor shape
    //    from push_conflict.rs::stale_base_push).
    // 2. Build PASSING file:// bare mirror.
    // 3. Init working tree + add bare mirror as local remote.
    // 4. Drive helper with bus URL + fast-import payload updating id=1.
    // 5. Assert exit non-zero; stdout contains
    //    "error refs/heads/main some-actions-failed".
    // 6. Assert stderr contains the failing record id (e.g., "issue 1"
    //    or similar — confirm exact wording from execute_action's
    //    error formatting).
    // 7. Assert wiremock saw 1 PATCH + 1 list_changed_since
    //    (Mock::expect(1) on each).
    // 8. Assert mirror's main ref unchanged via
    //    `git -C <mirror_dir> rev-parse main`.
    // 9. Assert audit_events_cache row counts: helper_push_accepted = 0,
    //    helper_push_partial_fail_mirror_lag = 0, helper_push_started = 1.
    // 10. Assert refs/mirrors/<sot>-head and -synced-at unchanged.
    todo!("executor implements per the assertion list above")
}
```

### 3c. Cargo check + run new tests (sequential)

```bash
cargo check -p reposix-remote --tests 2>&1 | tail -10
cargo nextest run -p reposix-remote --test bus_write_sot_fail 2>&1 | tail -20
cargo nextest run -p reposix-remote --test bus_write_post_precheck_409 2>&1 | tail -20
```

### 3d. Atomic commit (both files)

```bash
git add crates/reposix-remote/tests/bus_write_sot_fail.rs \
        crates/reposix-remote/tests/bus_write_post_precheck_409.rs
git commit -m "test(reposix-remote): bus_write_sot_fail.rs + bus_write_post_precheck_409.rs fault-injection (b)+(c) (DVCS-BUS-WRITE-06)

- crates/reposix-remote/tests/bus_write_sot_fail.rs — bus_write_sot_mid_stream_fail_no_mirror_push_no_lag_audit asserts:
  - helper exits non-zero with error refs/heads/main some-actions-failed
  - wiremock saw 2 PATCHes (id=1 returned 200, id=2 returned 500; loop bailed at id=2)
  - NO mirror push attempted; mirror baseline preserved
  - audit_events_cache: helper_push_accepted = 0, helper_push_partial_fail_mirror_lag = 0, helper_push_started = 1
  - refs/mirrors/<sot>-head and -synced-at unchanged
  - D-09 / Pitfall 3: SoT partial state observable on wiremock request log (id=1 server-side updated; id=2 unchanged)
- crates/reposix-remote/tests/bus_write_post_precheck_409.rs — bus_write_post_precheck_conflict_409_no_mirror_push asserts:
  - helper exits non-zero with error refs/heads/main some-actions-failed
  - stderr names failing record id
  - wiremock saw exactly 1 PATCH + 1 list_changed_since
  - NO mirror push; mirror baseline preserved
  - audit_events_cache: helper_push_accepted = 0, helper_push_partial_fail_mirror_lag = 0
  - D-02 RATIFIED: test does NOT assert per-failure REST audit row (deferred to v0.14.0 GOOD-TO-HAVE)

Phase 83 / Plan 02 / Task 03 / DVCS-BUS-WRITE-06 fault scenarios (b)+(c)."
```

NO push — T04 is terminal.
</action>

<verify>
  <automated>cargo nextest run -p reposix-remote --test bus_write_sot_fail --test bus_write_post_precheck_409 2>&1 | tail -10 && grep -q 'bus_write_sot_mid_stream_fail_no_mirror_push_no_lag_audit' crates/reposix-remote/tests/bus_write_sot_fail.rs && grep -q 'bus_write_post_precheck_conflict_409_no_mirror_push' crates/reposix-remote/tests/bus_write_post_precheck_409.rs</automated>
</verify>

<done>
- `crates/reposix-remote/tests/bus_write_sot_fail.rs` exists with
  `bus_write_sot_mid_stream_fail_no_mirror_push_no_lag_audit`
  asserting all 9 invariants from § 3a (mid-stream 5xx fault).
- `crates/reposix-remote/tests/bus_write_post_precheck_409.rs`
  exists with `bus_write_post_precheck_conflict_409_no_mirror_push`
  asserting all 10 invariants from § 3b (post-precheck 409 fault).
- `cargo nextest run -p reposix-remote --test bus_write_sot_fail
  --test bus_write_post_precheck_409` passes both tests.
- Single atomic commit (both files); commit message names
  DVCS-BUS-WRITE-06 scenarios (b)+(c) + D-09 + D-02.
- NO push — T04 is terminal.
</done>
