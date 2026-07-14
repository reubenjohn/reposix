← [back to index](./index.md)

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

| Risk                                                                                                                                                                                                  | Likelihood | Mitigation                                                                                                                                                                                                                                                                                                                                                                  |
|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **`apply_writes` lift accidentally drops a behavior** (e.g., L1 cursor write, mirror-head ref write, `log_helper_push_accepted` row) | MEDIUM     | S1's mechanical-lift contract + D-05's regression check via existing tests. After T02 lands, run `cargo nextest run -p reposix-remote --tests` (per-crate, sequential). Any failure = the lift broke an invariant; fix in the same task before T03 starts.                                                                                                                                  |
| **`bus_handler::push_mirror` cwd assumption fails** in some test environment | LOW        | Pitfall 6 / Assumption A1. P83-01 T05's `bus_write_happy.rs` includes a fixture that asserts the test working tree is the cwd (via `std::env::current_dir()` inside a sub-helper called from the test side, OR by asserting the mirror push lands in the expected bare repo). Document explicitly in `bus_handler.rs` module doc.                                                            |
| **`update`-hook fixture doesn't actually fail the push on macOS / non-Linux** | MEDIUM (Q-D scope) | D-04 gates the fixture `#[cfg(unix)]`. Linux + macOS both honor `update` hooks per git porcelain semantics. If macOS CI runner ever joins the workflow, validate by running the failing-mirror test locally first.                                                                                                                                                          |
| **`wiremock::Mock::expect(0)` does NOT panic on Drop if the route was never hit** (Assumption A3) | LOW        | The donor pattern `tests/push_conflict.rs` already uses `Mock::expect(N)`; the wiremock 0.6.5 docs confirm `expect(0)` panics on Drop if the route was hit. P83-02's `bus_write_post_precheck_409.rs` uses `Mock::expect(0)` to assert "no PATCH writes happened" — verify by running the test against a passing fixture and confirming it FAILs as expected.                                |
| **Cache audit op `helper_push_partial_fail_mirror_lag` fails CHECK on stale cache.db** | LOW        | Pitfall 7 / D-03. The audit helper is best-effort (returns `()`, WARN-logs on INSERT failure). Stale caches WARN-log; fresh caches accept. NO migration script. Established P79 + P80 pattern.                                                                                                                                                                              |
| **Mirror-fail test races with ambient cargo test parallelism** | LOW        | Each test creates its own `tempfile::tempdir()` for the bare mirror; no global state shared between tests. wiremock's per-test `MockServer::start()` returns unique ports.                                                                                                                                                                                                  |
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
