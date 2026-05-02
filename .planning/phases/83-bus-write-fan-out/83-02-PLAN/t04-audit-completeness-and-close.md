← [back to index](./index.md)

# Task 83-02-T04 — Audit-completeness test + catalog flip + CLAUDE.md addendum + per-phase push (terminal)

<read_first>
- `crates/reposix-cache/src/audit.rs` — `audit_events_cache` ops
  enumerated by the schema CHECK list (post-P83-01 T03).
- `crates/reposix-core/src/audit.rs` — `audit_events` (backend
  audit) — confirm the table + helper functions written by sim
  adapter on REST mutation success.
- `crates/reposix-sim/src/lib.rs` (or equivalent) — confirm the
  sim adapter's `audit_events` table is queryable from a test;
  the SimBackend exposes the table path OR has a test helper.
  If neither, T04's read_first surfaces this as a precondition;
  RESEARCH.md § "Audit Completeness Contract" notes the test
  may need a dedicated test sim that exposes its audit table.
  Resolution: confirm during T04 read_first; if blocking, file
  as SURPRISES-INTAKE entry per OP-8.
- `crates/reposix-remote/tests/bus_write_happy.rs` (post-P83-01
  T05) — donor pattern; `bus_write_audit_completeness.rs` is a
  superset of the happy-path test with stricter audit-row
  assertions.
- `quality/runners/run.py` — `--cadence pre-pr` invocation shape.
- `CLAUDE.md` — confirm the P83-01 T06 paragraph "Bus write
  fan-out (P83+)" is present; the addendum extends it.
- `quality/catalogs/agent-ux.json` — post-T01..T03; confirm 4
  P83-02 rows are present at FAIL; 1 P83-01 row
  (`agent-ux/bus-write-mirror-fail-returns-ok`) is also still FAIL
  (T02 shipped its test; T04 flips the row).
</read_first>

<action>
Four concerns: audit-completeness test → catalog flip → CLAUDE.md
addendum → per-phase push (terminal).

### 4a. New file — `crates/reposix-remote/tests/bus_write_audit_completeness.rs`

Setup: standard happy-path (wiremock SoT + file:// mirror with
passing hook + 2 records to update + 1 to create).

Test name: `bus_write_audit_completeness_happy_path_writes_both_tables`.

Assertions (RESEARCH.md § "Test (d)" + § "Audit Completeness
Contract"):

1. Helper exits zero, stdout `ok refs/heads/main`.
2. Open `audit_events_cache` (cache.db) via `count_audit_cache_rows`.
   Assert row counts:
   - `helper_backend_instantiated`: 1
   - `helper_push_started`: 1
   - `helper_push_accepted`: 1
   - `mirror_sync_written`: 1
   - `helper_push_partial_fail_mirror_lag`: 0
3. Open `audit_events` (sim DB or backend audit table; the test
   may need a sibling helper `count_audit_events_rows` if the sim
   doesn't expose the path directly — confirm during read_first
   and add the helper to `tests/common.rs` if missing).
   Assert row counts: per-record mutations match the executed
   actions (e.g., 2 update_record + 1 create_record = 3 backend
   audit rows for a 2-update-1-create payload).
4. Assert mirror's `main` ref points at new SoT SHA (full happy
   path — mirror push landed).
5. Assert `refs/mirrors/<sot>-head` and `refs/mirrors/<sot>-synced-at`
   both populated.

```rust
// crates/reposix-remote/tests/bus_write_audit_completeness.rs
mod common;

#[test]
fn bus_write_audit_completeness_happy_path_writes_both_tables() {
    // 1. wiremock SoT: list_records returns prior; list_changed_since
    //    returns empty; PATCH/POST/DELETE return 200 for all paths.
    // 2. Build PASSING file:// bare mirror.
    // 3. Init working tree + add bare mirror as local remote.
    // 4. Drive helper with bus URL + fast-import payload that
    //    updates id=1, updates id=2, creates id=99 (2 updates +
    //    1 create = 3 backend mutations).
    // 5. Assert exit zero, stdout contains "ok refs/heads/main".
    // 6. Assert audit_events_cache row counts via count_audit_cache_rows
    //    per the contract table.
    // 7. Open audit_events (sim DB) and assert row counts: 2
    //    update_record rows + 1 create_record row.
    //    If sim's audit_events table location requires a sibling
    //    helper, add count_audit_events_rows to tests/common.rs
    //    in this task (OR file as SURPRISES-INTAKE entry if the
    //    sim doesn't expose its audit table; OP-8).
    // 8. Assert mirror's main ref points at new SoT SHA.
    // 9. Assert refs/mirrors/<sot>-head and -synced-at populated.
    todo!("executor implements per the assertion list above")
}
```

### 4b. Cargo check + run all P83 tests (regression sweep)

```bash
cargo check -p reposix-remote --tests 2>&1 | tail -10
cargo nextest run -p reposix-remote --test bus_write_audit_completeness 2>&1 | tail -20
# expect: bus_write_audit_completeness_happy_path_writes_both_tables passes

# Full P83 sweep — all 6 bus_write_* tests pass:
cargo nextest run -p reposix-remote --test bus_write_happy \
                                    --test bus_write_no_mirror_remote \
                                    --test bus_write_mirror_fail \
                                    --test bus_write_sot_fail \
                                    --test bus_write_post_precheck_409 \
                                    --test bus_write_audit_completeness 2>&1 | tail -10

# Crate-wide sweep — regression invariant per D-05:
cargo nextest run -p reposix-remote 2>&1 | tail -10
```

### 4c. Commit the audit-completeness test (separate from catalog flip — keeps the test commit clean)

```bash
git add crates/reposix-remote/tests/bus_write_audit_completeness.rs
# If a sibling tests/common.rs helper was added (count_audit_events_rows):
# git add crates/reposix-remote/tests/common.rs
git commit -m "test(reposix-remote): bus_write_audit_completeness.rs dual-table audit assertion (DVCS-BUS-WRITE-06 audit-completeness)

- crates/reposix-remote/tests/bus_write_audit_completeness.rs — bus_write_audit_completeness_happy_path_writes_both_tables asserts:
  - audit_events_cache: helper_backend_instantiated (1) + helper_push_started (1) + helper_push_accepted (1) + mirror_sync_written (1) + helper_push_partial_fail_mirror_lag (0)
  - audit_events: per-record mutations match the executed actions (2 update_record + 1 create_record for the test's payload)
  - mirror's main ref points at new SoT SHA (full happy path — mirror push landed)
  - refs/mirrors/<sot>-head and -synced-at both populated

OP-3 dual-table audit non-optional contract enforced end-to-end.

Phase 83 / Plan 02 / Task 04 / DVCS-BUS-WRITE-06 audit-completeness."
```

### 4d. Catalog flip — flip 4 P83-02 rows + 1 lingering P83-01 row FAIL → PASS

```bash
python3 quality/runners/run.py --cadence pre-pr 2>&1 | tee /tmp/p83-02-runner.log
```

After T02 + T03 + T04a ship, ALL 5 lingering FAIL rows (4 P83-02 +
1 P83-01) flip to PASS. Confirm via:

```bash
python3 -c '
import json
data = json.load(open("quality/catalogs/agent-ux.json"))
rows = {r["id"]: r["status"] for r in data["rows"]}
required = [
    # P83-01 lingering row:
    "agent-ux/bus-write-mirror-fail-returns-ok",
    # P83-02 rows:
    "agent-ux/bus-write-fault-injection-mirror-fail",
    "agent-ux/bus-write-fault-injection-sot-mid-stream",
    "agent-ux/bus-write-fault-injection-post-precheck-409",
    "agent-ux/bus-write-audit-completeness",
]
not_pass = [(r, rows[r]) for r in required if rows.get(r) != "PASS"]
assert not not_pass, f"rows not PASS: {not_pass}"

# Sanity: confirm P83-01 rows that were already PASS stay PASS:
for r in ["agent-ux/bus-write-sot-first-success",
          "agent-ux/bus-write-no-helper-retry",
          "agent-ux/bus-write-no-mirror-remote-still-fails"]:
    assert rows.get(r) == "PASS", f"{r} regressed: {rows.get(r)}"

print("all 8 P83 rows now PASS")
'
```

### 4e. CLAUDE.md addendum — extend the P83-01 paragraph

Edit `CLAUDE.md`. Locate the P83-01-introduced "Bus write fan-out
(P83+)" paragraph in § Architecture. Append ONE additional sentence
at the end of the paragraph (DO NOT replace; extend):

```
Fault-injection coverage (DVCS-BUS-WRITE-06): four integration
tests under `crates/reposix-remote/tests/bus_write_*.rs` exercise
mirror-fail / SoT-mid-stream-fail / post-precheck-409 / dual-table
audit-completeness scenarios per RESEARCH.md § "Audit Completeness
Contract" — every push end-state writes audit rows to BOTH
`audit_events_cache` (cache-internal) AND `audit_events` (backend
mutations), enforcing the OP-3 dual-table contract.
```

### 4f. Per-phase push (terminal — closes the phase)

```bash
git add quality/catalogs/agent-ux.json \
        CLAUDE.md
git commit -m "quality(agent-ux): flip 5 P83 rows FAIL→PASS + CLAUDE.md addendum (DVCS-BUS-WRITE-06 close — phase 83 complete)

- quality/catalogs/agent-ux.json — 5 rows flipped FAIL → PASS by python3 quality/runners/run.py --cadence pre-pr:
  - agent-ux/bus-write-mirror-fail-returns-ok (lingering P83-01 row; bus_write_mirror_fail.rs landed in P83-02 T02)
  - agent-ux/bus-write-fault-injection-mirror-fail (P83-02)
  - agent-ux/bus-write-fault-injection-sot-mid-stream (P83-02)
  - agent-ux/bus-write-fault-injection-post-precheck-409 (P83-02)
  - agent-ux/bus-write-audit-completeness (P83-02)
- CLAUDE.md — § Architecture Bus write fan-out (P83+) paragraph extended with the fault-injection coverage sentence naming the four tests + the OP-3 dual-table contract

All 8 P83 catalog rows now PASS. Phase 83 closes; verifier subagent dispatch follows this push.

Phase 83 / Plan 02 / Task 04 / DVCS-BUS-WRITE-06 (close — phase 83 complete)."
git push origin main
```

If pre-push BLOCKS: treat as plan-internal failure. Diagnose, fix,
NEW commit (NEVER amend). Do NOT bypass with `--no-verify`. Re-run
`git push origin main` until it succeeds.

After this push lands, the phase verifier subagent dispatches.
The subagent grades all 8 P83 catalog rows from artifacts; the
verdict file is written to `quality/reports/verdicts/p83/VERDICT.md`.
Phase 83 close is contingent on the verdict being GREEN.
</action>

<verify>
  <automated>cargo nextest run -p reposix-remote --test bus_write_audit_completeness 2>&1 | tail -10 && python3 -c 'import json; rows = {r["id"]: r["status"] for r in json.load(open("quality/catalogs/agent-ux.json"))["rows"]}; required = ["agent-ux/bus-write-sot-first-success","agent-ux/bus-write-mirror-fail-returns-ok","agent-ux/bus-write-no-helper-retry","agent-ux/bus-write-no-mirror-remote-still-fails","agent-ux/bus-write-fault-injection-mirror-fail","agent-ux/bus-write-fault-injection-sot-mid-stream","agent-ux/bus-write-fault-injection-post-precheck-409","agent-ux/bus-write-audit-completeness"]; not_pass = [(r, rows.get(r)) for r in required if rows.get(r) != "PASS"]; assert not not_pass, f"rows not PASS: {not_pass}"' && grep -q "Fault-injection coverage (DVCS-BUS-WRITE-06)" CLAUDE.md</automated>
</verify>

<done>
- `crates/reposix-remote/tests/bus_write_audit_completeness.rs`
  exists with `bus_write_audit_completeness_happy_path_writes_both_tables`
  asserting all 9 invariants from § 4a (dual-table audit contract).
- `cargo nextest run -p reposix-remote --test bus_write_audit_completeness`
  passes.
- `cargo nextest run -p reposix-remote` passes ALL tests (regression
  invariant per D-05 confirmed end-of-phase).
- `quality/catalogs/agent-ux.json` ALL 8 P83 rows have `status: PASS`:
  - 4 P83-01 rows (`bus-write-sot-first-success`,
    `bus-write-mirror-fail-returns-ok`, `bus-write-no-helper-retry`,
    `bus-write-no-mirror-remote-still-fails`)
  - 4 P83-02 rows (`bus-write-fault-injection-mirror-fail`,
    `bus-write-fault-injection-sot-mid-stream`,
    `bus-write-fault-injection-post-precheck-409`,
    `bus-write-audit-completeness`).
- CLAUDE.md § Architecture's P83 paragraph extended with the
  fault-injection coverage sentence naming the four shipped
  tests + the OP-3 dual-table contract.
- `git push origin main` succeeded with pre-push GREEN. The plan's
  terminal commit cites DVCS-BUS-WRITE-06 close (phase 83
  complete).
- Phase 83 closes after this push; verifier subagent dispatch is
  the orchestrator-level action that follows.
</done>

---

## Plan-internal close protocol

After T04 push lands, P83-02 (and Phase 83 as a whole) transitions
out of the executor's hands. The orchestrator (top-level
coordinator) handles the remaining steps:

1. **Verifier subagent dispatched.** Unbiased subagent per
   `quality/PROTOCOL.md § "Verifier subagent prompt template"`
   (verbatim copy). Grades the 8 P83 catalog rows from artifacts
   with zero session context.
2. **Verdict at `quality/reports/verdicts/p83/VERDICT.md`.** Format
   per `quality/PROTOCOL.md`. Phase loops back if RED.
3. **STATE.md cursor advanced.** Update `.planning/STATE.md` Current
   Position from "P82 SHIPPED ... next P83" → "P83 SHIPPED 2026-MM-DD"
   (commit SHA cited).
4. **REQUIREMENTS.md DVCS-BUS-WRITE-01..06 checkboxes flipped.**
   `[ ]` → `[x]` after verifier GREEN. NOT a plan task.
5. **+2 reservation.** If T02–T04 surfaced any out-of-scope items
   (e.g., the sim audit_events table location issue from § 4a
   read_first; D-04 cfg(unix) fixture portability if macOS CI
   surfaces a difference; D-02 deferred per-failure REST audit op
   if user demand emerges from P85 troubleshooting docs), the
   discovering subagent appended them to
   `.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` (per
   OP-8). The verifier subagent's honesty spot-check confirms
   whether the intake reflects what was actually observed.

NONE of these steps are plan tasks — they are orchestrator actions
following the per-phase-push contract.
