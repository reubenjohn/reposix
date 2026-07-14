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

## Chapters

- **[Decisions](./decisions.md)** — D-01..D-10: ten ratified plan-time decisions covering `apply_writes` deferral shape, audit strategy, schema atomicity, platform gating, refactor regression contract, new audit op, catalog home, mirror-push flags, Confluence non-atomicity, and cursor-advance trade-off.
- **[Architecture notes](./architecture-notes.md)** — Subtle architectural points S1 (mechanical lift contract) and S2 (`bus_handler` post-PRECHECK wiring), plus the threat model crosswalk (T-83-01..T-83-05 STRIDE register addendum).
- **[Execution](./execution.md)** — Phase-close protocol, risks + mitigations table, +2 reservation out-of-scope candidates, subagent delegation table, and developer-facing verification approach.
