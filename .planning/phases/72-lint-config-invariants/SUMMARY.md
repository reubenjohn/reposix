---
phase: 72-lint-config-invariants
plan: 01
subsystem: docs-alignment / code-dimension
tags: [lint-invariants, docs-alignment, quality-gates, v0.12.1]
provides:
  - quality/gates/code/lint-invariants/ (8 shell verifiers + sub-area README)
  - 9 catalog rows transitioned MISSING_TEST -> BOUND
  - quality/reports/verdicts/p72/{status-before,status-after,summary-before,summary-after,delta}.{txt,json}
affects:
  - docs/development/contributing.md (re-measured test-count prose, D-06)
  - CLAUDE.md (P72 H3 under v0.12.1 — in flight, D-10)
  - crates/reposix-sim/src/main.rs (eager-fix per D-09 / OP-8)
  - .planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md (1 LOW entry)
metrics:
  duration: ~2h wall-clock (warm cargo cache)
  completed_date: 2026-04-29
  commits: 14
  catalog_rows_transitioned: 9
  alignment_ratio_delta: "+0.0195 (0.8743 -> 0.8938)"
  claims_missing_test_delta: "-9 (22 -> 13)"
---

# Phase 72 Plan 01: Lint-config invariants Summary

Bound 9 README + docs/development/contributing.md workspace-level invariants to 8 single-purpose shell verifiers under quality/gates/code/lint-invariants/. The walker now detects drift on prose AND verifier file body for forbid-unsafe / MSRV / clippy missing_errors_doc / cargo-check / cargo-test-count / stable-channel / demo-script claims.

## Completed tasks (14 / 14)

| #  | Wave | Task                                                            | Commit    | Requirement        |
| -- | ---- | --------------------------------------------------------------- | --------- | ------------------ |
| 1  | 1    | Catalog-first scaffold (8 stubs + sub-area README + BEFORE)     | 3a1bd0c   | LINT-CONFIG-01..09 |
| 2  | 2    | demo-script-exists.sh                                           | 7a17293   | LINT-CONFIG-08     |
| 3  | 2    | forbid-unsafe-code.sh + eager-fix reposix-sim/src/main.rs       | 181a0fa   | LINT-CONFIG-01     |
| 4  | 2    | rust-msrv.sh                                                    | f358651   | LINT-CONFIG-02     |
| 5  | 2    | rust-stable-channel.sh                                          | ed0198f   | LINT-CONFIG-05     |
| 6  | 2    | cargo-check-workspace.sh                                        | 149512b   | LINT-CONFIG-06     |
| 7  | 2    | errors-doc-section.sh (clippy::missing_errors_doc per D-07)     | 7141ba7   | LINT-CONFIG-04     |
| 8  | 2    | cargo-test-count.sh (initial floor 50)                          | d6d3d45   | LINT-CONFIG-07     |
| 9  | 2    | tests-green.sh (compile-only per D-05)                          | 468e249   | LINT-CONFIG-03     |
| 10 | 3    | Re-measure cargo-test count -> 368, sync prose + floor (D-06)   | bc6e7f6   | LINT-CONFIG-07     |
| 11 | 4    | Bind 9 catalog rows                                             | 14f8224   | LINT-CONFIG-01..09 |
| 12 | 4    | walk + AFTER snapshot + SURPRISES-INTAKE LOW entry              | 0ffc5b0   | LINT-CONFIG-01..09 |
| 13 | 4    | CLAUDE.md H3 subsection (D-10)                                  | 269b75b   | LINT-CONFIG-09     |
| 14 | 4    | This SUMMARY.md                                                 | _pending_ | (none)             |

## Catalog row transitions (MISSING_TEST -> BOUND, 9 rows)

1. README-md/forbid-unsafe-code -> forbid-unsafe-code.sh
2. README-md/rust-1-82-requirement -> rust-msrv.sh
3. README-md/tests-green -> tests-green.sh
4. docs-development-contributing-md/forbid-unsafe-per-crate -> forbid-unsafe-code.sh (D-01 shared)
5. docs-development-contributing-md/errors-doc-section-required -> errors-doc-section.sh
6. docs-development-contributing-md/rust-stable-no-nightly -> rust-stable-channel.sh
7. docs-development-contributing-md/cargo-check-workspace-available -> cargo-check-workspace.sh
8. docs-development-contributing-md/cargo-test-133-tests -> cargo-test-count.sh
9. docs-development-contributing-md/demo-script-exists -> demo-script-exists.sh

## Alignment ratio delta

| Metric                | BEFORE | AFTER  | Delta   |
| --------------------- | ------ | ------ | ------- |
| claims_total          | 388    | 388    | 0       |
| claims_bound          | 313    | 320    | +7      |
| claims_missing_test   | 22     | 13     | -9      |
| claims_retired        | 30     | 30     | 0       |
| alignment_ratio       | 0.8743 | 0.8938 | +0.0195 |
| coverage_ratio        | 0.2031 | 0.2031 | 0       |

Sanity gate (AFTER_missing <= BEFORE_missing - 9): PASSED (22 - 9 = 13).

claims_bound rose only +7 (vs the 9 P72 rows newly bound) because 2 pre-existing BOUND rows flipped to STALE_DOCS_DRIFT during the post-bind walk — both cite files OUTSIDE P72's planned modification set. Logged to SURPRISES-INTAKE.md.

## Verifier scripts shipped (8 files)

All `#!/usr/bin/env bash` + `set -euo pipefail`, chmod +x, exit 0 GREEN against current workspace, name failing files/lines in stderr (Principle B). Memory budget (D-04): cargo invocations serialized.

| Verifier                       | Mechanism                                                                                  |
| ------------------------------ | ------------------------------------------------------------------------------------------ |
| forbid-unsafe-code.sh          | find + grep over crates/*/src/{lib,main}.rs (covers TWO rows per D-01)                     |
| rust-msrv.sh                   | grep Cargo.toml for `rust-version = "1.82"`                                                |
| tests-green.sh                 | cargo test --workspace --no-run (compile-only per D-05)                                    |
| errors-doc-section.sh          | cargo clippy ... -W clippy::missing_errors_doc, jq=0 (D-07; not grep)                      |
| rust-stable-channel.sh         | grep rust-toolchain.toml for `channel = "stable"`                                          |
| cargo-check-workspace.sh       | cargo check --workspace -q                                                                 |
| cargo-test-count.sh            | cargo test --workspace --no-run --message-format=json, unique-by-target.name >= 368        |
| demo-script-exists.sh          | [ -x scripts/dark-factory-test.sh ]                                                        |

## Prose updates

docs/development/contributing.md:20 re-measured to ">= 368 test binaries" (was: "133 tests") BEFORE the row bind, per D-06. Semantic shift from individual #[test] fns to test BINARIES, matching what the verifier asserts. Commit bc6e7f6. README.md does NOT cite a test count (verified via grep); no README change needed.

## CLAUDE.md update (D-10)

New `## v0.12.1 — in flight` H2 + `### P72 — Lint-config invariants` H3 (commit 269b75b). 18 lines under the 30-line cap. Cross-references quality/PROTOCOL.md (Principle A) and CLAUDE.md "Build memory budget" (D-04). Banned-words lint passed.

## +2 phase practice (OP-8) audit trail

**Eager-fixed in-phase (D-09 satisfied):**
- crates/reposix-sim/src/main.rs missing #![forbid(unsafe_code)] — single-file fix, folded into commit 181a0fa alongside the verifier (< 1h, < 5 files, no new dep).

**Appended to SURPRISES-INTAKE.md (P76 to drain):**
- 2 pre-existing BOUND rows flipped to STALE_DOCS_DRIFT during the post-bind walk:
  - planning-milestones-v0-11-0-phases-REQUIREMENTS-md/polish-03-mermaid-render
  - docs/decisions/009-stability-commitment/cli-subcommand-surface
- Both cite files OUTSIDE P72's set; severity LOW; entry at .planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md 2026-04-29 19:58. SCOPE BOUNDARY honoured per executor.md.

**Appended to GOOD-TO-HAVES.md:** none — no polish items observed.

Honesty check: every observation made during execution either landed in-phase or was logged to intake. The single-file forbid-unsafe gap is documented as eager-fix (commit 181a0fa); the 2 STALE_DOCS_DRIFT spillovers are documented as intake (LOW, P76 to triage).

## Verifier dispatch — TOP-LEVEL ORCHESTRATOR ACTION

Per D-08 + CLAUDE.md OP-7 + quality/PROTOCOL.md § "Verifier subagent prompt template": the executing agent does NOT grade itself. After this SUMMARY commits, the top-level orchestrator MUST dispatch:

```
Task(subagent_type="gsd-verifier",          # or "general-purpose" if unavailable
     description="P72 verifier dispatch",
     prompt=<verbatim QG-06 prompt template from quality/PROTOCOL.md, with N=72>)
```

Inputs the verifier reads with ZERO session context:
- quality/catalogs/doc-alignment.json (the 9 row ids; last_verdict == BOUND, tests[0] under quality/gates/code/lint-invariants/)
- .planning/milestones/v0.12.1-phases/REQUIREMENTS.md (LINT-CONFIG-01..09)
- quality/reports/verdicts/p72/{status-before.txt, status-after.txt, summary-before.json, summary-after.json, delta.txt}
- CLAUDE.md (confirms P72 H3 in `git diff main...HEAD -- CLAUDE.md`)
- .planning/milestones/v0.12.1-phases/SURPRISES-INTAKE.md (honesty check per OP-8 — 1 LOW entry from P72)

Verdict path: quality/reports/verdicts/p72/VERDICT.md

Phase does NOT close until verdict graded GREEN.

## Self-Check: PASSED

Files spot-check (all 14 artifacts):
- FOUND: 8 verifier .sh files under quality/gates/code/lint-invariants/
- FOUND: quality/gates/code/lint-invariants/README.md
- FOUND: quality/reports/verdicts/p72/{status-before,status-after}.txt
- FOUND: quality/reports/verdicts/p72/{summary-before,summary-after}.json
- FOUND: quality/reports/verdicts/p72/delta.txt

Commits spot-check:
- FOUND: 3a1bd0c (catalog-first), 181a0fa (forbid-unsafe + eager-fix), 14f8224 (bind), 269b75b (CLAUDE.md), 0ffc5b0 (walk + intake), bc6e7f6 (re-measure)

All claims verifiable from committed artifacts.