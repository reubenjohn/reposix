---
phase: 83
title: "Bus remote — write fan-out (SoT-first, mirror-best-effort, fault injection)"
milestone: v0.13.0
requirements: [DVCS-BUS-WRITE-01, DVCS-BUS-WRITE-02, DVCS-BUS-WRITE-03, DVCS-BUS-WRITE-04, DVCS-BUS-WRITE-05, DVCS-BUS-WRITE-06]
depends_on: [80, 82]
plans:
  - 83-01-PLAN.md  # write fan-out core: catalog → apply_writes refactor prelude → cache audit op + schema delta → bus_handler write fan-out → 2 happy-path/no-mirror tests → close
  - 83-02-PLAN.md  # fault injection + audit completeness: catalog → mirror-fail → SoT-fail + post-precheck-409 → audit-completeness + close
waves:
  1: [83-01]
  2: [83-02]
---

# Phase 83 — Bus remote: write fan-out (SoT-first, mirror-best-effort, fault injection) (overview)

This is the **riskiest** DVCS-substantive phase of milestone v0.13.0
— steps 4–9 of the architecture-sketch's `§ 3` bus algorithm. P82
shipped the read/dispatch surface (URL parser, prechecks A + B,
capability branching) and ends in a clean `error refs/heads/main
bus-write-not-yet-shipped` after both prechecks pass. P83 replaces
that stub with the SoT-first-write + mirror-best-effort + audit +
ref-update logic. Per `decisions.md` Q3.6 (RATIFIED 2026-04-30):
**no helper-side retry** on transient mirror-write failure —
surface, audit, let the user retry the whole push.

**Two plans, sequential waves** per RESEARCH.md § "Plan Splitting"
recommendation + ROADMAP P83 §147 *"may want to split"* carve-out
+ CLAUDE.md "Build memory budget" sequential-cargo rule. P83-01
ships the write-fan-out core; P83-02 ships the fault-injection
suite + audit-completeness verification.

- **83-01 — write fan-out core (~6 tasks).** Catalog-first → lift
  `handle_export` write loop into shared
  `crates/reposix-remote/src/write_loop.rs::apply_writes` (P81-style
  narrow-deps refactor; preserves single-backend behavior verbatim;
  `anyhow::Result` throughout — NO new error variants) → mint cache
  audit op `helper_push_partial_fail_mirror_lag` (extend
  `cache_schema.sql:28-48` op CHECK list per P79/P80 precedent;
  helper at `audit.rs::log_helper_push_partial_fail_mirror_lag` +
  `Cache::` wrapper) → replace `bus_handler::handle_bus_export`'s
  `emit_deferred_shipped_error` stub with the full algorithm
  (`apply_writes_bus` → `push_mirror` shell-out → branch on
  outcome → ref/audit writes → `ok refs/heads/main`) → 2
  integration tests (`bus_write_happy.rs` happy path with empty +
  populated mirror; `bus_write_no_mirror_remote.rs` SC4 / Q3.5
  regression) → catalog flip + CLAUDE.md update + per-phase push.

- **83-02 — fault injection + audit completeness (~4 tasks).**
  Catalog-first → mirror-fail integration test (`bus_write_mirror_fail.rs`
  with failing-update-hook bare-repo fixture; `#[cfg(unix)]` per
  Q-D) → SoT-fail tests (`bus_write_sot_fail.rs` mid-stream 5xx +
  `bus_write_post_precheck_409.rs`) → audit-completeness test
  (`bus_write_audit_completeness.rs` queries BOTH audit tables on
  happy path) + catalog flip + CLAUDE.md addendum + per-phase
  push.

**Architecture (read BEFORE diving into tasks):**

The SoT-write half of the algorithm is `handle_export` lines 360–606
verbatim — parse stdin, run L1 precheck, plan, execute create/update/delete,
write `helper_push_accepted` cache audit row, write `last_fetched_at`
cursor, derive `sot_sha` via `cache.refresh_for_mirror_head()`, write
the two `refs/mirrors/<sot>-*` refs, write `mirror_sync_written` audit
row, ack `ok refs/heads/main`. P83-01's job is NOT to rewrite this
loop. It is to:

1. **Lift** the post-precheck portion into a shared
   `apply_writes(...)` function with a narrow-deps signature
   `(cache, backend, backend_name, project, rt, proto, parsed)`
   returning `WriteOutcome { SotOk { sot_sha, files_touched, summary } |
   Conflict | PlanRejected | SotPartialFail | PrecheckBackendUnreachable }`.
   The lift is a single atomic refactor commit — `git diff` shows
   `handle_export`'s body shrink to the wrapper shape (parse →
   `apply_writes` → match outcome → `log_token_cost` → ack); existing
   single-backend integration tests (`mirror_refs.rs`,
   `push_conflict.rs`, `bulk_delete_cap.rs`, `perf_l1.rs`,
   `stateless_connect.rs`) ALL still GREEN.
2. **Defer mirror-synced-at to caller** (Q-A RATIFIED below). The
   shared function writes `refs/mirrors/<sot>-head` always (on
   SoT-success) but does NOT write `synced-at` — single-backend
   caller writes it after `apply_writes`; bus caller defers it
   until `push_mirror` returns Ok. This preserves the load-bearing
   invariant `synced_at <= head_ts_implicit` on the partial-fail
   path (RESEARCH.md Pitfall 1).
3. **Interpose `git push <mirror_remote_name> main`** between
   SoT-success and ref/audit writes for the bus path. Plain
   `git push` — NO `--force-with-lease` (RESEARCH.md Pitfall 2;
   P84 territory). NO retry (Q3.6).

The mirror-push subprocess inherits the bus_handler's `cwd` (the
working tree where `mirror_remote_name` was resolved during P82's
STEP 0). `bus_handler::precheck_mirror_drift` (P82) already uses
the same bare `Command::new("git")` shape against the same cwd —
no `current_dir(...)` call needed.

**On the SoT-succeed-mirror-fail path:**
- `refs/mirrors/<sot>-head` IS updated to the new SoT SHA (head moved).
- `refs/mirrors/<sot>-synced-at` is NOT touched (frozen at last
  successful mirror sync — observable lag).
- New audit row `helper_push_partial_fail_mirror_lag` records the
  exit code + stderr tail (3-line tail, matches RESEARCH.md
  Pattern 2).
- `helper_push_accepted` cache-audit row IS written (SoT-success).
- `mirror_sync_written` cache-audit row is NOT written (the
  success-only row).
- stderr WARNING: *"warning: SoT push succeeded; mirror push failed
  (will retry on next push or via webhook sync). Reason: exit=<N>;
  tail=<stderr_tail>"*.
- Helper returns `ok refs/heads/main` to git anyway (Q3.6 contract:
  SoT promise satisfied → user perceives success; lag is recoverable
  on next push or webhook sync).

**On the SoT-fail path (Conflict / PlanRejected / SotPartialFail /
PrecheckBackendUnreachable):**
- Mirror push is NEVER attempted.
- `refs/mirrors/<sot>-head` and `synced-at` UNCHANGED.
- Reject lines + audit rows already emitted inside `apply_writes_bus`.
- Helper exits with `error refs/heads/main <kind>` (existing
  `handle_export` shape).

## Wave plan

Strictly sequential — P83-01 ships before P83-02 starts. Each plan
is its own wave. P83-01 ends with `git push origin main` per CLAUDE.md
push cadence; P83-02's terminal push closes the phase and triggers
verifier subagent dispatch.

| Wave | Plans  | Cargo? | File overlap with prior wave | Notes                                                                                                              |
|------|--------|--------|------------------------------|--------------------------------------------------------------------------------------------------------------------|
| 1    | 83-01  | YES (T02 + T03 + T04 + T05) | none with P82 | catalog + apply_writes refactor + cache audit op + schema delta + bus_handler write fan-out + 2 happy/no-mirror tests + close |
| 2    | 83-02  | YES (T02 + T03 + T04)       | shared `tests/common.rs` (additive append; non-conflicting since P83-01 added `make_failing_mirror_fixture` and `count_audit_cache_rows`) | 3 fault-injection tests + audit-completeness test + close |

**Files modified (cross-plan audit):**

| Plan  | Files                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                          |
|-------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 83-01 | `crates/reposix-remote/src/write_loop.rs` (new), `crates/reposix-remote/src/main.rs` (mod decl + `handle_export` body shrink), `crates/reposix-remote/src/bus_handler.rs` (replace `emit_deferred_shipped_error` stub with `apply_writes_bus` + `push_mirror` + branch), `crates/reposix-cache/src/audit.rs` (append `log_helper_push_partial_fail_mirror_lag`), `crates/reposix-cache/src/cache.rs` (append `Cache::log_helper_push_partial_fail_mirror_lag` wrapper), `crates/reposix-cache/fixtures/cache_schema.sql` (extend op CHECK list), `crates/reposix-cache/src/audit.rs` `mod tests` (1 unit test for the new helper), `crates/reposix-remote/tests/common.rs` (append `make_failing_mirror_fixture` + `count_audit_cache_rows` — usable by P83-02), `crates/reposix-remote/tests/bus_write_happy.rs` (new), `crates/reposix-remote/tests/bus_write_no_mirror_remote.rs` (new), `quality/catalogs/agent-ux.json` (4 new rows), `quality/gates/agent-ux/bus-write-sot-first-success.sh` (new), `quality/gates/agent-ux/bus-write-mirror-fail-returns-ok.sh` (new — exercised by P83-02's mirror-fail test, but the row mints in P83-01's catalog-first commit per RESEARCH.md § "Catalog Row Design" P83a/P83b split), `quality/gates/agent-ux/bus-write-no-helper-retry.sh` (new — grep-based source-pattern check), `quality/gates/agent-ux/bus-write-no-mirror-remote-still-fails.sh` (new), `CLAUDE.md` |
| 83-02 | `crates/reposix-remote/tests/bus_write_mirror_fail.rs` (new; uses `make_failing_mirror_fixture` from P83-01's common.rs), `crates/reposix-remote/tests/bus_write_sot_fail.rs` (new), `crates/reposix-remote/tests/bus_write_post_precheck_409.rs` (new), `crates/reposix-remote/tests/bus_write_audit_completeness.rs` (new), `quality/catalogs/agent-ux.json` (4 new rows), `quality/gates/agent-ux/bus-write-fault-injection-mirror-fail.sh` (new), `quality/gates/agent-ux/bus-write-fault-injection-sot-mid-stream.sh` (new), `quality/gates/agent-ux/bus-write-fault-injection-post-precheck-409.sh` (new), `quality/gates/agent-ux/bus-write-audit-completeness.sh` (new), `CLAUDE.md` (addendum to the P83-01 paragraph: name the four shipped fault-injection tests + the dual-table audit-completeness contract) |

**No file overlap between 83-01 and 83-02 except `quality/catalogs/agent-ux.json`
+ `CLAUDE.md` + `crates/reposix-remote/tests/common.rs` — all three are
strictly additive (append-only diffs); 83-02 reads but does not modify
83-01's writes to those files.** Per CLAUDE.md "Build memory budget"
the two plans run in separate cargo windows: 83-01 holds the
`reposix-cache` + `reposix-remote` cargo lock sequentially across
T02–T05; 83-02 holds the `reposix-remote` cargo lock sequentially
across T02–T04. No parallel cargo invocations within OR across the
two plans.

## Plan summary table

| Plan  | Goal                                                                                                                | Tasks | Cargo? | Catalog rows minted | Tests added                                                                                                                          | Files modified (count) |
|-------|---------------------------------------------------------------------------------------------------------------------|-------|--------|---------------------|--------------------------------------------------------------------------------------------------------------------------------------|------------------------|
| 83-01 | apply_writes refactor + cache audit op + bus write fan-out + 2 happy-path/no-mirror tests                           | 6     | YES (T02+T03+T04+T05) | 4 (FAIL → PASS at T06) | 1 unit (audit helper roundtrip) + 2 integration (`bus_write_happy.rs`, `bus_write_no_mirror_remote.rs`) = 3 total                     | ~16 (write_loop.rs new + bus_handler.rs edit + main.rs edit + audit.rs extend + cache.rs wrapper + cache_schema.sql delta + 2 new test files + 4 verifier shells + catalog edit + common.rs append + CLAUDE.md) |
| 83-02 | 3 fault-injection tests + audit-completeness test                                                                   | 4     | YES (T02+T03+T04)     | 4 (FAIL → PASS at T04) | 4 integration (`bus_write_mirror_fail.rs`, `bus_write_sot_fail.rs`, `bus_write_post_precheck_409.rs`, `bus_write_audit_completeness.rs`) | ~10 (4 new test files + 4 verifier shells + catalog edit + CLAUDE.md addendum) |

Total: 10 tasks across 2 plans. Wave plan: sequential.

Test count: 1 unit (audit helper INSERT round-trip) + 6 integration
tests (2 in P83-01, 4 in P83-02) = 7 total. Plus the existing
single-backend `handle_export` integration tests (`mirror_refs.rs`,
`push_conflict.rs`, `bulk_delete_cap.rs`, `perf_l1.rs`,
`stateless_connect.rs`) all GREEN as the regression check on the
P83-01 T02 refactor (Q-E RATIFIED — no separate catalog row for
the regression invariant).

## Decisions ratified at plan time

The six open questions surfaced by RESEARCH.md § "Open Questions
for the Planner" are RATIFIED here so the executing subagent and
the verifier subagent both grade against the same contract. Each
decision references the source artifact and the rationale. The user
directive (orchestrator instructions for P83) pre-resolved each
question; this section captures them as Plan-time decisions.

### D-01 — Q-A: `apply_writes` defers mirror-synced-at to caller (RATIFIED)

**Decision:** the shared `apply_writes` function writes
`refs/mirrors/<sot>-head` always (on SoT-success) but does NOT write
`refs/mirrors/<sot>-synced-at`. The single-backend caller
(`handle_export`) writes synced-at after `apply_writes` returns
`SotOk`. The bus caller (`bus_handler::handle_bus_export`) defers
synced-at until `push_mirror` returns Ok. This shape produces a
single `apply_writes` entry point — NO `apply_writes_bus` second
function, NO `update_synced_at: bool` flag.

**Why this shape (and not: dual entry points or a flag):** Open
Question 3 in RESEARCH.md flagged the boolean flag as cognitively
heavy. A single function with the synced-at write deferred to the
caller produces symmetric call sites — both `handle_export` and
`bus_handler` end with their own ref/audit write block. The bus
path's block branches on `MirrorResult`; the single-backend path's
block is unconditional (because for single-backend, "SoT success"
already means "mirror current" — single-backend has no separate
mirror leg). The lifted body in `write_loop.rs` is concerned only
with SoT-side correctness; mirror-side decisions live in the
respective caller.

**Implementation note (T02):** `handle_export`'s post-`apply_writes`
block writes `head` (already done by `apply_writes`), then
`synced-at`, then `mirror_sync_written` audit row, then
`log_token_cost`, then `ok refs/heads/main`. The bus path's
post-`apply_writes` block writes `head` (already done by
`apply_writes`), runs `push_mirror`, branches on outcome, writes
synced-at + `mirror_sync_written` ON success OR
`helper_push_partial_fail_mirror_lag` on failure, then `ok
refs/heads/main` either way.

**Source:** RESEARCH.md § "Open Questions for the Planner" Q-A;
Open Question 3; user directive *"`apply_writes` writes mirror refs
itself, or defers to caller? DEFER to caller"*.

### D-02 — Q-B: do NOT audit failed REST attempts in P83 (RATIFIED)

**Decision:** `apply_writes_bus`'s `execute_action` loop continues
the existing `handle_export` semantics — log only successes (the
backend adapter's per-record `audit_events` row writes inside the
adapter's success path). Failed REST attempts (e.g. 409 on PATCH
id=2) do NOT get a `helper_push_rest_failure` cache-audit row in
P83. Filed as v0.13.0 GOOD-TO-HAVES candidate for P88.

**Why defer (and not: ship the per-failure row in P83):** P83 is
already the riskiest phase. Adding a new audit op
(`helper_push_rest_failure`) plus integration coverage plus the
schema delta would expand scope unnecessarily. The existing audit
shape (per-success row in `audit_events` + the helper-RPC-turn row
in `audit_events_cache`) IS what `handle_export` ships today — bus
inherits the same trade-off. P85's troubleshooting docs may surface
user demand; if so, file a v0.14.0 row.

**Implementation note (T04 / P83-02 audit-completeness test):**
the test (c) post-precheck-409 scenario asserts `audit_events_cache`
has `helper_push_started` (always) but NOT `helper_push_accepted`
or `helper_push_partial_fail_mirror_lag` — and asserts the test
explicitly does NOT look for a per-failure row (i.e., no assertion
of the form `count(op = 'helper_push_rest_failure') == 1`).

**Source:** RESEARCH.md § "Open Question 1"; user directive *"Audit
failed REST attempts? NO — log only successes; v0.13.0 GOOD-TO-HAVE
filed for v0.14.0"*.

### D-03 — Q-C: schema delta is ATOMIC with the audit-helper-fn commit (RATIFIED)

**Decision:** P83-01 T03 lands the schema-delta + helper function +
`Cache::` wrapper in a SINGLE atomic commit. `cache_schema.sql:28-48`
gains `'helper_push_partial_fail_mirror_lag'` in the op CHECK list,
the comment narrative on lines 22-27 is extended to name P83's new
op, `audit.rs` gains `log_helper_push_partial_fail_mirror_lag`,
`cache.rs` gains the `Cache::log_helper_push_partial_fail_mirror_lag`
wrapper, and the `audit.rs::mod tests` block gains one INSERT
roundtrip test. Single commit. Matches P79 (`attach_walk` added in
one commit) + P80 (`mirror_sync_written` added in one commit)
precedent.

**Why atomic (and not: schema delta as separate prelude commit):**
the schema delta is meaningless without the helper that writes the
row, AND vice versa. Bisecting through a "schema delta only" commit
would land in a state where fresh caches accept rows that no helper
writes (semantically benign but confusing). Atomic commit is the
established pattern from P79/P80.

**Stale cache.db semantics:** existing cache.db files keep the
legacy CHECK list (per the existing comment at `cache_schema.sql:11-21`
*"On stale cache.db files the new ops will fail the CHECK and fall
through the audit best-effort path (warn-logged)"*). The audit
helper is best-effort (returns `()`, WARN-logs on INSERT failure),
so stale caches WARN-log + the push still succeeds. Fresh caches
accept the row immediately. No migration script. RESEARCH.md
Pitfall 7.

**Source:** RESEARCH.md § "Open Question for the Planner" Q-C;
user directive *"Schema delta atomic with helper, or separate
commit? ATOMIC with the audit-helper-fn commit"*; P79/P80 atomic-
schema-delta precedent.

### D-04 — Q-D: failing-update-hook fixture gated `#[cfg(unix)]` (RATIFIED)

**Decision:** the `make_failing_mirror_fixture` helper added to
`crates/reposix-remote/tests/common.rs` (P83-01 T05) writes a
POSIX shell script to `<bare>/hooks/update` and chmods it 0o755 via
`std::os::unix::fs::PermissionsExt`. The helper itself is gated
`#[cfg(unix)]` and the consuming test files
(`bus_write_mirror_fail.rs` in P83-02 T02) gate their `#[test]`
functions `#[cfg(unix)]` likewise. Windows CI (not currently
supported per CLAUDE.md "Tech stack" — Linux only) skips the test
cleanly without a build error.

**Why gate (and not: write a portable fixture):** the `update` hook
mechanism is POSIX-shell-script-based; emulating it on Windows
requires either WSL or a `pwsh` rewrite. Reposix's CI is Linux-only
at this phase (no Windows runners in `.github/workflows/`). Gating
preserves cross-platform compilation while skipping execution
where the fixture cannot run.

**Implementation note (P83-01 T05):**

```rust
#[cfg(unix)]
pub fn make_failing_mirror_fixture() -> (tempfile::TempDir, String) {
    use std::os::unix::fs::PermissionsExt;
    // ... `git init --bare`, write hooks/update with `exit 1`, chmod 0o755 ...
}
```

`#[cfg(unix)]` on the public helper means consuming tests must
gate likewise; trying to call it from a non-unix `#[cfg]` block
fails at compile time, surfacing the gating intent immediately.

**Source:** RESEARCH.md § "Open Question for the Planner" Q-D +
Assumption A2; user directive *"Failing-update-hook fixture
portable to Windows? `#[cfg(unix)]` gate if needed; reposix CI is
Linux-only at this phase"*.

### D-05 — Q-E: refactor task does NOT carry a regression catalog row (RATIFIED)

**Decision:** P83-01 T02's refactor (lift `handle_export` write
loop into `write_loop::apply_writes`) does NOT mint a catalog row
asserting "existing single-backend tests still GREEN." The
existing per-test `[[test]]` targets in
`crates/reposix-remote/Cargo.toml` (`mirror_refs`, `push_conflict`,
`bulk_delete_cap`, `perf_l1`, `stateless_connect`,
`stateless_connect_e2e`) are the regression check. Pre-push gate
runs the workspace-wide test suite; if the refactor regresses
single-backend behavior, the gate fires before the per-phase push
lands.

**Why no row (and not: mint a regression-asserting row):** the
refactor's invariant is "`handle_export` external behavior
unchanged" — best expressed as "all existing tests pass," not as a
new catalog row. A regression-asserting row would duplicate what
the test runner already proves; minting it adds noise to the
catalog without adding signal. The blast-radius of a refactor
regression is "single-backend push breaks" which is already
covered by the existing tests + their own catalog rows
(`agent-ux/sync-reconcile-subcommand`,
`agent-ux/mirror-refs-write-on-success`, etc., all of which
exercise the lifted code path).

**Implementation note (T02):** at the end of T02, run `cargo
nextest run -p reposix-remote --tests` (per-crate, sequential, per
CLAUDE.md "Build memory budget"). All existing integration tests
must pass. The verifier subagent confirms via the verdict file.

**Source:** RESEARCH.md § "Open Question for the Planner" Q-E;
user directive *"Refactor task carries its own regression catalog
row? NO — existing tests + pre-push runner are sufficient"*.

### D-06 — Q-F: NEW op `helper_push_partial_fail_mirror_lag` (RATIFIED)

**Decision:** P83-01 T03 mints a NEW audit op
`helper_push_partial_fail_mirror_lag` in `audit_events_cache` for
the SoT-succeed-mirror-fail end-state. Does NOT reuse
`mirror_sync_written` with a status-field overload.

**Why new op (and not: reuse `mirror_sync_written`):**

1. **CHECK constraint clarity.** The op CHECK list at
   `cache_schema.sql:28-48` enumerates the legitimate ops; querying
   *"all partial-fails in last 24h"* becomes one `WHERE op =
   'helper_push_partial_fail_mirror_lag'` clause. Reusing
   `mirror_sync_written` with a `status` field embedded in `reason`
   would require fragile substring-grepping (`WHERE reason LIKE
   '%fail%'`).
2. **Symmetric with `helper_push_accepted` vs
   `helper_push_rejected_conflict`.** Each push end-state has its
   own op. Adding `helper_push_partial_fail_mirror_lag` extends
   the existing pattern: each distinct end-state → distinct op.
3. **Forensic queries on the dual-table audit shape (OP-3) become
   trivially expressible.** A reader asking *"which bus pushes
   landed SoT writes but failed mirror?"* writes one SQL query
   against `audit_events_cache` with `op =
   'helper_push_partial_fail_mirror_lag'`, joined to
   `audit_events` by `(backend, project, ts)` to fetch the
   per-record mutations.

**Schema row layout** (RESEARCH.md § "Mirror-Lag Audit Row Shape"):

```
op:          'helper_push_partial_fail_mirror_lag'
backend:     <backend_name>           e.g. 'sim' / 'confluence'
project:     <project>                e.g. 'demo' / 'TokenWorld'
issue_id:    NULL                     (helper-RPC turn, not per-record)
oid:         <NEW_SHA hex>            the SoT SHA head moved to
bytes:       NULL
reason:      "exit=<N>;tail=<stderr_tail>"
ts:          <RFC3339>
```

**Helper signature** (mints in `crates/reposix-cache/src/audit.rs`,
sibling of `log_helper_push_accepted` at line 230):

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

**`Cache::` wrapper** (sibling of `log_mirror_sync_written` at
`mirror_refs.rs:274`):

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

**Source:** RESEARCH.md § "Mirror-Lag Audit Row Shape" + § "Open
Question for the Planner" Q-F; user directive *"New op vs reuse
`mirror_sync_written`? NEW op `helper_push_partial_fail_mirror_lag`
(clear semantics; clean queries)"*; P79 (`attach_walk`) + P80
(`mirror_sync_written`) precedent for distinct-op-per-end-state.

### D-07 — `agent-ux.json` is the catalog home (NOT a new `bus-write.json`)

**Decision:** P83-01 T01 + P83-02 T01 add 8 new rows to the
existing `quality/catalogs/agent-ux.json` (joining the 12 P82 rows
+ the prior P79/P80/P81 rows). NOT a new `bus-write.json`.

**Why:** dimension catalogs route to `quality/gates/<dim>/` runner
discovery — `agent-ux` is the existing dimension. Splitting it into
two catalog files would force the runner to discover both via tag,
adding indirection for no benefit. P82's D-04 set this precedent;
P83 inherits.

**Source:** RESEARCH.md § "Catalog Row Design" (recommends
`agent-ux.json`); P82 D-04.

### D-08 — Plain `git push` for the mirror push (NO `--force-with-lease`)

**Decision:** `bus_handler::push_mirror` runs
`git push <mirror_remote_name> main` — plain. NO `--force-with-lease`.
NO `--force`. NO any flag-based override. The verifier shell
`quality/gates/agent-ux/bus-write-no-helper-retry.sh` greps the
helper source for the absence of these tokens and fails RED if any
are present.

**Why plain push (and not: cargo-cult `--force-with-lease` from
P84):** P84's webhook workflow uses `--force-with-lease` because
it races with bus pushes. Bus push doesn't race with itself within
a single `git push` invocation: by the time we reach `push_mirror`,
PRECHECK A already trapped any concurrent mirror drift, AND our
SoT write IS the new authoritative state. If a concurrent
webhook-sync raced in between PRECHECK A and our `push_mirror`,
our push fails with non-fast-forward → that's the partial-fail
path → audit + ok + recover on next push. NO force.

**Implementation note (T04):**

```rust
let out = Command::new("git")
    .args(["push", mirror_remote_name, "main"])
    .output()
    .with_context(|| format!("spawn `git push {mirror_remote_name} main`"))?;
```

NO third arg in the `args(...)` list beyond `["push", name, "main"]`.

**Source:** RESEARCH.md § "Anti-Patterns to Avoid" + Pitfall 2;
user directive *"NO `--force-with-lease`. Plain `git push <mirror>
main`. P84 owns force-with-lease"*.

### D-09 — Confluence non-atomicity across actions documented as inherited semantic

**Decision:** the existing `handle_export::execute_action` per-action
loop (lines 502-512) is best-effort-stop-on-first-error. Bus path
inherits this verbatim — no transaction boundary spanning multiple
PATCH/PUT calls; on partial-fail, SoT state is `id_1: new_v, id_2:
old_v` (only id_1 succeeded before the loop bailed). RESEARCH.md
Pitfall 3 documents this. P83 plan body reproduces the documentation
in `bus_handler.rs`'s module-doc.

**Why document (and not: add atomicity):** atomic two-phase commit
across SoT actions is OUT OF SCOPE per `REQUIREMENTS.md § "Out of
Scope"` (*"Bus remote is 'SoT-first, mirror-best-effort with lag
tracking,' not 2PC. Document the asymmetry; don't try to hide
it."*). The recovery story is the user's next push reads new SoT
state via PRECHECK B's `list_changed_since` and either accepts
the change (if version still matches) or rejects with conflict.

**Test contract (P83-02 T03 fault-injection (b)):**
`bus_write_sot_fail.rs` asserts the EXACT partial state — id=1 has
new version (PATCH 200), id=2 unchanged (PATCH 500), no audit row
for ids subsequent to id=2, mirror unchanged. RESEARCH.md § "Test
(b)" is the verbatim contract.

**Source:** RESEARCH.md § "Pitfall 3"; `REQUIREMENTS.md § "Out of
Scope"`; existing `handle_export` per-action loop semantic.

### D-10 — `last_fetched_at` cursor advance is L1-trade-off documented inline

**Decision:** `apply_writes` advances `last_fetched_at` to `now`
on SoT-success — BEFORE `push_mirror`. If mirror push fails, the
cursor has advanced past the SoT state, but PRECHECK B's
`list_changed_since(cursor)` will return EMPTY on the next push
attempt (no new SoT changes between mirror-fail and retry). The
next push proceeds normally.

**Why this trade-off (and not: defer cursor advance until mirror
success):** the L1 trade-off documented in
`architecture-sketch.md § "Performance subtlety"` is the canonical
basis for this — `reposix sync --reconcile` (DVCS-PERF-L1-02
shipped P81) is the on-demand escape hatch for cache desync. Bus
inherits the L1 trust model.

**Race window:** if a confluence-side edit lands between
SoT-write-success and `push_mirror`, the cursor advance to `now`
would mask that edit on the next push. P83 documents this as a
known L1 trade-off, NOT a P83 bug. RESEARCH.md § "Open Question 2".

**Source:** RESEARCH.md § "Open Question 2"; `architecture-sketch.md
§ "Performance subtlety"`; `decisions.md § Q3.1` RATIFIED L1.

## Subtle architectural points (read before T02 of either plan)

The two below are flagged because they are the most likely sources
of executor friction. The executor must internalize them before
writing the wiring code.

### S1 — `apply_writes` body lifts verbatim — preserve, don't rewrite

The lift in P83-01 T02 is **mechanical**, not creative. The body
of `apply_writes_impl` is `handle_export` lines 360-606 with three
specific replacements:

1. `state.cache.as_ref()` → `cache` (bound parameter).
2. `state.backend.as_ref()` → `backend` (bound parameter).
3. `state.backend_name`, `state.project`, `state.rt` → `backend_name`,
   `project`, `rt` (bound parameters).
4. `state.push_failed = true; return Ok(());` → `return
   Ok(WriteOutcome::<variant>);` (the function returns a
   `WriteOutcome` enum that the caller maps to the `push_failed`
   flag).
5. The synced-at write (`cache.write_mirror_synced_at(...)`) is
   REMOVED from the lifted body — D-01 defers it to the caller.
6. The `mirror_sync_written` audit row write
   (`cache.log_mirror_sync_written(...)`) is REMOVED from the
   lifted body — D-01 defers it to the caller (because in the bus
   path it lives behind the mirror-success branch).
7. The `log_token_cost` call at lines 593-599 is REMOVED from the
   lifted body — single-backend caller writes it (with
   `chars_in + ack-bytes`); bus caller writes it (with `chars_in
   + ack-bytes + mirror-push-stderr-tail-bytes`). RESEARCH.md
   § "Open Question 4".

The `head` ref write (`cache.write_mirror_head(...)`) STAYS in
the lifted body because it's unconditional on SoT-success (D-01).

**Why this matters for T02.** A reviewer or executor might be
tempted to "improve" the lifted code (deduplicate logic, add
helper functions, simplify error handling). DO NOT. The lift is
mechanical preservation; the only change is the parameter shape
and what gets returned vs written-and-mutated. Single-backend
behavior must be byte-for-byte equivalent post-refactor — that's
the regression contract D-05 names.

### S2 — `bus_handler::handle_bus_export` post-PRECHECK shape

Replace the body of `handle_bus_export` from line 172 onward (the
current `emit_deferred_shipped_error` stub) with the full
algorithm. The order matters:

```rust
// PRECHECK B passed (line 170 in current code). Now: read stdin,
// write SoT, push mirror.

let mut buffered = BufReader::new(ProtoReader::new(proto));
let parse_result = parse_export_stream(&mut buffered);
drop(buffered);
let parsed = match parse_result {
    Ok(v) => v,
    Err(e) => return bus_fail_push(proto, state, "parse-error",
        &format!("parse export stream: {e:#}")),
};

// log_helper_push_started — same OP-3 row as handle_export emits.
if let Some(cache) = state.cache.as_ref() {
    cache.log_helper_push_started("refs/heads/main");
}

// SoT write half — shared with handle_export, factored into apply_writes.
let write_outcome = write_loop::apply_writes(
    state.cache.as_ref(),
    state.backend.as_ref(),
    &state.backend_name,
    &state.project,
    &state.rt,
    proto,
    parsed,
)?;

let (sot_sha, _files_touched, _summary) = match write_outcome {
    write_loop::WriteOutcome::SotOk { sot_sha, files_touched, summary } =>
        (sot_sha, files_touched, summary),
    // All non-Ok outcomes already emitted reject lines + audit rows
    // inside apply_writes. Set push_failed and return cleanly.
    _ => {
        state.push_failed = true;
        return Ok(());
    }
};

// SoT side succeeded. Mirror push (no retry per Q3.6).
let mirror_result = push_mirror(&mirror_remote_name)?;

match mirror_result {
    MirrorResult::Ok => {
        // Both refs current; lag = 0.
        if let Some(cache) = state.cache.as_ref() {
            if let Err(e) = cache.write_mirror_synced_at(
                &state.backend_name, chrono::Utc::now()) {
                tracing::warn!("write_mirror_synced_at failed: {e:#}");
            }
            let oid_hex = sot_sha.map(|o| o.to_hex().to_string())
                .unwrap_or_default();
            cache.log_mirror_sync_written(&oid_hex, &state.backend_name);
        }
        proto.send_line("ok refs/heads/main")?;
        proto.send_blank()?;
        proto.flush()?;
    }
    MirrorResult::Failed { exit_code, stderr_tail } => {
        // SoT contract satisfied; mirror lags. NO RETRY (Q3.6).
        // synced-at INTENTIONALLY NOT WRITTEN — frozen at last successful sync.
        if let Some(cache) = state.cache.as_ref() {
            let oid_hex = sot_sha.map(|o| o.to_hex().to_string())
                .unwrap_or_default();
            cache.log_helper_push_partial_fail_mirror_lag(
                &oid_hex, exit_code, &stderr_tail);
        }
        crate::diag(&format!(
            "warning: SoT push succeeded; mirror push failed \
             (will retry on next push or via webhook sync). \
             Reason: exit={exit_code}; tail={stderr_tail}"
        ));
        proto.send_line("ok refs/heads/main")?;
        proto.send_blank()?;
        proto.flush()?;
    }
}
Ok(())
```

The `head` ref write happens INSIDE `apply_writes` (D-01). The
`synced-at` ref write happens HERE (D-01). The
`mirror_sync_written` audit row also moves HERE (under the
`MirrorResult::Ok` arm); the `helper_push_partial_fail_mirror_lag`
audit row is the new addition for the failure arm.

**Why this matters for T04.** A reviewer might wonder why
`apply_writes` doesn't just write everything. D-01's deferral is
intentional — the bus path's mirror-failure leg needs synced-at
NOT written, and a single function with a flag is more confusing
than a clear caller-side block.

## Threat model crosswalk

Per CLAUDE.md § "Threat model" — this phase introduces TWO new
trifecta surfaces (the `git push` shell-out's argument boundary +
the per-record SoT write's expanded fault surface) and reuses three
existing surfaces unchanged:

| Existing surface              | What P83 changes                                                                                                                                                                                                                                                                                                                                                          |
|-------------------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| Helper outbound HTTP          | UNCHANGED — `apply_writes`'s SoT REST writes are the same `BackendConnector` trait + `client()` factory + `REPOSIX_ALLOWED_ORIGINS` allowlist used since v0.9.0. NO new HTTP construction site.                                                                                                                                                                            |
| Cache prior-blob parse (`Tainted` bytes) | UNCHANGED — `apply_writes` runs `precheck_export_against_changed_set` (P81) which parses prior blobs. The parser path is preserved verbatim by the lift (S1).                                                                                                                                                                                                              |
| `Tainted<T>` propagation      | UNCHANGED — `parse_export_stream` produces `Tainted<Record>`; `execute_action`'s `sanitize(Tainted::new(issue), meta)` boundary is the same. NO new tainted-bytes seam.                                                                                                                                                                                                    |
| **`git push` shell-out (NEW)** | NEW: `Command::new("git").args(["push", mirror_remote_name, "main"]).output()`. The `mirror_remote_name` is helper-resolved from `git config` (NOT user-controlled at this point — P82's STEP 0 already validated). STRIDE category: Tampering — mitigated by `mirror_remote_name.starts_with('-')` defensive reject + bounded-by-`git`-remote-name-validation provenance. |
| **Mirror push subprocess stderr_tail (NEW operator-readable seam)** | NEW: 3-line stderr tail captured for the audit row + WARNING log. The stderr is git-controlled (not record-content controlled), but operator-readable; could leak repo-internal info (commit SHAs, ref names). Trimming to 3 lines bounds the leak surface. STRIDE category: Information Disclosure — mitigated by trim. |
| **Per-record SoT-fail seam** (no new shell-out, but expanded fault surface in tests) | UNCHANGED in code path, EXPANDED in test coverage. Tests inject 5xx + 409 via wiremock — the helper code is unchanged; the fault-injection surface validates that the helper's existing failure handling is correct, not that new failure handling exists. |

`<threat_model>` STRIDE register addendum (carried into the plan
bodies):

- **T-83-01 (Tampering — argument injection via `mirror_remote_name`
  in `git push` shell-out):** reject `-`-prefix on
  `mirror_remote_name` BEFORE shell-out, mirroring P82's T-82-01.
  `mirror_remote_name` is helper-resolved (not user-controlled), so
  the defense is defensive-in-depth.
- **T-83-02 (Information Disclosure — stderr_tail leakage in audit
  row):** trim to 3 lines (RESEARCH.md Pattern 2). 3-line bound
  documented in `audit.rs::log_helper_push_partial_fail_mirror_lag`
  doc comment.
- **T-83-03 (Repudiation — partial-fail with mirror lag undetected):**
  the `helper_push_partial_fail_mirror_lag` audit row records the
  SoT SHA + exit code + stderr tail. Plus the head≠synced-at
  invariant on the refs side gives a vanilla-`git`-only operator a
  way to detect lag without database access.
- **T-83-04 (Denial of Service — `git push` against private mirrors
  hangs on SSH-agent prompt):** documented in CLAUDE.md update.
  Tests use `file://` fixture exclusively. Same disposition as
  T-82-03 (accept).
- **T-83-05 (Tampering — Confluence non-atomicity across actions):**
  ACCEPT. RESEARCH.md Pitfall 3 + D-09. The recovery story is
  next-push reads new SoT via PRECHECK B; documented inline in
  `bus_handler.rs` module-doc + CLAUDE.md update.

## Phase-close protocol

Per CLAUDE.md OP-7 + REQUIREMENTS.md § "Recurring success criteria
across every v0.13.0 phase":

1. **All commits pushed.** P83-01 ends with `git push origin main`
   in its T06; P83-02 ends with `git push origin main` in its T04
   (terminal). Pre-push gate-passing is part of each plan's close
   criterion.
2. **Pre-push gate GREEN.** If pre-push BLOCKS: treat as
   plan-internal failure (fix, NEW commit, re-push). NO `--no-verify`
   per CLAUDE.md git safety protocol.
3. **Verifier subagent dispatched.** AFTER 83-02 T04 pushes
   (i.e., after the phase's terminal task completes), the
   orchestrator dispatches an unbiased verifier subagent per
   `quality/PROTOCOL.md § "Verifier subagent prompt template"`
   (verbatim copy). The subagent grades the 8 P83 catalog rows
   from artifacts with zero session context. **Verifier dispatch
   is between plans 83-01 and 83-02 — NO. The phase verifier
   dispatch happens after BOTH plans are pushed; intermediate
   phase-internal pushes do NOT trigger separate verifier dispatches.**
4. **Verdict at `quality/reports/verdicts/p83/VERDICT.md`.** Format
   per `quality/PROTOCOL.md`. Phase loops back if RED.
5. **STATE.md cursor advanced.** Update `.planning/STATE.md` Current
   Position from "P82 SHIPPED ... next P83" → "P83 SHIPPED 2026-MM-DD"
   (commit SHA cited).
6. **CLAUDE.md updated.** P83-01 T06 lands the `§ Architecture`
   bus-write-fan-out paragraph; P83-02 T04 appends the dual-table
   audit-completeness sentence + names the four shipped
   fault-injection tests.
7. **REQUIREMENTS.md DVCS-BUS-WRITE-01..06 checkboxes flipped.**
   Orchestrator (top-level) flips `[ ]` → `[x]` after verifier
   GREEN. NOT a plan task.

## Risks + mitigations

| Risk                                                                                                                                                                                                  | Likelihood | Mitigation                                                                                                                                                                                                                                                                                                                                                                                  |
|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **`apply_writes` lift accidentally drops a behavior** (e.g., L1 cursor write, mirror-head ref write, `log_helper_push_accepted` row) | MEDIUM     | S1's mechanical-lift contract + D-05's regression check via existing tests. After T02 lands, run `cargo nextest run -p reposix-remote --tests` (per-crate, sequential). Any failure = the lift broke an invariant; fix in the same task before T03 starts.                                                                                                                                  |
| **`bus_handler::push_mirror` cwd assumption fails** in some test environment | LOW        | Pitfall 6 / Assumption A1. P83-01 T05's `bus_write_happy.rs` includes a fixture that asserts the test working tree is the cwd (via `std::env::current_dir()` inside a sub-helper called from the test side, OR by asserting the mirror push lands in the expected bare repo). Document explicitly in `bus_handler.rs` module doc.                                                            |
| **`update`-hook fixture doesn't actually fail the push on macOS / non-Linux** | MEDIUM (Q-D scope) | D-04 gates the fixture `#[cfg(unix)]`. Linux + macOS both honor `update` hooks per git porcelain semantics. If macOS CI runner ever joins the workflow, validate by running the failing-mirror test locally first.                                                                                                                                                                          |
| **`wiremock::Mock::expect(0)` does NOT panic on Drop if the route was never hit** (Assumption A3) | LOW        | The donor pattern `tests/push_conflict.rs` already uses `Mock::expect(N)`; the wiremock 0.6.5 docs confirm `expect(0)` panics on Drop if the route was hit. P83-02's `bus_write_post_precheck_409.rs` uses `Mock::expect(0)` to assert "no PATCH writes happened" — verify by running the test against a passing fixture and confirming it FAILs as expected.                                |
| **Cache audit op `helper_push_partial_fail_mirror_lag` fails CHECK on stale cache.db** | LOW        | Pitfall 7 / D-03. The audit helper is best-effort (returns `()`, WARN-logs on INSERT failure). Stale caches WARN-log; fresh caches accept. NO migration script. Established P79 + P80 pattern.                                                                                                                                                                                              |
| **Mirror-fail test races with ambient cargo test parallelism** | LOW        | Each test creates its own `tempfile::tempdir()` for the bare mirror; no global state shared between tests. wiremock's per-test `MockServer::start()` returns unique ports.                                                                                                                                                                                                                  |
| **`apply_writes` returning `WriteOutcome::SotOk` with `sot_sha = None`** (i.e., refresh_for_mirror_head returned None on `files_touched == 0`) and the bus path attempting `cache.write_mirror_head(None)` | LOW-MED    | The lifted body's existing semantic (lines 558-573 of current `handle_export`) writes `head` only when `sot_sha.is_some()`. P83-01 T02 preserves this. The bus path's post-`apply_writes` block also reads `sot_sha: Option<gix::ObjectId>` and gates the synced-at write on `MirrorResult::Ok && sot_sha.is_some()` similarly.                                                              |
| **Pre-push hook BLOCKs on a pre-existing drift unrelated to P83** | LOW        | Per CLAUDE.md § "Push cadence — per-phase": treat as phase-internal failure. Diagnose, fix, NEW commit (NEVER amend), re-push. Do NOT bypass with `--no-verify`.                                                                                                                                                                                                                            |
| **`bus_handler.rs` module-doc grows past readability with the new write-fan-out additions** | LOW        | Split the module-doc into clear sections: ## Algorithm (cite architecture-sketch §3 steps 1-9 with P83 closing 4-9), ## Security (T-82-01..05 + T-83-01..03), ## Confluence non-atomicity (D-09 / Pitfall 3), ## Cwd assumption (Pitfall 6).                                                                                                                                                  |
| **Refactor in T02 collides with another running cargo invocation** | LOW        | Per CLAUDE.md "Build memory budget": one cargo invocation at a time. P83-01's tasks run strictly sequential. P83-02 starts ONLY after P83-01's terminal push lands.                                                                                                                                                                                                                          |

## +2 reservation: out-of-scope candidates

`.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` and
`GOOD-TO-HAVES.md` exist already (created during P79). P83 surfaces
candidates only when they materialize during execution.

Anticipated candidates the plan flags (per OP-8):

- **LOW** — `apply_writes` ergonomics (D-01 single-entry-point with
  caller-side synced-at deferral) might prove awkward when single-
  backend's caller block grows. If it does, file a v0.14.0
  GOOD-TO-HAVE for "extract a `write_mirror_state` helper that both
  callers invoke." NOT a P83 candidate unless caller block exceeds
  ~30 lines.
- **LOW-MED** — per-failure REST audit row (D-02 deferred). If P85
  troubleshooting docs reveal users want *"which record did the
  409 land on?"* signal exposed as audit, file a v0.14.0
  GOOD-TO-HAVE for op `helper_push_rest_failure`.
- **LOW** — `--force-with-lease` for bus push (D-08 RATIFIED no-force).
  If concurrent-push races prove common (telemetry from v0.14.0
  OTel work), revisit. NOT a P83 candidate.

Items NOT in scope for P83 (deferred per the v0.13.0 ROADMAP):

- Webhook-driven mirror sync (P84). Out of scope.
- DVCS docs (P85). Out of scope; P83-01 T06 + P83-02 T04 only
  update CLAUDE.md.
- Real-backend tests (TokenWorld + reubenjohn/reposix issues). Out
  of scope per OP-1 — milestone-close gates them.
- L2/L3 cache-desync hardening (deferred to v0.14.0).
- 30s TTL cache for cheap GH precheck (Q3.2 DEFERRED).
- Bidirectional bus / multi-SoT bus URL.

## Subagent delegation

Per CLAUDE.md "Subagent delegation rules" + the gsd-planner spec
"aggressive subagent delegation":

| Plan / Task                                                      | Delegation                                                                                                                                                                                                              |
|------------------------------------------------------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 83-01 T01 (4 catalog rows + 4 verifier shells)                  | `gsd-executor` — catalog-first commit; hand-edits agent-ux.json per documented gap (NOT Principle A). Same shape as P82 T01.                                                                                            |
| 83-01 T02 (`apply_writes` refactor + `handle_export` body shrink) | Same 83-01 executor. Cargo lock for `reposix-remote`. Per-crate cargo only. Atomic refactor commit.                                                                                                                     |
| 83-01 T03 (cache audit op + schema delta + helper + wrapper + unit test) | Same 83-01 executor. Cargo lock for `reposix-cache`. Per-crate cargo only. Atomic schema-delta-with-helper commit (D-03).                                                                                                |
| 83-01 T04 (`bus_handler` write fan-out replacing the deferred-shipped stub) | Same 83-01 executor. Cargo lock for `reposix-remote`. Per-crate cargo only.                                                                                                                                             |
| 83-01 T05 (2 integration tests + `tests/common.rs` helpers)     | Same 83-01 executor. Cargo lock for `reposix-remote` integration tests. Per-crate cargo only.                                                                                                                           |
| 83-01 T06 (catalog flip + CLAUDE.md + push)                      | Same 83-01 executor (terminal task). Pre-push gate must pass.                                                                                                                                                           |
| 83-02 T01 (4 catalog rows + 4 verifier shells)                  | `gsd-executor` (fresh invocation; sequential after 83-01 closes). Hand-edits.                                                                                                                                            |
| 83-02 T02 (mirror-fail integration test)                         | Same 83-02 executor. Cargo lock for `reposix-remote`.                                                                                                                                                                    |
| 83-02 T03 (SoT-fail + post-precheck-409 integration tests)       | Same 83-02 executor. Cargo lock for `reposix-remote`.                                                                                                                                                                    |
| 83-02 T04 (audit-completeness test + catalog flip + CLAUDE.md + push) | Same 83-02 executor (terminal task; closes phase). Pre-push gate must pass.                                                                                                                                              |
| Phase verifier (P83 close)                                       | Unbiased subagent dispatched by orchestrator AFTER 83-02 T04 pushes per `quality/PROTOCOL.md § "Verifier subagent prompt template"` (verbatim). Zero session context; grades the 8 catalog rows from artifacts.        |

Phase verifier subagent's verdict criteria (extracted for P83):

- **DVCS-BUS-WRITE-01:** `bus_handler::handle_bus_export` reads
  fast-import from stdin via `parse_export_stream`; `apply_writes`
  applies REST writes to SoT; on success writes
  `helper_push_accepted` to `audit_events_cache` AND per-record
  rows to `audit_events` AND advances `last_fetched_at`. Test
  `bus_write_happy.rs::happy_path_writes_both_refs_and_acks_ok`
  passes.
- **DVCS-BUS-WRITE-02:** on mirror-fail,
  `helper_push_partial_fail_mirror_lag` audit row written;
  `refs/mirrors/<sot>-head` updated; `synced-at` UNCHANGED; stderr
  WARN; `ok refs/heads/main` returned to git. Test
  `bus_write_mirror_fail.rs::bus_write_mirror_fail_returns_ok_with_lag_audit_row`
  passes.
- **DVCS-BUS-WRITE-03:** on mirror-success, `synced-at` advanced
  to now; `mirror_sync_written` audit row written; `ok refs/heads/main`
  returned. Asserted by the same `bus_write_happy.rs` test.
- **DVCS-BUS-WRITE-04:** no helper-side retry on transient mirror
  failure. Verifier `bus-write-no-helper-retry.sh` greps
  `crates/reposix-remote/src/bus_handler.rs` for absence of retry
  constructs (`for _ in 0..` adjacent to `push_mirror` calls; `loop {`
  blocks; `tokio::time::sleep` calls inside the bus_handler module).
  EXIT 0 if no retry construct present.
- **DVCS-BUS-WRITE-05:** P82's no-mirror-remote hint preserved
  end-to-end after P83 lands. Test
  `bus_write_no_mirror_remote.rs::bus_write_no_mirror_remote_emits_q35_hint`
  passes.
- **DVCS-BUS-WRITE-06:** three fault-injection tests + audit
  completeness all pass (P83-02 deliverable).
- New catalog rows in `quality/catalogs/agent-ux.json` (8); each
  verifier exits 0; status PASS after P83-02 T04.
- Recurring (per phase): catalog-first ordering preserved
  (P83-01 T01 + P83-02 T01 commit catalog rows BEFORE
  implementation tasks); per-phase pushes completed (one for
  P83-01 close, one for P83-02 close); verdict file at
  `quality/reports/verdicts/p83/VERDICT.md`; CLAUDE.md updated
  in P83-01 T06 + P83-02 T04.

## Verification approach (developer-facing)

After P83-02 T04 pushes and the orchestrator dispatches the verifier
subagent:

```bash
# Verifier-equivalent invocations (informational; the verifier subagent runs from artifacts):
bash quality/gates/agent-ux/bus-write-sot-first-success.sh
bash quality/gates/agent-ux/bus-write-mirror-fail-returns-ok.sh
bash quality/gates/agent-ux/bus-write-no-helper-retry.sh
bash quality/gates/agent-ux/bus-write-no-mirror-remote-still-fails.sh
bash quality/gates/agent-ux/bus-write-fault-injection-mirror-fail.sh
bash quality/gates/agent-ux/bus-write-fault-injection-sot-mid-stream.sh
bash quality/gates/agent-ux/bus-write-fault-injection-post-precheck-409.sh
bash quality/gates/agent-ux/bus-write-audit-completeness.sh
python3 quality/runners/run.py --cadence pre-pr  # re-grade catalog rows
cargo nextest run -p reposix-remote --test bus_write_happy
cargo nextest run -p reposix-remote --test bus_write_no_mirror_remote
cargo nextest run -p reposix-remote --test bus_write_mirror_fail
cargo nextest run -p reposix-remote --test bus_write_sot_fail
cargo nextest run -p reposix-remote --test bus_write_post_precheck_409
cargo nextest run -p reposix-remote --test bus_write_audit_completeness
cargo nextest run -p reposix-remote                  # full crate test sweep
cargo nextest run -p reposix-cache                   # full crate test sweep (audit helper unit test)
```

The fixtures use **wiremock SoT** (per P81's `tests/perf_l1.rs`
pattern) + **file:// bare-repo mirror with passing or failing
update hook** (P83-01 T05's `make_failing_mirror_fixture` helper).
No real-backend tests in P83 per OP-1 — milestone-close gates them.

This is a **subtle point worth flagging**: success criteria 1-3
(SoT-first / mirror-best-effort / synced-at-on-success) are
satisfied by two contracts simultaneously: (a) the helper exits
zero AND emits the expected stdout/stderr lines, AND (b) the
audit-row counts match the table in RESEARCH.md § "Audit
Completeness Contract". The integration tests assert BOTH.
