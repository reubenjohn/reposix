# P81 verdict — L1 perf migration (`list_changed_since`-based conflict detection + `reposix sync --reconcile`)

**Verifier:** unbiased subagent · zero session context
**Verdict:** **GREEN** (with three advisory items — see § Advisory items)
**Date:** 2026-05-01
**Milestone:** v0.13.0 — DVCS over REST
**Precondition:** P80 verdict GREEN at `quality/reports/verdicts/p80/VERDICT.md` (confirmed)

---

## Catalog row grading

All 3 P81 catalog rows were re-graded by invoking each verifier script
directly from a fresh shell (zero session context):

| Catalog row                                             | Verifier exit | Asserts | Status   |
| ------------------------------------------------------- | ------------- | ------- | -------- |
| `perf/handle-export-list-call-count`                    | 0             | 3/3     | **PASS** |
| `agent-ux/sync-reconcile-subcommand`                    | 0             | 3/3     | **PASS** |
| `docs-alignment/perf-subtlety-prose-bound`              | walk PASS     | n/a     | **PASS (BOUND)** |

Run-time evidence:

```text
bash quality/gates/perf/list-call-count.sh
  → cargo test -p reposix-remote --test perf_l1
        l1_precheck_uses_list_changed_since_not_list_records
  → 1 passed; 0 failed
  → "PASS: L1 precheck makes >=1 list_changed_since calls AND
     zero list_records calls (N=200 wiremock harness)"

bash quality/gates/agent-ux/sync-reconcile-subcommand.sh
  → cargo build -p reposix-cli
  → reposix sync --reconcile --help (exit 0)
  → cargo test -p reposix-cli --test sync sync_reconcile_advances_cursor
  → 1 passed; 0 failed
  → "PASS: reposix sync --reconcile help+subcommand smoke green"

bash quality/gates/docs-alignment/walk.sh
  → exit 0 (zero new STALE_DOCS_DRIFT for the perf-subtlety row)
  → catalog row last_verdict=BOUND, next_action=BIND_GREEN
```

Catalog `status: PASS` and `last_verified` timestamps for the perf and
agent-ux rows (`2026-05-01T11:37:34Z` / `2026-05-01T11:37:31Z`) are
consistent with the executor's claim. The doc-alignment row uses the
walk model (BOUND last_verdict) per docs-alignment dimension semantics.

---

## REQ-ID grading (DVCS-PERF-L1-01..03)

| REQ-ID         | Status | Evidence                                                                                                                                                                                                                                                                                                                                                                                              |
| -------------- | ------ | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| DVCS-PERF-L1-01 | PASS  | `crates/reposix-remote/src/precheck.rs:80-302` (`precheck_export_against_changed_set`) replaces unconditional `list_records` with cursor-based `list_changed_since` (`:101-103`); `crates/reposix-remote/src/main.rs:351-373` calls precheck from `handle_export` and routes Conflicts/Proceed; `crates/reposix-cache/src/cache.rs:443-484` ships `read_last_fetched_at` + `write_last_fetched_at` cursor methods; cursor write at `main.rs:496` after `log_helper_push_accepted`. Behaviorally proven by `perf_l1.rs::l1_precheck_uses_list_changed_since_not_list_records` (N=200 wiremock; expect(0) on `NoSinceQueryParam` + expect(1..) on `HasSinceQueryParam`). |
| DVCS-PERF-L1-02 | PASS  | `crates/reposix-cli/src/sync.rs` (entire file) implements `reposix sync [--reconcile]` calling `Cache::sync()`; `crates/reposix-cli/src/main.rs:181-185, 396` wires `Sync { reconcile, path }` clap-derive variant + dispatch. Behaviorally proven by `crates/reposix-cli/tests/sync.rs::sync_reconcile_advances_cursor` (3-test smoke suite: cursor-advance + help-renders + bare-form-prints-hint). |
| DVCS-PERF-L1-03 | PASS  | `precheck.rs` is `pub(crate)` and parametrized via narrow deps (`cache, backend, project, rt, parsed`) per M1 — no `&mut State` coupling. Module doc-comment (`precheck.rs:13-14`) names "Both `handle_export` (P81) and the future bus handler (P82+) call this same function." L2/L3 deferral is documented in 3 places per D-01: precheck.rs:5-7 (module doc), main.rs:516-518 (no-op skip rationale), CLAUDE.md:31 (Architecture paragraph closing sentence). |

---

## Honesty spot-check — perf regression test non-vacuity

The catalog row `perf/handle-export-list-call-count` and the prose row
`docs-alignment/perf-subtlety-prose-bound` BOTH cite the same single
test: `crates/reposix-remote/tests/perf_l1.rs::l1_precheck_uses_list_changed_since_not_list_records`.
A vacuous test (e.g., one that never enters the helper subprocess, or
mounts mocks the helper can't reach) would silently flip both rows to
PASS without proving anything. I read the test body end-to-end (lines
204-320) AND the positive-control sibling (lines 328-411):

**Non-vacuity proofs:**

1. **Helper subprocess actually runs** (line 313): the test asserts
   `out.status.success()` on the `git-remote-reposix` subprocess after
   feeding it a fast-export stream — the helper IS exercised, not a
   stub.

2. **Both matchers attach to mocks with active expectations** (line 263-284):
   - `NoSinceQueryParam` matcher → mock with `.expect(0).with_priority(1)`
     — wiremock's `MockServer::Drop` panics if >0 requests match.
   - `HasSinceQueryParam` matcher → mock with `.expect(1..).with_priority(1)`
     — wiremock's `MockServer::Drop` panics if 0 requests match.
   - The custom matchers are real `wiremock::Match` impls (lines 32-49)
     not no-op stubs: `NoSinceQueryParam` does `req.url.query_pairs().all(|(k,_)| k != "since")`,
     `HasSinceQueryParam` does the symmetric `.any(...)`.

3. **Priority-tier carve-out documented** (lines 252-258): the assertion
   mocks use `priority=1` (vs. the default-5 setup mocks) so wiremock's
   "lowest-priority-wins" rule routes helper traffic to the assertion
   mocks, not the warm-cache setup mocks. Without this, the `expect(0)`
   would be silently bypassed by setup-time `list_records` calls and
   the test would falsely pass.

4. **Positive control fails RED on flip** (lines 328-411): the
   `positive_control_list_records_call_fails_red` test mounts the SAME
   matcher with `expect(1)` on `NoSinceQueryParam` (the helper does
   NOT call list_records on the hot path → this expectation is unmet
   → wiremock Drop panics). The test is annotated
   `#[should_panic(expected = "Verifications failed")]` confirming the
   panic message contains the wiremock assertion-fail string. This
   closes RESEARCH.md MEDIUM "wiremock semantics need confirmation
   during Task 4".

5. **Real workload, not a smoke** (lines 211, 248): N=200 records
   seeded; `warm_cache(&cache, n)` materializes all 200 blobs via the
   async `read_blob` path so the precheck's `read_blob_cached` succeeds
   and the steady-state hot path (Step 5 of precheck.rs) does NOT fall
   through to the lazy-materialization gap fallback (which would itself
   fire ONE list_records call and break the test).

**Honesty grade: test is substantive, NOT vacuous.** RESEARCH.md MEDIUM
risk closed. Both catalog rows that depend on this test (`perf/...` AND
`docs-alignment/perf-subtlety-prose-bound`) are riding on a real proof.

---

## Plan-checker H1-H4 honesty spot-check

PLAN-CHECK.md graded RED with 4 HIGH issues (`81-l1-perf-migration/PLAN-CHECK.md:8`).
The planner revised; did the executor implement the revisions faithfully?

### H1 — `Cache::read_blob` is async + does hidden backend GET

**Required fix:** introduce sync, gix-only `read_blob_cached`; precheck
must use it instead of `read_blob`.

**Verification:**
- `crates/reposix-cache/src/cache.rs:509-527` ships
  `pub fn read_blob_cached(&self, oid) -> Result<Option<Tainted<Vec<u8>>>>` —
  synchronous (no `async fn`), uses `self.repo.try_find_object(oid)`
  (gix-local, no `await`, no backend touch), returns `Ok(None)` on
  cache-miss instead of fetching.
- `crates/reposix-remote/src/precheck.rs:259-264` calls
  `c.read_blob_cached(oid)` (NOT `read_blob`) on the materialize-prior
  path. Module doc at `precheck.rs:21-24` explicitly anti-patterns
  `read_blob`: "Don't call [`reposix_cache::Cache::read_blob`] here —
  it is async AND fetches from the backend on cache miss."
- `grep -n 'read_blob[^_]' crates/reposix-remote/src/precheck.rs` → 0 hits
  (the only references to `read_blob` in precheck are in the doc-comment
  warning).

**H1 closed: substantive fix.**

### H2 — `Tainted::peek()` does not exist

**Required fix:** use `inner_ref()` instead of `peek()`.

**Verification:**
- `grep -rn '\.peek()' crates/reposix-remote/src/ crates/reposix-cache/src/` → 0 hits.
- `crates/reposix-remote/src/precheck.rs:265` uses
  `String::from_utf8_lossy(blob.inner_ref())` — the actual API per
  `crates/reposix-core/src/taint.rs:48`.

**H2 closed: substantive fix.**

### H3 — invalid `crate::main::*` import path / State privacy

**Required fix:** widen State to `pub(crate)` so siblings can reach it.

**Verification:**
- `crates/reposix-remote/src/main.rs:48` declares `pub(crate) struct State { ... }`.
- The four fields the precheck path requires are `pub(crate)`:
  `:49 rt`, `:50 backend`, `:60 project`, `:76 cache` (4 of 7 fields
  widened; `backend_name`, `cache_project`, `push_failed`,
  `last_fetch_want_count` remain private — precheck doesn't consume them).
- Module doc at `main.rs:43-47` documents the Q3.1 ratification:
  *"the sibling `crate::precheck` module can access these fields without
  reaching into the binary root via the (invalid) `crate::main::*` path.
  The other fields stay private — the precheck does not consume them."*

**Note on import shape:** the prompt specified that precheck should
import `use crate::{State, issue_id_from_path};`. The actual import is
`use crate::{fast_import::ParsedExport, issue_id_from_path};` (precheck.rs:34-35)
— `State` is NOT imported because the M1 fix narrowed the precheck
signature to take `(cache, backend, project, rt, parsed)` directly
rather than `&mut State`. The State widening was made for `pub(crate)`
field-access *from the call site in main.rs* (`state.cache.as_ref()`,
`state.backend.as_ref()`, `&state.project`, `&state.rt`) — see
`main.rs:351-356`. The original H3 concern (`crate::main::*` invalid
import path) is moot because precheck does not need a State reference;
the widening is still needed because the call-site does
`&state.cache` etc through `pub(crate)` field-access. M1 supersedes
H3's literal import-shape; the concern (privacy / sibling access) is
resolved.

**H3 closed: substantive fix (widening + M1 narrow-deps refactor).**

### H4 — typed `Error::BackendUnreachable` / `Error::Cache` don't exist

**Required fix:** the remote crate uses `anyhow::Result`; precheck must
not import `crate::error::*`.

**Verification:**
- `crates/reposix-remote/src/precheck.rs:28` uses
  `use anyhow::{Context, Result};` (NOT `use crate::error::*;`).
- `grep -n 'crate::error\|Error::' crates/reposix-remote/src/precheck.rs` → 0 hits
  for `crate::error::*` or `Error::BackendUnreachable` / `Error::Cache`.
- REST call sites use `.context("backend-unreachable: list_changed_since")`
  (precheck.rs:103), `.context("backend-unreachable: list_records (first-push)")`
  (precheck.rs:113), `.with_context(|| format!("backend-unreachable: get_record({id:?})"))`
  (precheck.rs:207). Caller maps to existing `fail_push(diag, "backend-unreachable", ...)`
  shape (main.rs:368-372).

**H4 closed: substantive fix.**

**All 4 HIGH PLAN-CHECK fixes are present in code. No watered-down workarounds.**

---

## CLAUDE.md update confirmation (D-05)

`git show d21160c -- CLAUDE.md` shows the in-phase update required by
QG-07 + the plan's D-05 contract. Two distinct edits:

1. **§ Architecture — new "L1 conflict detection (P81+)" paragraph**
   (CLAUDE.md line 31). Verbatim core sentences:

   > "On every push, the helper reads its cache cursor
   > (`meta.last_fetched_at`), calls `backend.list_changed_since(since)`,
   > and only conflict-checks records that overlap the push set with
   > the changed-set. The cache is trusted as the prior; the agent's
   > PATCH against a backend-deleted record fails at REST time with a
   > 404 — recoverable via `reposix sync --reconcile`
   > (DVCS-PERF-L1-02). On the cursor-present hot path, the precheck
   > does ONE `list_changed_since` REST call plus ONE `get_record` per
   > record in `changed_set ∩ push_set` (typically zero or one); the
   > legacy unconditional `list_records` walk in `handle_export` is
   > gone. First-push fallback (no cursor yet) and steady-state pushes
   > with no actions executed (`files_touched == 0`) skip the
   > post-write `refresh_for_mirror_head` to keep the no-op cost at
   > zero list-records calls. L2/L3 hardening (background reconcile /
   > transactional cache writes) defers to v0.14.0 per
   > `.planning/research/v0.13.0-dvcs/architecture-sketch.md`
   > § Performance subtlety."

   All required D-05 elements present:
   - L1 trade-off (cache trusted as prior) — YES
   - Backend-side deletes surface as REST 404 on PATCH — YES
   - Recovery via `reposix sync --reconcile` — YES
   - L2/L3 deferral citation — YES (architecture-sketch.md § Performance subtlety)
   - Plus a P81-specific clarification: the no-op-skip pattern that
     drops the no-op-push list_records count from 1 → 0 (the
     refresh_for_mirror_head no-op skip; SURPRISES entry P81-T04).

2. **§ Commands — new `reposix sync --reconcile` bullet** (CLAUDE.md line 179):

   ```
   reposix sync --reconcile                                  # full list_records walk + cache rebuild (DVCS-PERF-L1-02)
   ```

**D-05 / QG-07 satisfied.** Both required updates present and integrated
into existing sections (NOT appended as a narrative addendum, matching
CLAUDE.md "revising existing sections to reflect the new state").

---

## SURPRISES-INTAKE review

`.planning/milestones/v0.13.0-phases/SURPRISES-INTAKE.md` has 2 P81
entries (matching the prompt's expectation):

| # | Discovered-by | Severity | What | STATUS |
| - | ------------- | -------- | ---- | ------ |
| 1 | P81-01-T01    | LOW      | bind-verb schedule shift T01→T04 (bind verb validates test-file-on-disk; perf_l1.rs lands T04) | OPEN |
| 2 | P81-01-T04    | LOW      | refresh_for_mirror_head fires unconditional list_records on success path; no-op-skip when files_touched=0 | RESOLVED |

**Intake judgment:**
- **Entry 1 (OPEN):** legitimate scope-bound deferral. The fix is a
  1-line schedule shift (mint the docs-alignment row in T04 alongside
  perf_l1.rs creation, not in T01). P88 may evaluate whether the bind
  verb should accept a `--test-pending` flag for true catalog-first
  contracts where the test ships in a later commit of the same phase.
  Not a P81 blocker.
- **Entry 2 (RESOLVED):** eager-resolution per OP-8. Single-edit fix
  in main.rs:516-528 (skip `refresh_for_mirror_head` when
  `files_touched == 0`); justified because no tree change occurred,
  so the existing mirror-head ref still reflects the prior tree's OID.
  Self-healing on next non-trivial push. CLAUDE.md L1 paragraph
  documents the no-op-skip semantics.

Neither entry was an actual scope-doubling situation. Eager-resolution
honesty on entry 2: the executor chose the smallest correct change
(the no-op skip) rather than the architecturally-complete one (rewrite
`refresh_for_mirror_head` itself to use `list_changed_since`), explicitly
deferring the larger refactor to v0.14.0 alongside L2/L3 cache-desync
hardening. Honest framing.

OP-8 honesty check: P81-01 plan body did NOT include the
`refresh_for_mirror_head` fix in `<must_haves>` even though the success
path was inside the phase's surface — the plan-author missed it. The
intake entry acknowledges this directly ("the P81 plan body's
`<must_haves>` did not include the replacement"). Eager-resolution +
intake journal is the correct response.

---

## Pre-push gate snapshot

```text
$ python3 quality/runners/run.py --cadence pre-push
summary: 26 PASS, 0 FAIL, 0 PARTIAL, 0 WAIVED, 0 NOT-VERIFIED -> exit=0
```

26/26 GREEN. Matches executor's T04 push-time claim. The terminal
`git push origin main` for `4869545` landed on origin/main per
`git log origin/main -7` — all 5 P81 commits visible:

```
4869545 quality(docs-alignment): refresh cli-subcommand-surface row source_hashes for P81 Sync variant addition
d21160c test(remote): N=200 perf regression + positive control + flip catalogs FAIL→PASS + CLAUDE.md update (DVCS-PERF-L1-01..03 close)
9321499 feat(cli): reposix sync --reconcile subcommand (DVCS-PERF-L1-02)
1bd50b5 feat(cache,remote): L1 precheck — read_last_fetched_at + precheck.rs + handle_export rewrite (DVCS-PERF-L1-01, DVCS-PERF-L1-03)
3a44a9e quality(perf,agent-ux): mint L1-perf catalog rows + 2 TINY verifiers (P81-01 T01 catalog-first)
```

---

## Phase-close protocol

| Item                                                              | Status                                                                                                                                                              |
| ----------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| All 3 catalog rows graded PASS                                    | YES (re-verified with zero session context; verifier exit 0 for both shell rows + walk-PASS / BOUND for the doc-alignment row)                                      |
| All 5 P81 commits pushed                                          | YES — `3a44a9e`, `1bd50b5`, `9321499`, `d21160c`, `4869545` all on `origin/main` (verified via `git log origin/main -7`)                                            |
| Pre-push gate locally GREEN                                       | YES — 26 PASS / 0 FAIL                                                                                                                                              |
| CLAUDE.md updated in-phase (QG-07 + D-05)                         | YES — both Architecture paragraph + § Commands bullet in commit `d21160c`; L1 trade-off + sync-reconcile recovery + L2/L3 deferral citation all present             |
| Catalog-first contract honored                                    | YES — T01 commit `3a44a9e` mints catalog rows + verifier shells with `status: FAIL`; T04 commit `d21160c` flips them PASS only after the implementation lands       |
| Push BEFORE verifier dispatch                                     | YES — pre-push gate ran on each of the 5 commits (per CLAUDE.md per-phase push cadence)                                                                            |
| Plan SUMMARY present                                              | **NO — 81-01-SUMMARY.md MISSING** (advisory; same shape as P79/P80's missing SUMMARYs)                                                                              |
| REQUIREMENTS.md DVCS-PERF-L1-01..03 checkboxes flipped            | **NO — still `[ ]`** (advisory; standard close-out chore — same shape as P79/P80)                                                                                   |
| STATE.md cursor updated                                           | **NO — still shows P80 SHIPPED, P81 next** (advisory; same shape as P79/P80)                                                                                        |
| SURPRISES-INTAKE updated for P81 deviations                       | YES — 2 entries journaled (T01 OPEN + T04 RESOLVED) with eager-resolution rationale                                                                                 |
| H1-H4 PLAN-CHECK fixes present in code                            | YES — all 4 fixes verified file:line above                                                                                                                          |
| Perf regression test non-vacuous                                  | YES — both matchers attach to active expectations; positive control panics on flip via `#[should_panic]`                                                            |

---

## Advisory items (do NOT block GREEN)

1. **81-01-SUMMARY.md missing.** No phase-close summary file exists
   under `.planning/phases/81-l1-perf-migration/`. Recommend orchestrator
   authors a thin `81-01-SUMMARY.md` post-hoc citing `d21160c` (or
   `4869545` if the housekeeping refresh-row is treated as the phase's
   terminal commit) as the close commit. Not a GREEN-blocker —
   close-out is reconstructable from PLAN + commit messages + this
   verdict. Same advisory as P79/P80; pattern is now an established
   v0.13.0 close-out chore that the orchestrator handles after
   verifier-grading.

2. **REQUIREMENTS.md + STATE.md not flipped.** DVCS-PERF-L1-01..03
   checkboxes still `[ ]`; STATE.md still shows P80 as last shipped.
   Standard phase-close chore — should land in the same post-hoc commit
   as item 1. (P79 and P80 close had this same advisory.)

3. **SURPRISES-INTAKE entry 1 (T01 bind-verb schedule) STATUS=OPEN.**
   The OPEN status is correct per OP-8 — P88 evaluates whether the
   bind verb should grow a `--test-pending` flag, not P81. Routed
   correctly. No action required from P81.

---

## Summary

3/3 catalog rows PASS (artifact-checked, not status-claimed). 3/3
DVCS-PERF-L1-* requirements have observable test coverage (precheck.rs
implementation + perf_l1.rs N=200 regression test + positive-control
sibling + sync.rs CLI surface + sync.rs 3-test smoke suite).
CLAUDE.md updated in-phase with the L1 conflict-detection Architecture
paragraph (cache-trusted-as-prior + REST-404-on-delete +
sync-reconcile-recovery + L2/L3-deferral) AND the § Commands
`reposix sync --reconcile` bullet. Pre-push gate 26/26 GREEN. All 5
P81 commits (`3a44a9e`, `1bd50b5`, `9321499`, `d21160c`, `4869545`)
pushed to `origin/main`. All 4 PLAN-CHECK HIGH issues (H1-H4) are
substantively fixed in code: `read_blob_cached` (sync, gix-local) is
present and used; `inner_ref()` replaces non-existent `peek()`; State
is `pub(crate)` with the right field-widening; `anyhow::Result` +
`.context(...)` replaces fictional typed Error variants. The perf
regression test is non-vacuous: both wiremock matchers attach to mocks
with active expectations (expect(0) on `NoSinceQueryParam` + expect(1..)
on `HasSinceQueryParam`); the positive-control sibling panics with
"Verifications failed" via `#[should_panic]` confirming the assertion
fails RED on flip. SURPRISES-INTAKE captures both P81 deviations
(bind-verb schedule shift OPEN; refresh_for_mirror_head no-op skip
RESOLVED).

**Verdict: GREEN.** Phase 81 ships. Three advisory items above route
to a small follow-up commit (post-hoc 81-01-SUMMARY + REQUIREMENTS/STATE
flips) but do not loop the phase back.
