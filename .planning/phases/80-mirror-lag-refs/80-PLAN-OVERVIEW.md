---
phase: 80
title: "Mirror-lag refs (`refs/mirrors/<sot>-head`, `<sot>-synced-at`)"
milestone: v0.13.0
requirements: [DVCS-MIRROR-REFS-01, DVCS-MIRROR-REFS-02, DVCS-MIRROR-REFS-03]
depends_on: [79]
plans:
  - 80-01-PLAN.md  # DVCS-MIRROR-REFS-01..03 (catalog → cache impl → helper wiring → integration tests + close)
waves:
  1: [80-01]
---

# Phase 80 — Mirror-lag refs (overview)

This is the SECOND DVCS-substantive phase of milestone v0.13.0 — the
"observability" leg of the bus-remote story. It lands BEFORE the bus
remote ships (P82–P83) so the refs are already in place when the bus
inherits the wiring point. **Single plan, four sequential tasks** per
RESEARCH.md § "Plan splitting":

- **T01 — Catalog-first.** Three rows in `quality/catalogs/agent-ux.json`
  (`mirror-refs-write-on-success`, `mirror-refs-readable-by-vanilla-fetch`,
  `mirror-refs-cited-in-reject-hint`) + three TINY shell verifiers under
  `quality/gates/agent-ux/`. Initial status `FAIL`. Hand-edit per
  documented gap (NOT Principle A) — same shape as P79's
  `agent-ux/reposix-attach-against-vanilla-clone` row, mints tracked by
  GOOD-TO-HAVES-01.
- **T02 — Cache crate impl.** New module `crates/reposix-cache/src/mirror_refs.rs`
  (writer + reader, mirroring `sync_tag.rs` shape verbatim); new
  `audit::log_mirror_sync_written`; pub mod + re-exports in `lib.rs`.
  Unit tests for writer/reader round-trip + annotated-tag message-body
  parsing. Per-crate cargo only (`cargo check -p reposix-cache`,
  `cargo nextest run -p reposix-cache`).
- **T03 — Helper crate wiring.** Insert two ref writes (head + synced-at)
  + audit-row write into `handle_export`'s success branch (lines 469–489
  per `crates/reposix-remote/src/main.rs` as of 2026-05-01); add
  `refs/mirrors/*` to the helper's stateless-connect ref advertisement;
  compose reject-hint stderr from `cache.read_mirror_synced_at` (None →
  omit hint cleanly per RESEARCH.md pitfall 7). Per-crate cargo only.
- **T04 — Integration tests + verifier flip + CLAUDE.md update + close.**
  Three integration tests in `crates/reposix-remote/tests/mirror_refs.rs`
  (one per catalog row); flip catalog rows FAIL → PASS via the runner;
  CLAUDE.md update (one paragraph in § Architecture / Threat model);
  per-phase `git push origin main`. Per-crate cargo only.

Sequential — never parallel. Even though T02 (cache) and T03 (helper)
touch different crates, sequencing per CLAUDE.md "Build memory budget"
rule (one cargo invocation at a time, never two in parallel) and per
RESEARCH.md § "Test fixture strategy" makes this strictly sequential.

## Wave plan

Strictly sequential — one plan, four tasks. T01 → T02 → T03 → T04
within the same plan body. The plan is its own wave.

| Wave | Plans  | Cargo? | File overlap        | Notes                                                                                    |
|------|--------|--------|---------------------|------------------------------------------------------------------------------------------|
| 1    | 80-01  | YES    | none with prior phase | catalog + cache crate + helper crate + integration tests + close — all in one plan body |

`files_modified` audit (single-plan phase, no cross-plan overlap to
audit):

| Plan  | Files                                                                                                                                                                                                                                                                          |
|-------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| 80-01 | `crates/reposix-cache/src/mirror_refs.rs` (new), `crates/reposix-cache/src/audit.rs`, `crates/reposix-cache/src/lib.rs`, `crates/reposix-remote/src/main.rs`, `crates/reposix-remote/src/stateless_connect.rs`, `crates/reposix-remote/tests/mirror_refs.rs` (new), `quality/catalogs/agent-ux.json`, `quality/gates/agent-ux/mirror-refs-write-on-success.sh` (new), `quality/gates/agent-ux/mirror-refs-readable-by-vanilla-fetch.sh` (new), `quality/gates/agent-ux/mirror-refs-cited-in-reject-hint.sh` (new), `CLAUDE.md` |

Per CLAUDE.md "Build memory budget" the executor holds the cargo lock
sequentially across T02 → T03 → T04. No parallel cargo invocations.
Doc-only tasks (T01: catalog row + verifier shell scaffolding;
T04 epilogue: CLAUDE.md edit) do NOT compile and may interleave freely
with other doc-only work outside this phase if the orchestrator schedules
them — but within this plan they remain sequential for executor
simplicity.

## Plan summary table

| Plan  | Goal                                                                                       | Tasks | Cargo?  | Catalog rows minted        | Tests added                                                                                  | Files modified (count) |
|-------|--------------------------------------------------------------------------------------------|-------|---------|----------------------------|----------------------------------------------------------------------------------------------|------------------------|
| 80-01 | Mirror-lag refs (cache helpers + helper wiring + advertise + reject-hint + tests + close)  | 4     | YES     | 3 (status FAIL → PASS at T04) | 4 unit (writer/reader round-trip + annotated-tag message + read-none-when-absent + ref-name-validation) + 3 integration (one per catalog row) | ~11 (1 new cache module + 1 new test file + 3 new verifier shells + cache audit + helper wiring + advertise + catalog + CLAUDE.md) |

Total: 4 tasks across 1 plan. Wave plan: sequential.

Test count: 4 unit (in `mirror_refs.rs` `#[cfg(test)] mod tests`) + 3
integration (in `crates/reposix-remote/tests/mirror_refs.rs`) = 7 total.

## Subtle architectural points (read before T03)

The two below are flagged because they are the most likely sources of
T02 → T03 review friction. Executor must internalize them before writing
the wiring code.

### S1 — Refs live in the CACHE's bare repo, not the working tree

The `refs/mirrors/<sot>-head` + `refs/mirrors/<sot>-synced-at` refs are
written to **the cache's bare repo** (`Cache::repo` — a `gix::Repository`
handle on `<cache-root>/reposix/<backend>-<project>.git/`), NOT to the
working tree's `.git/` directory. The working tree receives them via
the helper's `stateless-connect` `list` advertisement (T03 widens this
from `refs/heads/main` only to also include `refs/mirrors/*`).

**Why this matters for T03.** A reviewer skimming the wiring may expect
the helper to call something like `git update-ref` on the working tree's
`.git/` — that would be wrong. The helper has no working-tree handle in
`handle_export`. It has `state.cache: Option<Cache>`, and the cache owns
the bare repo. Mirror refs land in the cache; vanilla `git fetch` from
the working tree pulls them across via the helper.

This also resolves RESEARCH.md pitfall 4 ("Cache vs. working-tree
confusion"). Document the cache-as-source-of-truth point in the new
`mirror_refs.rs` module-doc (T02) and in CLAUDE.md (T04 epilogue).

### S2 — `sot_sha` is the cache's post-write synthesis-commit OID

The `<sot-host>-head` ref records "the SHA of the SoT's main at last
sync." For the single-backend push path (P80 scope), the helper does
NOT push to a real GH mirror — that wiring lands in P83. So the SHA
the ref records is the cache's **post-write synthesis-commit OID**:
the OID that `Cache::build_from()` returns AFTER the SoT REST writes
have landed.

The implementation in T03 calls `cache.build_from().await?` AFTER
`execute_action` succeeded for every action and BEFORE the
`proto.send_line("ok refs/heads/main")`. The returned `gix::ObjectId`
is what's written to `refs/mirrors/<sot>-head`. This is verified
against `crates/reposix-cache/src/builder.rs:56` (`pub async fn build_from(&self) -> Result<gix::ObjectId>`).

**Why not `parsed.commits.last()`?** RESEARCH.md Assumption A2 noted the
ParsedExport shape was uncertain. Verified at planning time: `ParsedExport`
has `commit_message`, `blobs`, `tree`, `deletes` — NO `commits` field.
The synthesis-commit OID is what's actually meaningful for the mirror-head
contract: it's the SHA the cache's bare repo presents to vanilla
`git fetch` after this push. Use `cache.build_from()` (already a
helper-internal API) — do NOT introduce a new accessor.

Trade-off this introduces: `build_from` is "tree sync = full" per
`crates/reposix-cache/src/lib.rs:7-10` — calling it a second time per
push touches the full tree-list cycle. For single-backend push at
P80 scale (one space, dozens to hundreds of issues) this is acceptable.
P81's L1 migration (`list_changed_since`) would let us avoid this
re-list, but P81 ships AFTER P80. Document the cost as a comment
adjacent to the `build_from` call in `handle_export`; cite the
`.planning/research/v0.13.0-dvcs/architecture-sketch.md § "Performance subtlety"`
deferral target.

## Hard constraints (carried into the plan body)

Per the user's directive (orchestrator instructions for P80) and
CLAUDE.md operating principles:

1. **Catalog-first (QG-06).** T01 mints THREE rows + verifier shells
   BEFORE T02-T04 implementation. Initial status `FAIL`. Hand-edit per
   documented gap (NOT Principle A) — annotated in commit message
   referencing GOOD-TO-HAVES-01.
2. **Per-crate cargo only (CLAUDE.md "Build memory budget").** Never
   `cargo --workspace`. Use `cargo check -p reposix-cache`,
   `cargo check -p reposix-remote`, `cargo nextest run -p <crate>`.
   Pre-push hook runs the workspace-wide gate; phase tasks never duplicate.
3. **Sequential execution.** Tasks T01 → T02 → T03 → T04 — never parallel,
   even though T02 (cache crate) and T03 (helper crate) touch different
   crates. CLAUDE.md "Build memory budget" rule is "one cargo invocation
   at a time" — sequencing the tasks naturally honors this.
4. **OP-3 audit unconditional.** When `handle_export` writes the refs on
   success, the cache logs an `audit_events_cache` row with
   `op = 'mirror_sync_written'`. Ref writes are best-effort (log WARN +
   continue per RESEARCH.md pattern §"Wiring `handle_export` success path");
   the audit row is UNCONDITIONAL — even if both ref writes failed,
   the audit row records the event. T03's wiring uses the same shape as
   the existing `log_helper_push_accepted` precedent (line 471).
5. **Per-phase push BEFORE verifier (CLAUDE.md "Push cadence — per-phase",
   codified 2026-04-30).** T04 ends with `git push origin main`; pre-push
   gate must pass; verifier subagent grades the three catalog rows
   AFTER push lands. Verifier dispatch is an orchestrator-level action
   AFTER this plan completes — NOT a plan task.
6. **CLAUDE.md update in same PR (QG-07).** T04 documents the
   `refs/mirrors/<sot-host>-{head,synced-at}` namespace + the Q2.2
   doc-clarity contract carrier (full docs treatment defers to P85).
7. **No new error variants.** Per RESEARCH.md "Wiring `handle_export`
   success path", ref-write failure is best-effort with a
   `tracing::warn!` line. The push still acks `ok` to git. This
   matches the existing `Cache::log_*` family pattern (let-else +
   `let _ = ...` drop) and introduces NO new `RemoteError` variant
   nor new `cache::Error` variant.

## Threat model crosswalk

Per CLAUDE.md § "Threat model" — this phase introduces NO new
trifecta surface beyond what the cache + helper already have:

| Existing surface              | What P80 changes                                                                                                     |
|-------------------------------|----------------------------------------------------------------------------------------------------------------------|
| Cache writes to bare repo     | NEW ref namespace `refs/mirrors/...` — gix-validated names; `sot_host` slug from `state.backend_name` (controlled enum), NOT user input. STRIDE category: Tampering — mitigated by `gix::refs::FullName::try_from` rejecting `..`, `:`, control bytes. |
| Helper outbound HTTP          | UNCHANGED — refs are local to the cache; no new HTTP construction site.                                              |
| Reject-message stderr         | NEW: stderr now cites `refs/mirrors/<sot>-synced-at` timestamp (when present). Bytes copied from cache's annotated-tag message body — these are written by the helper itself (no taint propagation from the SoT). RFC3339 string + arithmetic only. STRIDE category: Information Disclosure (misleading) — mitigated by Q2.2 doc clarity contract carrier. |
| Audit log                     | NEW op `mirror_sync_written` in `audit_events_cache`. Schema unchanged (`op` column is `TEXT`, no DDL change). UNCONDITIONAL per OP-3. |

No `<threat_model>` STRIDE register addendum required — the new
surfaces map to existing categories with existing mitigations. Plan
body's `<threat_model>` section enumerates the three threats per
CLAUDE.md template requirements.

Reflog growth on long-lived caches (RESEARCH.md pitfall 6) is filed as
a v0.14.0 operational concern. P80 adds a one-line note in
`mirror_refs.rs` so a future agent finds it.

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
