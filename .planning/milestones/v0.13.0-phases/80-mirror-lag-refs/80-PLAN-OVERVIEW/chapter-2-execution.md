← [back to index](./index.md)

# Execution & close

## Phase-close protocol

Per CLAUDE.md OP-7 + REQUIREMENTS.md § "Recurring success criteria
across every v0.13.0 phase":

1. **All commits pushed.** Plan terminates with `git push origin main`
   in T04 (per CLAUDE.md "Push cadence — per-phase", codified
   2026-04-30, closes backlog 999.4). Pre-push gate-passing is part of
   the plan's close criterion.
2. **Pre-push gate GREEN.** If pre-push BLOCKS: treat as plan-internal
   failure (fix, NEW commit, re-push). NO `--no-verify` per CLAUDE.md
   git safety protocol.
3. **Verifier subagent dispatched.** AFTER 80-01 pushes (i.e., after
   T04 completes), the orchestrator dispatches an unbiased verifier
   subagent per `quality/PROTOCOL.md` § "Verifier subagent prompt
   template" (verbatim copy). The subagent grades the three P80 catalog
   rows from artifacts with zero session context.
4. **Verdict at `quality/reports/verdicts/p80/VERDICT.md`.** Format per
   `quality/PROTOCOL.md`. Phase loops back if verdict is RED.
5. **STATE.md cursor advanced.** Update `.planning/STATE.md` Current
   Position from "P79 SHIPPED ... next P80" → "P80 SHIPPED 2026-MM-DD"
   (commit SHA cited). Update `progress` block: `completed_phases: 3`,
   `total_plans: 7`, `completed_plans: 7`, `percent: ~27`.
6. **CLAUDE.md updated in T04.** T04's CLAUDE.md edit lands in the
   terminal commit (one paragraph: § Architecture or § Threat model
   gains the `refs/mirrors/<sot>-{head,synced-at}` namespace convention
   note + the Q2.2 staleness-window clarification).
7. **REQUIREMENTS.md DVCS-MIRROR-REFS-01..03 checkboxes flipped.**
   Orchestrator (top-level) flips `[ ]` → `[x]` after verifier GREEN.
   NOT a plan task.

## Risks + mitigations

| Risk                                                                                            | Likelihood | Mitigation                                                                                                                                                                                                                                                                                       |
|-------------------------------------------------------------------------------------------------|------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| **Annotated-tag API differs at gix 0.83** (RESEARCH.md A1)                                      | LOW        | RESEARCH.md A1 names a fallback path: two `RefEdit`s (write tag object via `repo.write_object`, point ref at it). Either way, the implementation is bounded ≤ 30 lines. T02 verifies the API name during `cargo check -p reposix-cache`; if `Repository::tag(...)` does not exist at the workspace pin, executor falls back to the two-RefEdit path and notes the API divergence in T02's commit message. |
| **`build_from` re-call cost on the success path** (S2 above)                                    | LOW        | Acceptable at P80 scale. Documented as a comment in `handle_export`; P81's L1 migration replaces this with `list_changed_since`. No SURPRISES-INTAKE entry needed — the deferral target is a planned phase.                                                                                       |
| **`stateless-connect` advertisement widening breaks vanilla `git fetch` for unrelated repos**   | LOW        | T03 widens advertisement to include `refs/mirrors/*`. The widening is additive (refs the helper publishes; never restricts what was visible). Existing vanilla-fetch tests in `crates/reposix-remote/tests/agent_flow.rs` continue to pass — confirmed by T04's full-suite `cargo nextest run -p reposix-remote`. If a regression surfaces, file as SURPRISES-INTAKE. |
| **Reject-path None-handling**                                                                   | LOW        | RESEARCH.md pitfall 7. T03 handles `read_mirror_synced_at` returning `None` (first-push case) by omitting the timestamp hint cleanly — no "synced at None ago" output. Tested via integration test `reject_hint_first_push_omits_synced_at_line` (sibling to the cited-in-reject-hint test). |
| **Catalog row's `pre-pr` cadence vs. pre-push gate**                                            | LOW        | Same pattern as P79: catalog row's `pre-pr` cadence is NOT graded by pre-push, so the row's initial FAIL status does not block the T04 terminal push. Verified by P79's successful close 2026-05-01. The runner re-grades to PASS during T04 BEFORE the push commits.                              |
| **Reflog noise on long-lived caches** (RESEARCH.md pitfall 6)                                   | LOW        | Filed as v0.14.0 operational concern. T02 adds a one-line note in `mirror_refs.rs`'s module-doc citing the deferral target — future agents find it.                                                                                                                                              |
| **CLAUDE.md update drifts from implementation**                                                 | LOW        | T04 happens AFTER T02 + T03 + integration tests pass. The CLAUDE.md text references the just-shipped behavior; no drift possible. If a test surfaces a defect (T03 fix-forward), CLAUDE.md edit reflects the shipped state, not the planned state.                                                |
| **Pre-push hook BLOCKs on a pre-existing drift unrelated to P80**                               | LOW        | Per CLAUDE.md § "Push cadence — per-phase": treat as phase-internal failure. Diagnose, fix, NEW commit (NEVER amend), re-push. Do NOT bypass with `--no-verify`.                                                                                                                                  |
| **Cargo memory pressure** (load-bearing CLAUDE.md rule)                                        | LOW        | Strict serial cargo across all four tasks. Per-crate fallback (`cargo check -p reposix-cache` then `cargo check -p reposix-remote`) is documented in each task. T01 + T04 epilogue are doc-only; T02 + T03 + T04 test-run are the cargo-bearing tasks (sequential).                                  |

## +2 reservation: out-of-scope candidates

`.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` and
`GOOD-TO-HAVES.md` exist already (created during P79). P80 surfaces
expected candidates only when they materialize during execution — none
pre-filed at planning time.

Anticipated candidates the plan flags (per OP-8):

- **LOW** — `gix::Repository::tag(...)` API name differs at the
  workspace pin (RESEARCH.md A1). Eager-resolve in T02 if fallback is
  ≤ 30 lines (it is — see RESEARCH.md A1). NOT a SURPRISES candidate.
- **LOW** — `parsed` field accessor for the SoT SHA differs
  from RESEARCH.md A2. RESOLVED at planning time: the SoT SHA derives
  from `cache.build_from()` post-write, NOT from `parsed`. NOT a
  candidate.
- **LOW** — helper's `stateless-connect` `list` advertisement widening
  is more than a one-line edit (RESEARCH.md A3). Eager-resolve in T03
  if bounded; if T03 finds the widening requires a non-trivial change
  to the protocol layer, file as SURPRISES-INTAKE for P87 absorption.
- **LOW** — Annotated-tag message-body parsing edge case (e.g.,
  multi-line tag messages from a third-party tool). The tag is written
  by THIS helper exclusively in P80; no third-party writers exist. If
  P83's bus-write or P84's webhook sync introduces a different
  message-body shape, file as SURPRISES at that time.

Items NOT in scope for P80 (deferred per the v0.13.0 ROADMAP):

- Bus remote URL parser / prechecks / writes (P82+). Mirror-lag refs
  are wired into the EXISTING single-backend `handle_export`; bus
  inheritance happens in P83.
- Webhook-driven sync (P84). Out of scope. P80 establishes the ref
  namespace; P84 layers webhook-side writes on top using the same
  cache helpers.
- L1 perf migration (P81). Out of scope. Note the `build_from`
  re-call cost as a comment (S2) but do NOT replace `list_records` in
  this phase.
- DVCS docs (P85). Out of scope; T04 only updates CLAUDE.md.
- Mirror-lag refs for bus-remote algorithm (Q2.3 — bus updates both
  refs). Out of scope; P83's territory.

## Subagent delegation

Per CLAUDE.md "Subagent delegation rules" + the gsd-planner spec
"aggressive subagent delegation":

| Plan / Task                                              | Delegation                                                                                                                                                                                                            |
|----------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 80-01 T01 (catalog rows + 3 verifier shells)             | `gsd-executor` — catalog-first commit; **hand-edits agent-ux JSON per documented gap (NOT Principle A)**; 3 verifier shells ship in same atomic commit.                                                              |
| 80-01 T02 (cache crate `mirror_refs.rs` + audit + lib)   | Same 80-01 executor. Cargo lock held for `reposix-cache`. Per-crate cargo only.                                                                                                                                       |
| 80-01 T03 (helper wiring in `handle_export` + advertise) | Same 80-01 executor. Cargo lock held for `reposix-remote`. Per-crate cargo only.                                                                                                                                       |
| 80-01 T04 (integration tests + verifier flip + CLAUDE.md + push) | Same 80-01 executor (terminal task). Cargo lock for `reposix-remote` integration test run. Per-crate cargo only.                                                                                                       |
| Phase verifier (P80 close)                               | Unbiased subagent dispatched by orchestrator AFTER 80-01 T04 pushes per `quality/PROTOCOL.md` § "Verifier subagent prompt template" (verbatim). Zero session context; grades the three catalog rows from artifacts. |

Phase verifier subagent's verdict criteria (extracted for P80):

- DVCS-MIRROR-REFS-01: `crates/reposix-cache/src/mirror_refs.rs` exists
  with `Cache::write_mirror_head` + `Cache::write_mirror_synced_at` +
  `Cache::read_mirror_synced_at` public APIs (each with `# Errors` doc);
  `audit::log_mirror_sync_written` exists; unit tests pass
  (`cargo nextest run -p reposix-cache mirror_refs`).
- DVCS-MIRROR-REFS-02: `handle_export` success branch writes both refs +
  the audit row before `proto.send_line("ok refs/heads/main")`;
  integration test `write_on_success_updates_both_refs` passes; refs
  resolvable in cache's bare repo via `git for-each-ref refs/mirrors/`;
  annotated-tag message body matches `mirror synced at <RFC3339>`.
- DVCS-MIRROR-REFS-03: integration test
  `reject_hint_cites_synced_at_with_age` passes — after a successful
  push (refs populated) followed by a stale-prior-conflict push, the
  conflict-reject stderr contains both `refs/mirrors/<sot>-synced-at`
  AND a parseable RFC3339 timestamp + `(N minutes ago)`. Sibling test
  `reject_hint_first_push_omits_synced_at_line` covers the None case.
- New catalog rows in `quality/catalogs/agent-ux.json` (3) bound
  (hand-edit per documented gap GOOD-TO-HAVES-01); each verifier exits
  0; status PASS after T04.
- OP-3: cache `audit_events_cache` table contains a row with
  `op = 'mirror_sync_written'` for the integration-test push run.
  **Unconditional — no deferral pre-authorized.**
- Recurring (per phase): catalog-first ordering preserved (T01 commits
  catalog rows BEFORE T02-T04 implementation); per-phase push
  completed; verdict file at `quality/reports/verdicts/p80/VERDICT.md`;
  CLAUDE.md updated in T04.

## Verification approach (developer-facing)

After T04 pushes and the orchestrator dispatches the verifier subagent:

```bash
# Verifier-equivalent invocation (informational; the verifier subagent runs from artifacts):
bash quality/gates/agent-ux/mirror-refs-write-on-success.sh
bash quality/gates/agent-ux/mirror-refs-readable-by-vanilla-fetch.sh
bash quality/gates/agent-ux/mirror-refs-cited-in-reject-hint.sh
python3 quality/runners/run.py --cadence pre-pr  # re-grade catalog rows
cargo nextest run -p reposix-cache mirror_refs
cargo nextest run -p reposix-remote --test mirror_refs
```

Each verifier shell follows the `quality/gates/agent-ux/reposix-attach.sh`
shape (TINY ~30-50 lines; cargo build → start sim → mktemp → scenario →
assert via `git for-each-ref` / `git log` / stderr grep). Reuse the
`REPOSIX_SIM_ORIGIN` + `REPOSIX_CACHE_DIR` env vars from P79's verifier
for cache isolation.

The fixture for the integration tests is **option (a) — `git --bare init`
local mirror** per RESEARCH.md § "Test fixture strategy". The "mirror"
in the ref name is conceptual at P80 — the actual GH-mirror push
happens in P83. P80 verifies refs propagate through the helper's
`stateless-connect` advertisement; the actual mirror endpoint is not
exercised until P83 + P84.

This is a **subtle point worth flagging again**: success criterion 3
("vanilla `git fetch` brings refs along") is satisfied by fetching from
the *cache via the helper*, not from a real GH mirror. The GH mirror
smoke test is `#[ignore]`-tagged for v0.13.0 milestone-close (per
RESEARCH.md test fixture strategy row (b)) and lives in P88 territory,
NOT P80.
