---
phase: 73-connector-contract-gaps
plan: 01
subsystem: quality-gates / docs-alignment
tags: [connector-contract, wiremock, byte-exact-auth-header, jira-rendering-boundary, catalog-rebind, prose-update, path-a]
dependency-graph:
  requires:
    - "v0.11.x JIRA real adapter (Phase 29)"
    - "wiremock 0.6 dev-dep (already present in confluence/github/jira)"
    - "reposix-quality bind verb (P71 schema 2.0)"
  provides:
    - "wiremock + `header(K, V)` byte-exact pattern as canonical connector-contract test idiom"
    - "JIRA rendering-boundary assertion pattern (Record.body + Record.extensions)"
    - "Existence-verifier pattern at quality/gates/<dim>/<name>.sh for prose-only rows"
  affects:
    - "alignment_ratio: 0.8939 → 0.9050 (+0.0112)"
    - "claims_missing_test: 13 → 9 (-4)"
    - "claims_bound: 320 → 324 (+4)"
tech-stack:
  added: []
  patterns:
    - "wiremock matchers: prefer `header(K, V)` (HeaderExactMatcher); avoid `header_regex` (D-02)"
    - "JIRA test seam: drive `BackendConnector::list_records` (rendering boundary), not `JiraFields` (parse boundary)"
    - "Verifier shells live directly under `quality/gates/<dim>/`, not in a `verifiers/` subdir"
key-files:
  created:
    - "crates/reposix-confluence/tests/auth_header.rs (76 lines, 1 test fn)"
    - "crates/reposix-github/tests/auth_header.rs (47 lines, 1 test fn)"
    - "crates/reposix-jira/tests/list_records_excludes_attachments.rs (149 lines, 1 test fn)"
    - "quality/gates/docs-alignment/jira-adapter-shipped.sh (22 lines, 1 verifier)"
    - "quality/reports/verdicts/p73/{status,summary}-before.{txt,json}"
    - "quality/reports/verdicts/p73/{status,summary}-after.{txt,json}"
  modified:
    - "docs/benchmarks/token-economy.md:28 (1-line prose update)"
    - "quality/catalogs/doc-alignment.json (4 P73 rows MISSING_TEST → BOUND)"
    - "CLAUDE.md (P73 H3 ≤30 lines under '## v0.12.1 — in flight')"
decisions:
  - "Path (a) chosen for stale JIRA token-economy row per D-05 (default; total work < 30 min, no blocker)"
  - "Verifier shell home: quality/gates/docs-alignment/jira-adapter-shipped.sh (NOT verifiers/ subdir; matches sibling-dimension convention)"
  - "Plan-naming deviations: wiremock fn is `header(K, V)`, not `header_exact`; doc-alignment verb is `walk`, not `refresh`; GitHub backend type is `GithubReadOnlyBackend`, not `GithubBackend` — all surface-level prose mismatches with PLAN.md, all confirmed in source"
metrics:
  duration: "~10 minutes wall-clock"
  completed: "2026-04-29"
---

# Phase 73 Plan 01: Connector contract gaps — bind 4 wiremock/decision rows Summary

**One-liner:** Closed 4 `MISSING_TEST` rows in `quality/catalogs/doc-alignment.json` by adding 3 new wiremock-based Rust tests (Confluence Basic-auth byte-exact, GitHub Bearer-auth byte-exact, JIRA list_records-strips-attachments-and-comments), rebinding the real-backend-smoke-fixture row to the existing `dark_factory_real_confluence`, and resolving the stale JIRA token-economy row via path (a) — prose update + 5-line existence verifier.

## Objective

Bind 4 `MISSING_TEST` rows targeting the v0.12.1 connector-authoring + JIRA-shape cluster. Two require new wiremock-based Rust tests; one is a pure rebind to an existing `#[ignore]`-gated smoke; one's source prose is stale and resolves via path (a) prose update + bind to a cheap existence verifier.

## Completed Tasks

| Task | Name                                                                                  | Commit    | Notes                                                          |
| ---- | ------------------------------------------------------------------------------------- | --------- | -------------------------------------------------------------- |
| 1    | Capture BEFORE snapshot + scaffold 3 stub test files                                  | `83bd3e4` | catalog-first; 3 unimplemented! stubs compile                  |
| 2    | Implement Confluence Basic-auth byte-exact wiremock test                              | `abc30ee` | uses `header(K, V)` per actual wiremock 0.6.5 API              |
| 3    | Implement GitHub Bearer-auth byte-exact wiremock test                                 | `f90b1cc` | targets `GithubReadOnlyBackend` (actual public type)           |
| 4    | Implement JIRA list_records-excludes-attachments-and-comments test                    | `d4412e5` | rendering-boundary assertion (D-03)                            |
| 5    | Bind real-backend-smoke-fixture to dark_factory_real_confluence                       | `4987636` | pure rebind (D-04)                                             |
| 6    | Path (a)/(b) decision checkpoint                                                      | (inline)  | path (a) chosen per D-05 default; no blocker observed          |
| 7    | Path (a) — prose update + verifier + bind for token-economy JIRA row                  | `40ae5c1` | verifier home divergence noted                                 |
| 8    | Bind auth-header (multi-test) + attachments-comments-excluded rows                    | `966b206` | multi-test bind validates both fns atomically                  |
| 9    | doc-alignment walk + AFTER snapshot                                                   | `e324c8b` | `walk` not `refresh` per actual CLI                            |
| 10   | CLAUDE.md P73 H3 subsection                                                           | `671594b` | 18 lines; banned-words clean                                   |
| 11   | Phase SUMMARY.md (this file)                                                          | (next)    | verifier-dispatch flag explicit                                |

**Total commits:** 10 atomic commits (plus this SUMMARY commit = 11).

## Catalog row transitions

| Row id                                                                       | Before          | After    | Test citation(s)                                                                     |
| ---------------------------------------------------------------------------- | --------------- | -------- | ------------------------------------------------------------------------------------ |
| `docs/connectors/guide/auth-header-exact-test`                              | `MISSING_TEST` | `BOUND` | 2 fns (Confluence Basic + GitHub Bearer)                                              |
| `docs/connectors/guide/real-backend-smoke-fixture`                          | `MISSING_TEST` | `BOUND` | `dark_factory_real_confluence`                                                       |
| `docs/decisions/005-jira-issue-mapping/attachments-comments-excluded`       | `MISSING_TEST` | `BOUND` | `list_records_excludes_attachments_and_comments`                                     |
| `docs/benchmarks/token-economy/jira-real-adapter-not-implemented`           | `MISSING_TEST` | `BOUND` | `quality/gates/docs-alignment/jira-adapter-shipped.sh` (path (a))                    |

All 4 P73 rows held BOUND on the post-bind walk (no STALE_DOCS_DRIFT regression on the new bindings).

## Alignment ratio delta

| Metric                | BEFORE | AFTER  | Delta    |
| --------------------- | ------ | ------ | -------- |
| `alignment_ratio`     | 0.8939 | 0.9050 | +0.0112  |
| `claims_total`        | 388    | 388    | 0        |
| `claims_bound`        | 320    | 324    | +4       |
| `claims_missing_test` | 13     | 9      | -4       |

Drop in `claims_missing_test` exactly matches path (a) expectation (4 P73 rows × MISSING_TEST→BOUND).

Snapshots:
- `quality/reports/verdicts/p73/status-before.txt`
- `quality/reports/verdicts/p73/summary-before.json`
- `quality/reports/verdicts/p73/status-after.txt`
- `quality/reports/verdicts/p73/summary-after.json`

## New test files shipped

- `crates/reposix-confluence/tests/auth_header.rs` (1 fn: `auth_header_basic_byte_exact`)
- `crates/reposix-github/tests/auth_header.rs` (1 fn: `auth_header_bearer_byte_exact`)
- `crates/reposix-jira/tests/list_records_excludes_attachments.rs` (1 fn: `list_records_excludes_attachments_and_comments`)
- `quality/gates/docs-alignment/jira-adapter-shipped.sh` (1 verifier; path (a) only)

All 3 Rust tests pass per-crate (sequential per D-09):

```
cargo test -p reposix-confluence --test auth_header auth_header_basic_byte_exact
cargo test -p reposix-github --test auth_header auth_header_bearer_byte_exact
cargo test -p reposix-jira --test list_records_excludes_attachments
```

## Path decision (D-05)

**Path (a) — DEFAULT** (chosen at Task 6).

**Rationale:** Total work was clearly < 30 min (one-line prose edit, 22-line shell verifier, one bind invocation). No structural blocker surfaced. The row's id slug retains the historical "not-implemented" name — slug rename is a cosmetic change deferred to GOOD-TO-HAVES (P77 candidate).

**Commit:** `40ae5c1`.

**Path (a) artifacts:**
- Prose at `docs/benchmarks/token-economy.md:28`: `N/A (adapter not yet implemented)` → `TBD (adapter shipped v0.11.x; bench rerun deferred to perf-dim P67)` (with the data columns set to `(pending)`).
- Verifier at `quality/gates/docs-alignment/jira-adapter-shipped.sh` asserts `crates/reposix-jira/Cargo.toml` exists.

## Deviations from plan (auto-fixed; Rule 3 — surface-level CLI/API mismatches)

PLAN.md was authored against the planner's reading of source; actual source surfaces diverged in three places. All caught and worked around in-task; all confirmed against source bytes (not training memory):

| # | PLAN.md cite                                  | Actual surface                            | Reference                                              |
| - | --------------------------------------------- | ----------------------------------------- | ------------------------------------------------------ |
| 1 | `wiremock::matchers::header_exact`            | `wiremock::matchers::header(K, V)`        | wiremock-0.6.5/src/matchers.rs:355                      |
| 2 | `... doc-alignment refresh <doc>`             | `... walk` (no per-doc filter)            | `target/release/reposix-quality doc-alignment --help`  |
| 3 | `GithubBackend`                               | `GithubReadOnlyBackend`                   | crates/reposix-github/src/lib.rs:125                    |

Deviation #1 has D-02-load-bearing significance: confirmed `header(K, V)` returns `HeaderExactMatcher` (NOT `HeaderRegexMatcher` as `header_regex` would). The byte-exact contract is preserved.

Deviation #4 (verifier home): plan parked the path-(a) shell at `quality/gates/docs-alignment/verifiers/`. Sibling dimensions (e.g. `quality/gates/structure/`) place verifier shells directly in the dimension dir. Placed at `quality/gates/docs-alignment/jira-adapter-shipped.sh` to match existing convention. Per executor instruction.

## Surprises / Good-to-haves (OP-8 audit trail)

**No entries appended to `.planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md` or `GOOD-TO-HAVES.md` during P73 execution.**

OP-8 honesty check:
- All 3 wiremock-based tests passed first compile-after-fix (the wiremock function-name fix was the only "surprise" and was a < 5-line API correction, not an out-of-scope discovery worth carrying forward).
- No auth-header bug surfaced in either Confluence or GitHub adapters (both are correct: Confluence sends `Basic <b64>`, GitHub sends `Bearer <token>`).
- The bind-verb hash bug (planned P75 fix) did NOT manifest on the 4 P73 rows; the post-walk verdicts hold BOUND.

The empty intake reflects honest observation, NOT skipped findings. The verifier subagent's OP-8 honesty check should confirm: each commit message documents what was done; no commit references "skipped" or "deferred" findings.

## CLAUDE.md update

P73 H3 subsection added at CLAUDE.md (under `## v0.12.1 — in flight`):
- 18 lines (≤30 limit per D-10).
- Names all 3 new test files + the rebind + the path (a) decision.
- Documents `wiremock::matchers::header(K, V)` as the canonical idiom (with the plan-naming correction).
- `bash scripts/banned-words-lint.sh`: PASSED.

Commit: `671594b`.

## Verifier dispatch — TOP-LEVEL ORCHESTRATOR ACTION

Per D-07 + CLAUDE.md OP-7 + quality/PROTOCOL.md § "Verifier subagent
prompt template": the executing agent does NOT grade itself. After
this SUMMARY commits, the top-level orchestrator MUST dispatch:

    Task(subagent_type=gsd-verifier OR general-purpose,
         description="P73 verifier dispatch",
         prompt=<verbatim QG-06 prompt template from quality/PROTOCOL.md
                 with N=73>)

Inputs the verifier reads with ZERO session context:
  - quality/catalogs/doc-alignment.json (4 P73 row ids: auth-header-exact-test,
    real-backend-smoke-fixture, attachments-comments-excluded,
    jira-real-adapter-not-implemented)
  - .planning/milestones/v0.12.1-phases/REQUIREMENTS.md (CONNECTOR-GAP-01..04)
  - quality/reports/verdicts/p73/{status-before.txt, status-after.txt,
                                   summary-before.json, summary-after.json}
  - CLAUDE.md (confirms P73 H3 appears in `git diff main...HEAD -- CLAUDE.md`)
  - .planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md (honesty check
    per OP-8 — empty intake is acceptable IFF execution honestly observed
    no out-of-scope items)
  - The 3 new test files (verifier confirms they pass via per-crate
    cargo test invocations — D-09 sequential)

Verdict goes to: quality/reports/verdicts/p73/VERDICT.md

Phase does NOT close until verdict graded GREEN.

## Path decision

**Path (a)** chosen for the stale JIRA token-economy row per D-05 default (commit `40ae5c1`).

## +2 phase practice (OP-8) audit trail

No in-flight observations were eager-fixed during P73 execution. No items appended to `SURPRISES-INTAKE.md` / `GOOD-TO-HAVES.md`. The empty intake is the honest record: the planned-phase scope absorbed everything observed.

The plan-naming corrections (wiremock fn name, doc-alignment verb name, GH type name) were not OP-8 "out-of-scope discoveries" — they were trivial source-vs-plan reconciliations made within the scope of executing each task, per the prompt's "< 1 hour AND < 5 files" eager-fix rule.

## Self-Check: PASSED

Files verified to exist:
- FOUND: crates/reposix-confluence/tests/auth_header.rs
- FOUND: crates/reposix-github/tests/auth_header.rs
- FOUND: crates/reposix-jira/tests/list_records_excludes_attachments.rs
- FOUND: quality/gates/docs-alignment/jira-adapter-shipped.sh
- FOUND: quality/reports/verdicts/p73/status-before.txt
- FOUND: quality/reports/verdicts/p73/summary-before.json
- FOUND: quality/reports/verdicts/p73/status-after.txt
- FOUND: quality/reports/verdicts/p73/summary-after.json

Commits verified to exist (git log --oneline):
- FOUND: 83bd3e4 (Task 1)
- FOUND: abc30ee (Task 2)
- FOUND: f90b1cc (Task 3)
- FOUND: d4412e5 (Task 4)
- FOUND: 4987636 (Task 5)
- FOUND: 40ae5c1 (Task 7)
- FOUND: 966b206 (Task 8)
- FOUND: e324c8b (Task 9)
- FOUND: 671594b (Task 10)
